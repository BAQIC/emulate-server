use core::error;
use std::net::SocketAddr;

use super::entity;
use super::entity::sea_orm_active_enums;
use super::service;
use axum::{
    extract::{Query, Request, State},
    http::{header, StatusCode},
    Form, Json, RequestExt,
};
use log::{error, info};
use qasmsim;
use sea_orm::DbConn;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct EmulateMessage {
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

#[derive(Deserialize)]
pub struct AgentNum {
    agent_num: String,
}

/// Convert entity::Model::Options to qasmsim::options::Options
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

/// Merge fields into a json object
pub fn merge_json(v: &Value, fields: Vec<(String, String)>) -> Value {
    // If v is not an object, return v. Otherwise, merge fields into v
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

/// This method is used to test the server
pub async fn root() -> (StatusCode, Json<Value>) {
    info!("Root request");
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
        Ok(result) => {
            info!("Root request success");
            (
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
            )
        }
        Err(err) => {
            error!("Root request failed: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}

/// Direct consume the task and return the result without database operation
#[deprecated(since = "0.1.0", note = "Please use `comsume_task` instead")]
async fn consume_body(Form(emulate_message): Form<EmulateMessage>) -> (StatusCode, Json<Value>) {
    info!("Consume body in emulate request");
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
        Ok(result) => {
            info!("Consume body in emulate request success");
            (
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
            )
        }
        Err(err) => {
            error!("Consume body in emulate request failed: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}

/// Emulate the quantum circuit and return the result with database operation
#[deprecated(since = "0.1.0", note = "Please use `submit` instead")]
pub async fn emulate(request: Request) -> (StatusCode, Json<Value>) {
    // curl -v -H "Content-Type: x-www-form-urlencoded" -X POST
    // 10.31.4.69:3000/emulate -d @bell.qasm -d shots=1000 -d format=json
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == "application/x-www-form-urlencoded" => {
            let Form(emulate_message) = request.extract().await.unwrap();
            consume_body(Form(emulate_message)).await
        }
        _ => {
            error!(
                "Emulate request failed: content type {:?} not specified / not support",
                request.headers().get(header::CONTENT_TYPE).unwrap()
            );
            (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"Error": format!("content type {:?} not specified / not support", request.headers().get(header::CONTENT_TYPE).unwrap())}),
                ),
            )
        }
    }
}

/// Initialize the qthread with num of physical agents
pub async fn init_qthread(
    state: State<ServerState>,
    query_message: Query<AgentNum>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Init qthread with {:?} physical agents",
        query_message.0.agent_num
    );
    let db = &state.db;
    match service::resource::Resource::random_init_physical_agents(
        db,
        query_message.0.agent_num.parse::<u32>().unwrap(),
    )
    .await
    {
        Ok(_) => {
            info!(
                "Add {:?} physical agents added successfully",
                query_message.0.agent_num
            );
            (
                StatusCode::OK,
                Json(json!({"Message": "Physical Agents added successfully"})),
            )
        }
        Err(err) => {
            error!("Add physical agents failed: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}

/// Consume the task when the server receives a submit request
pub async fn consume_task(
    Form(emulate_message): Form<EmulateMessage>,
    state: State<ServerState>,
) -> (StatusCode, Json<Value>) {
    info!("Consume task in submit request");
    // get qasm simulation options
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

    // add this task to the database
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

    info!("Task {:?} added successfully", task.id);

    // If there is an available agent, run the task. Otherwise, return the task id
    match task.status {
        sea_orm_active_enums::TaskStatus::Running => {
            let result = qasmsim::run(&task.source, options.shots);
            service::qthread::Qthread::finish_task(
                &state.db,
                task.id,
                match result {
                    // If the task is simulated successfully, update the task status to succeeded.
                    // Otherwise, update the task status to failed/
                    Ok(ref result) => (
                        Some(qasmsim::print_result(result, &options)),
                        sea_orm_active_enums::TaskStatus::Succeeded,
                        sea_orm_active_enums::AgentStatus::Succeeded,
                    ),
                    Err(ref err) => (
                        // Add the error message to the result field
                        Some(format!("{}", err)),
                        sea_orm_active_enums::TaskStatus::Failed,
                        sea_orm_active_enums::AgentStatus::Failed,
                    ),
                },
            )
            .await
            .unwrap();

            match result {
                Ok(result) => {
                    info!("Task {:?} is succeeded", task.id);
                    (
                        StatusCode::OK,
                        match options.format {
                            // Merge the result with the task id
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
                    )
                }
                // The simulation is failed
                Err(err) => {
                    error!("Task {:?} is failed: {}", task.id, err);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "Message": "Task is failed",
                            "Result": Some(format!("{}", err)),
                            "task_id": task.id
                        })),
                    )
                }
            }
        }
        sea_orm_active_enums::TaskStatus::NotStarted => {
            info!("Task {:?} is waiting", task.id);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "Message": "There is no available agent to run the task",
                    "task_id": task.id
                })),
            )
        }
        status => {
            error!("Task status {:?} is not valid", status);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "Error": format!("Task status {:?} is not valid", status),
                    "task_id": task.id
                })),
            )
        }
    }
}

