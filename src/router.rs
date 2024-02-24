use super::entity;
use super::entity::sea_orm_active_enums;
use super::service;
use axum::{
    extract::{Query, Request, State},
    http::{header, StatusCode},
    Form, Json, RequestExt,
};
use qasmsim;
use sea_orm::DbConn;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct EmulateMessage {
    qasm: String,
    shots: usize,
    format: String,
}

#[derive(Clone)]
pub struct ServerState {
    pub db: DbConn,
}

#[derive(Deserialize)]
pub struct TaskID {
    task_id: String,
}

pub fn model_option_to_qasm_option(option: entity::options::Model) -> qasmsim::options::Options {
    qasmsim::options::Options {
        shots: match option.shots {
            Some(shot_num) => Some(shot_num as usize),
            None => None,
        },
        format: match option.format {
            sea_orm_active_enums::Format::Json => qasmsim::options::Format::Json,
            sea_orm_active_enums::Format::Tabular => qasmsim::options::Format::Tabular,
        },
        binary: option.binary,
        hexadecimal: option.hexadecimal,
        integer: option.integer,
        statevector: option.statevector,
        probabilities: option.probabilities,
        times: option.times,
    }
}

pub fn merge_json(v: &Value, fields: Vec<(String, String)>) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            for (key, value) in fields {
                m.insert(key, Value::String(value));
            }
            Value::Object(m)
        }
        v => v.clone(),
    }
}

pub async fn root() -> (StatusCode, Json<Value>) {
    let source = "
    OPENQASM 2.0;
    include \"qelib1.inc\";
    qreg q[2];
    creg c[2];
    x q;
    h q;
    measure q -> c;
    ";

    let options = qasmsim::options::Options {
        shots: Some(1000),
        format: qasmsim::options::Format::Json,
        ..Default::default()
    };

    match qasmsim::run(source, options.shots) {
        Ok(result) => (
            StatusCode::OK,
            match options.format {
                qasmsim::options::Format::Json => Json(
                    serde_json::from_str::<Value>(&qasmsim::print_result(&result, &options))
                        .unwrap(),
                ),
                qasmsim::options::Format::Tabular => {
                    Json(json!({"Result": qasmsim::print_result(&result, &options)}))
                }
            },
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}

async fn consume_body(Form(emulate_message): Form<EmulateMessage>) -> (StatusCode, Json<Value>) {
    let options = qasmsim::options::Options {
        shots: if emulate_message.shots == 0 {
            None
        } else {
            Some(emulate_message.shots)
        },
        format: match emulate_message.format.as_str() {
            "json" => qasmsim::options::Format::Json,
            "tabular" => qasmsim::options::Format::Tabular,
            _ => qasmsim::options::Format::Json,
        },
        ..Default::default()
    };

    match qasmsim::run(&emulate_message.qasm, options.shots) {
        Ok(result) => (
            StatusCode::OK,
            match options.format {
                qasmsim::options::Format::Json => Json(
                    serde_json::from_str::<Value>(&qasmsim::print_result(&result, &options))
                        .unwrap(),
                ),
                qasmsim::options::Format::Tabular => {
                    Json(json!({"Result": qasmsim::print_result(&result, &options)}))
                }
            },
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}

pub async fn emulate(request: Request) -> (StatusCode, Json<Value>) {
    // curl -v -H "Content-Type: x-www-form-urlencoded" -X POST 10.31.4.69:3000/emulate -d @bell.qasm -d shots=1000 -d format=json
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == "application/x-www-form-urlencoded" => {
            let Form(emulate_message) = request.extract().await.unwrap();
            consume_body(Form(emulate_message)).await
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"Error": format!("content type {:?} not specified / not support", request.headers().get(header::CONTENT_TYPE).unwrap())}),
            ),
        ),
    }
}

pub async fn init_db(state: State<ServerState>) -> (StatusCode, Json<Value>) {
    let db = &state.db;
    match service::resource::Resource::random_init_physical_agents(db, 10).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"Message": "Physical Agents added successfully"})),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}

