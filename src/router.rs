use super::entity;
use super::entity::sea_orm_active_enums;
use super::service;
use axum::{
    extract::{Path, Query, Request, State},
    http::{header, StatusCode},
    Form, Json, RequestExt,
};
use log::{error, info};
use reqwest::Response;
use sea_orm::DbConn;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct EmulateMessage {
    code: String,
    shots: usize,
    agent: AgentType,
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
pub struct AgentInfo {
    pub ip: String,
    pub port: i32,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum AgentType {
    #[serde(rename = "qpp-sv")]
    QppSV,
    #[serde(rename = "qpp-dm")]
    QppDM,
    #[serde(rename = "qasmsim")]
    QASMSim,
    #[serde(rename = "cudaq")]
    CUDAQ,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::QppSV => write!(f, "qpp-sv"),
            AgentType::QppDM => write!(f, "qpp-dm"),
            AgentType::QASMSim => write!(f, "qasmsim"),
            AgentType::CUDAQ => write!(f, "cudaq"),
        }
    }
}

pub struct Options {
    pub shots: Option<usize>,
    pub agent_type: AgentType,
}

/// Convert entity::Model::Options to Options
pub fn model_option_to_qasm_option(option: entity::options::Model) -> Options {
    Options {
        shots: match option.shots {
            Some(shot_num) => Some(shot_num as usize),
            None => None,
        },
        agent_type: match option.agent_type {
            sea_orm_active_enums::AgentType::QppSv => AgentType::QppSV,
            sea_orm_active_enums::AgentType::QppDm => AgentType::QppDM,
            sea_orm_active_enums::AgentType::QasmSim => AgentType::QASMSim,
            sea_orm_active_enums::AgentType::Cudaq => AgentType::CUDAQ,
        },
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

pub async fn invoke_agent(
    address: &str,
    code: &str,
    shots: usize,
    agent: AgentType,
) -> Result<Response, reqwest::Error> {
    let body = [
        ("code", code.to_string()),
        ("shots", shots.to_string()),
        ("agent", agent.to_string()),
    ];

    reqwest::Client::new()
        .post(address)
        .form(&body)
        .send()
        .await
}

/// Initialize the qthread with num of physical agents
pub async fn init_qthread(
    state: State<ServerState>,
    query_message: Query<AgentInfo>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Init qthread with {}:{:?} physical agent",
        query_message.0.ip, query_message.0.port
    );
    let db = &state.db;
    match service::resource::Resource::add_physical_agent(
        db,
        uuid::Uuid::new_v4(),
        &query_message.0.ip,
        query_message.0.port,
    )
    .await
    {
        Ok(_) => {
            info!(
                "Add {}:{} physical agents added successfully",
                query_message.0.ip, query_message.0.port
            );
            (
                StatusCode::OK,
                Json(
                    json!({"Message": format!("Physical Agent {}:{} added successfully", query_message.0.ip, query_message.0.port)}),
                ),
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
    let options = Options {
        shots: if emulate_message.shots == 0 {
            None
        } else {
            Some(emulate_message.shots)
        },
        agent_type: emulate_message.agent,
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
            source: emulate_message.code.clone(),
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
            let physical_agent = service::resource::Resource::get_physical_agent(
                &state.db,
                service::resource::Resource::get_agent(&state.db, task.agent_id.unwrap())
                    .await
                    .unwrap()
                    .unwrap()
                    .physical_id,
            )
            .await
            .unwrap()
            .unwrap();
            let result = invoke_agent(
                &format!(
                    "http://{}:{}/submit",
                    physical_agent.ip, physical_agent.port
                ),
                &emulate_message.code,
                emulate_message.shots,
                emulate_message.agent,
            )
            .await;

            let (status, value) = if result.is_ok() {
                (
                    Some(result.as_ref().unwrap().status()),
                    match result.unwrap().json::<Value>().await {
                        Ok(v) => v,
                        Err(err) => json!({"Error": format!("{}", err)}),
                    },
                )
            } else {
                (None, json!({"Error": format!("{}", result.unwrap_err())}))
            };

            service::qthread::Qthread::finish_task(
                &state.db,
                task.id,
                match status {
                    // If the task is simulated successfully, update the task status to succeeded.
                    // Otherwise, update the task status to failed/
                    Some(s) if s == reqwest::StatusCode::OK => (
                        Some(serde_json::to_string_pretty(&value).expect("json pretty print")),
                        sea_orm_active_enums::TaskStatus::Succeeded,
                        sea_orm_active_enums::AgentStatus::Succeeded,
                    ),
                    Some(_) | None => (
                        Some(serde_json::to_string_pretty(&value).expect("json pretty print")),
                        sea_orm_active_enums::TaskStatus::Failed,
                        sea_orm_active_enums::AgentStatus::Failed,
                    ),
                },
            )
            .await
            .unwrap();

            match status {
                Some(s) if s == reqwest::StatusCode::OK => {
                    info!("Task {:?} is succeeded", task.id);
                    (
                        StatusCode::OK,
                        Json(merge_json(
                            &value,
                            vec![
                                ("status".to_string(), "succeeded".to_string()),
                                ("task_id".to_string(), task.id.to_string()),
                            ],
                        )),
                    )
                }
                Some(_) | None => {
                    error!("Task {:?} is failed", task.id);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(merge_json(
                            &value,
                            vec![
                                ("status".to_string(), "failed".to_string()),
                                ("task_id".to_string(), task.id.to_string()),
                            ],
                        )),
                    )
                }
            }
        }
        sea_orm_active_enums::TaskStatus::NotStarted => {
            info!("Task {:?} is waiting", task.id);
            (
                StatusCode::OK,
                Json(json!({
                    "status": "waiting",
                    "task_id": task.id
                })),
            )
        }
        status => {
            error!("Task status {:?} is not valid", status);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "error",
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
            let option = model_option_to_qasm_option(
                service::options::Options::get_option(&db, task.option_id)
                    .await
                    .unwrap()
                    .unwrap(),
            );

            let physical_agent = service::resource::Resource::get_physical_agent(
                &db,
                service::resource::Resource::get_agent(db, task.agent_id.unwrap())
                    .await
                    .unwrap()
                    .unwrap()
                    .physical_id,
            )
            .await
            .unwrap()
            .unwrap();

            info!(
                "Task {:?} is running on agent {}:{}",
                task.id, physical_agent.ip, physical_agent.port
            );
            let result = invoke_agent(
                &format!(
                    "http://{}:{}/submit",
                    physical_agent.ip, physical_agent.port
                ),
                &task.source,
                match option.shots {
                    Some(s) => s,
                    None => 0,
                },
                option.agent_type,
            )
            .await;

            let (status, value) = if result.is_ok() {
                (
                    Some(result.as_ref().unwrap().status()),
                    match result.unwrap().json::<Value>().await {
                        Ok(v) => v,
                        Err(err) => json!({"Error": format!("{}", err)}),
                    },
                )
            } else {
                (None, json!({"Error": format!("{}", result.unwrap_err())}))
            };

            // Qasm simulation
            service::qthread::Qthread::finish_task(
                &db,
                task.id,
                match status {
                    // If the task is simulated successfully, update the task status to succeeded.
                    // Otherwise, update the task status to failed
                    Some(s) if s == reqwest::StatusCode::OK => {
                        info!("Task {:?} is succeeded", task.id);
                        (
                            Some(serde_json::to_string_pretty(&value).expect("json pretty print")),
                            sea_orm_active_enums::TaskStatus::Succeeded,
                            sea_orm_active_enums::AgentStatus::Succeeded,
                        )
                    }
                    Some(_) | None => {
                        error!("Task {:?} is failed", task.id);
                        (
                            Some(serde_json::to_string_pretty(&value).expect("json pretty print")),
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
        Some(content_type) => match content_type.to_str().unwrap() {
            "application/x-www-form-urlencoded" => {
                let Form(emulate_message) = request.extract().await.unwrap();
                consume_task(Form(emulate_message), state).await
            }
            "application/json" => {
                let Json::<EmulateMessage>(emulate_message) = request.extract().await.unwrap();
                consume_task(Form(emulate_message), state).await
            }
            _ => {
                error!(
                    "Submit request failed: content type {:?} not support",
                    content_type
                );
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"Error": format!("content type {:?} not support", content_type)})),
                )
            }
        },
        _ => {
            error!("Submit request failed: content type not specified");
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("content type not specified")})),
            )
        }
    }
}