/// Consume the task when there are some waiting tasks and available agents
pub async fn consume_task_back(db: &DbConn, waiting_task: entity::task::Model) {
    info!("Consume task in consume waiting task thread");
    let task = service::qthread::Qthread::submit_task_without_add(&db, waiting_task)
        .await
        .unwrap();

    // If there is an available agent, run the task. Otherwise, do nothing
    match task.status {
        sea_orm_active_enums::TaskStatus::Running => {
            let qasm_option = model_option_to_qasm_option(
                service::options::Options::get_option(&db, task.option_id)
                    .await
                    .unwrap()
                    .unwrap(),
            );

            // Qasm simulation
            let result = qasmsim::run(&task.source, qasm_option.shots);
            service::qthread::Qthread::finish_task(
                &db,
                task.id,
                match result {
                    Ok(result) => {
                        info!("Task {:?} is succeeded", task.id);
                        (
                            Some(qasmsim::print_result(&result, &qasm_option)),
                            sea_orm_active_enums::TaskStatus::Succeeded,
                            sea_orm_active_enums::AgentStatus::Succeeded,
                        )
                    }
                    Err(err) => {
                        error!("Task {:?} is failed: {}", task.id, err);
                        (
                            Some(format!("{}", err)),
                            sea_orm_active_enums::TaskStatus::Failed,
                            sea_orm_active_enums::AgentStatus::Failed,
                        )
                    }
                },
            )
            .await
            .unwrap();
        }
        sea_orm_active_enums::TaskStatus::NotStarted => {
            info!("Task {:?} is waiting", task.id);
        }
        status => {
            error!("Task {:?} status {:?} is not valid", task.id, status);
        }
    }
}

/// Submit the task to the server
pub async fn submit(state: State<ServerState>, request: Request) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        // If the content type is correct, consume the task
        Some(content_type) if content_type == "application/x-www-form-urlencoded" => {
            let Form(emulate_message) = request.extract().await.unwrap();
            consume_task(Form(emulate_message), state).await
        }
        _ => {
            error!(
                "Submit request failed: content type {:?} not specified / not support",
                request.headers().get(header::CONTENT_TYPE).unwrap()
            );
            (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({"Error": format!("content type {:?} not specified / not support", request.headers().get(header::CONTENT_TYPE).unwrap())}),
                ),
            )
        }
    }
}

/// Get the task status by task id
pub async fn get_task(
    state: State<ServerState>,
    query_message: Query<TaskID>,
) -> (StatusCode, Json<Value>) {
    info!("Get task status by task id: {:?}", query_message.task_id);
    let db = &state.db;
    match service::task::Task::get_task(db, uuid::Uuid::parse_str(&query_message.task_id).unwrap())
        .await
    {
        Ok(task) => match task {
            Some(task) => match task.status {
                sea_orm_active_enums::TaskStatus::Running => {
                    info!("Task {:?} is running", task.id);
                    (
                        StatusCode::OK,
                        Json(json!({
                            "Message": "Task is running",
                            "task_id": task.id
                        })),
                    )
                }
                sea_orm_active_enums::TaskStatus::NotStarted => {
                    info!("Task {:?} is waiting", task.id);
                    (
                        StatusCode::OK,
                        Json(json!({
                            "Message": "Task is waiting",
                            "task_id": task.id
                        })),
                    )
                }
                sea_orm_active_enums::TaskStatus::Succeeded => {
                    info!("Task {:?} is succeeded", task.id);
                    (
                        StatusCode::OK,
                        Json(merge_json(
                            &serde_json::from_str::<Value>(&task.result.unwrap()).unwrap(),
                            vec![
                                ("Message".to_owned(), "Task is succeeded".to_owned()),
                                ("task_id".to_owned(), task.id.to_string()),
                            ],
                        )),
                    )
                }
                sea_orm_active_enums::TaskStatus::Failed => {
                    info!("Task {:?} is failed", task.id);
                    (
                        StatusCode::OK,
                        Json(json!({
                            "Message": "Task is failed",
                            "task_id": task.id,
                            "result": task.result.unwrap()
                        })),
                    )
                }
            },
            None => {
                info!("Task with id {:?} not found", query_message.task_id);
                (
                    StatusCode::BAD_REQUEST,
                    Json(
                        json!({"Error": format!("Task with id {} not found", query_message.task_id)}),
                    ),
                )
            }
        },
        Err(err) => {
            error!(
                "Get task {:?} status failed: {}",
                query_message.task_id, err
            );
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}