pub async fn consume_task(
    Form(emulate_message): Form<EmulateMessage>,
    state: State<ServerState>,
) -> (StatusCode, Json<Value>) {
    let options = qasmsim::options::Options {
        shots: if emulate_message.shots == 0 {
            None
        } else {
            Some(emulate_message.shots)
        },
        format: match emulate_message.format.as_str() {
            "json" => qasmsim::options::Format::Json,
            "tabular" => qasmsim::options::Format::Tabular,
            _ => qasmsim::options::Format::Json,
        },
        ..Default::default()
    };

    let task = service::qthread::Qthread::submit_task(
        &state.db,
        entity::task::Model {
            id: uuid::Uuid::new_v4(),
            option_id: service::options::Options::add_qasm_options(&state.db, &options)
                .await
                .unwrap()
                .id,
            source: emulate_message.qasm.clone(),
            status: sea_orm_active_enums::TaskStatus::NotStarted,
            result: None,
            created_time: chrono::Utc::now().naive_utc(),
            updated_time: chrono::Utc::now().naive_utc(),
            agent_id: None,
        },
    )
    .await
    .unwrap();

    match task.status {
        sea_orm_active_enums::TaskStatus::Running => {
            let result = qasmsim::run(&task.source, options.shots);
            service::qthread::Qthread::finish_task(
                &state.db,
                task.id,
                match result {
                    Ok(ref result) => (
                        Some(qasmsim::print_result(result, &options)),
                        sea_orm_active_enums::TaskStatus::Succeeded,
                        sea_orm_active_enums::AgentStatus::Succeeded,
                    ),
                    Err(ref err) => (
                        Some(format!("{}", err)),
                        sea_orm_active_enums::TaskStatus::Failed,
                        sea_orm_active_enums::AgentStatus::Failed,
                    ),
                },
            )
            .await
            .unwrap();

            match result {
                Ok(result) => (
                    StatusCode::OK,
                    match options.format {
                        qasmsim::options::Format::Json => Json(merge_json(
                            &serde_json::from_str::<Value>(&qasmsim::print_result(
                                &result, &options,
                            ))
                            .unwrap(),
                            vec![
                                ("Message".to_owned(), "Task is succeeded".to_owned()),
                                ("task_id".to_owned(), task.id.to_string()),
                            ],
                        )),
                        qasmsim::options::Format::Tabular => Json(json!({
                            "Message": "Task is succeeded",
                            "Result": Some(qasmsim::print_result(&result, &options)),
                            "task_id": task.id
                        })),
                    },
                ),
                Err(err) => (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "Message": "Task is failed",
                        "Result": Some(format!("{}", err)),
                        "task_id": task.id
                    })),
                ),
            }
        }

        sea_orm_active_enums::TaskStatus::NotStarted => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "Message": "There is no available agent to run the task",
                "task_id": task.id
            })),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": "Task status is not valid"})),
        ),
    }
}

pub async fn consume_task_back(db: &DbConn, waiting_task: entity::task::Model) {
    let task = service::qthread::Qthread::submit_task_without_add(&db, waiting_task)
        .await
        .unwrap();

    match task.status {
        sea_orm_active_enums::TaskStatus::Running => {
            let qasm_option = model_option_to_qasm_option(
                service::options::Options::get_option(&db, task.option_id)
                    .await
                    .unwrap()
                    .unwrap(),
            );

            let result = qasmsim::run(&task.source, qasm_option.shots);
            service::qthread::Qthread::finish_task(
                &db,
                task.id,
                match result {
                    Ok(result) => (
                        Some(qasmsim::print_result(&result, &qasm_option)),
                        sea_orm_active_enums::TaskStatus::Succeeded,
                        sea_orm_active_enums::AgentStatus::Succeeded,
                    ),
                    Err(err) => (
                        Some(format!("{}", err)),
                        sea_orm_active_enums::TaskStatus::Failed,
                        sea_orm_active_enums::AgentStatus::Failed,
                    ),
                },
            )
            .await
            .unwrap();
        }
        sea_orm_active_enums::TaskStatus::NotStarted => {}
        _ => {
            println!("Task status is not valid")
        }
    }
}

pub async fn submit(state: State<ServerState>, request: Request) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == "application/x-www-form-urlencoded" => {
            let Form(emulate_message) = request.extract().await.unwrap();
            consume_task(Form(emulate_message), state).await
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"Error": format!("content type {:?} not specified / not support", request.headers().get(header::CONTENT_TYPE).unwrap())}),
            ),
        ),
    }
}

pub async fn get_task(
    state: State<ServerState>,
    query_message: Query<TaskID>,
) -> (StatusCode, Json<Value>) {
    let db = &state.db;
    match service::task::Task::get_task(db, uuid::Uuid::parse_str(&query_message.task_id).unwrap())
        .await
    {
        Ok(task) => match task {
            Some(task) => match task.status {
                sea_orm_active_enums::TaskStatus::Running => (
                    StatusCode::OK,
                    Json(json!({
                        "Message": "Task is running",
                        "task_id": task.id
                    })),
                ),
                sea_orm_active_enums::TaskStatus::NotStarted => (
                    StatusCode::OK,
                    Json(json!({
                        "Message": "Task is waiting",
                        "task_id": task.id
                    })),
                ),
                sea_orm_active_enums::TaskStatus::Succeeded => (
                    StatusCode::OK,
                    Json(merge_json(
                        &serde_json::from_str::<Value>(&task.result.unwrap()).unwrap(),
                        vec![
                            ("Message".to_owned(), "Task is succeeded".to_owned()),
                            ("task_id".to_owned(), task.id.to_string()),
                        ],
                    )),
                ),
                sea_orm_active_enums::TaskStatus::Failed => (
                    StatusCode::OK,
                    Json(json!({
                        "Message": "Task is failed",
                        "task_id": task.id,
                        "result": task.result.unwrap()
                    })),
                ),
            },
            None => (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("Task with id {} not found", query_message.task_id)})),
            ),
        },
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}