pub async fn _get_task(db: &DbConn, task_id: Uuid) -> (StatusCode, Json<Value>) {
    info!("Get task status by task id: {:?}", task_id);
    let db = db;
    match service::task::Task::get_task(db, task_id).await {
        Ok(task) => match task {
            Some(task) => match task.status {
                sea_orm_active_enums::TaskStatus::Running => {
                    info!("Task {:?} is running", task.id);
                    (
                        StatusCode::OK,
                        Json(json!({
                            "status": "running",
                            "task_id": task.id
                        })),
                    )
                }
                sea_orm_active_enums::TaskStatus::NotStarted => {
                    info!("Task {:?} is waiting", task.id);
                    (
                        StatusCode::OK,
                        Json(json!({
                            "status": "waiting",
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
                                ("status".to_owned(), "succeeded".to_owned()),
                                ("task_id".to_owned(), task.id.to_string()),
                            ],
                        )),
                    )
                }
                sea_orm_active_enums::TaskStatus::Failed => {
                    info!("Task {:?} is failed", task.id);
                    (
                        StatusCode::OK,
                        Json(merge_json(
                            &serde_json::from_str::<Value>(&task.result.unwrap()).unwrap(),
                            vec![
                                ("status".to_owned(), "failed".to_owned()),
                                ("task_id".to_owned(), task.id.to_string()),
                            ],
                        )),
                    )
                }
            },
            None => {
                info!("Task with id {:?} not found", task_id);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "task_id": task_id,
                        "status": "error",
                        "Error": "Task not found"
                    })),
                )
            }
        },
        Err(err) => {
            error!("Get task {:?} status failed: {}", task_id, err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "task_id": task_id,
                    "status": "error",
                    "Error": format!("{}", err)
                })),
            )
        }
    }
}

/// Get the task status by task id
pub async fn get_task(
    state: State<ServerState>,
    query_message: Query<TaskID>,
) -> (StatusCode, Json<Value>) {
    _get_task(
        &state.db,
        uuid::Uuid::parse_str(&query_message.task_id).unwrap(),
    )
    .await
}

pub async fn get_task_with_id(
    state: State<ServerState>,
    Path(task_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    _get_task(&state.db, task_id).await
}
