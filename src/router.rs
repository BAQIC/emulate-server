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
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct EmulateMessage {
    code: String,
    qubits: usize,
    depth: usize,
    shots: usize,
}

#[derive(Clone)]
pub struct ServerState {
    pub db: DbConn,
    pub config: super::config::QSchedulerConfig,
}

pub type SharedState = Arc<RwLock<ServerState>>;

#[derive(Deserialize)]
pub struct TaskID {
    task_id: String,
}

#[derive(Deserialize)]
pub struct AgentInfo {
    pub ip: String,
    pub port: u32,
    pub qubit_count: u32,
    pub circuit_depth: u32,
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
    qasm: &str,
    shots: usize,
) -> Result<Response, reqwest::Error> {
    let body = [("qasm", qasm.to_string()), ("shots", shots.to_string())];

    reqwest::Client::new()
        .post(address)
        .form(&body)
        .send()
        .await
}

/// Initialize the qthread with num of physical agents
pub async fn add_physical_agent(
    State(state): State<SharedState>,
    Query(query_message): Query<AgentInfo>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Init qthread with {}:{:?} physical agent",
        query_message.ip, query_message.port
    );
    let db = &state.write().await.db;

    match service::physical_agent::PhysicalAgent::add_physical_agent(
        db,
        entity::physical_agent::Model {
            id: uuid::Uuid::new_v4(),
            status: sea_orm_active_enums::PhysicalAgentStatus::Running,
            ip: query_message.ip.clone(),
            port: query_message.port as i32,
            qubit_count: query_message.qubit_count as i32,
            qubit_idle: query_message.qubit_count as i32,
            circuit_depth: query_message.circuit_depth as i32,
        },
    )
    .await
    {
        Ok(_) => {
            info!(
                "Add {}:{} physical agents added successfully",
                query_message.ip, query_message.port
            );
            (
                StatusCode::OK,
                Json(
                    json!({"Message": format!("Physical Agent {}:{} added successfully", query_message.ip, query_message.port)}),
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
pub async fn add_task(
    Form(emulate_message): Form<EmulateMessage>,
    State(state): State<SharedState>,
) -> (StatusCode, Json<Value>) {
    info!("Consume task in submit request");

    // add this task to the database
    match service::task_active::TaskActive::add_task(
        &state.write().await.db,
        entity::task_active::Model {
            id: uuid::Uuid::new_v4(),
            source: emulate_message.code.clone(),
            result: None,
            qubits: emulate_message.qubits as i32,
            depth: emulate_message.depth as i32,
            shots: emulate_message.shots as i32,
            exec_shots: 0,
            v_exec_shots: state.write().await.config.min_vexec_shots as i32,
            created_time: chrono::Utc::now().naive_utc(),
            updated_time: chrono::Utc::now().naive_utc(),
        },
    )
    .await
    {
        Ok(task) => {
            info!(
                "Task {:?} (qubits: {:?}, depth: {:?}, shots: {:?}) added successfully",
                task.id, task.qubits, task.depth, task.shots
            );
            (
                StatusCode::OK,
                Json(json!({
                    "task_id": task.id,
                    "status": "waiting"
                })),
            )
        }
        Err(err) => {
            error!("Add task failed: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}

/// Consume the task when there are some waiting tasks and available agents
pub async fn consume_task_back(db: &DbConn, waiting_task: entity::task::Model) {
    info!("Consume task in consume waiting task thread");
    let task = service::task_assignment::Qthread::submit_task_without_add(&db, waiting_task)
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
            service::task_assignment::Qthread::finish_task(
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
pub async fn submit(state: State<SharedState>, request: Request) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        // If the content type is correct, consume the task
        Some(content_type) => match content_type.to_str().unwrap() {
            "application/x-www-form-urlencoded" => {
                let Form(emulate_message) = request.extract().await.unwrap();
                add_task(Form(emulate_message), state).await
            }
            "application/json" => {
                let Json::<EmulateMessage>(emulate_message) = request.extract().await.unwrap();
                add_task(Form(emulate_message), state).await
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
    match service::task_active::TaskActive::get_task(db, task_id).await {
        Ok(task) => match task {
            Some(task) => {
                info!("Task {:?} is running", task.id);
                (
                    StatusCode::OK,
                    Json(json!({
                        "status": "found",
                        "task_id": task.id
                    })),
                )
            }
            None => match service::task::Task::get_task(db, task_id).await {
                Ok(task) => match task {
                    Some(task) => match task.status {
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
            },
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
    State(state): State<SharedState>,
    Query(query_message): Query<TaskID>,
) -> (StatusCode, Json<Value>) {
    _get_task(
        &state.write().await.db,
        uuid::Uuid::parse_str(&query_message.task_id).unwrap(),
    )
    .await
}

pub async fn get_task_with_id(
    State(state): State<SharedState>,
    Path(task_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    _get_task(&state.write().await.db, task_id).await
}
