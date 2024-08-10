//! The modult that contains the task router. The task router is responsible for
//! handling the task submit and get task status requests. The task submit
//! request is used to submit a task to the scheduler. The get task status
//! request is used to get the task status by task id.

use super::ServerState;
use crate::entity;
use crate::entity::sea_orm_active_enums;
use crate::service;
use axum::{
    extract::{Path, Query, Request, State},
    http::{header, StatusCode},
    Form, Json, RequestExt,
};
use log::{error, info};
use reqwest::Response;
use sea_orm::DbConn;
use serde::Deserialize;
use serde_json::{json, map::Entry, Value};
use std::fmt;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub enum TaskMode {
    #[serde(rename = "sequence")]
    Sequence,
    #[serde(rename = "aggregation")]
    Aggregation,
    #[serde(rename = "max")]
    Max,
    #[serde(rename = "min")]
    Min,
}

impl fmt::Display for TaskMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskMode::Sequence => write!(f, "sequence"),
            TaskMode::Aggregation => write!(f, "aggregation"),
            TaskMode::Max => write!(f, "max"),
            TaskMode::Min => write!(f, "min"),
        }
    }
}

/// ## Emulate message
/// The emulate message is used to submit a task to the server. That means the
/// user can submit a task to the server by sending a POST request with the
/// emulate message.
/// - `code`: The code is the quantum assembly code that the user wants to run.
/// - `qubits`: The number of qubits that the user wants to run.
/// - `depth`: The depth of the circuit that the user wants to run.
/// - `shots`: The number of shots that the user wants to run.
#[derive(Deserialize, Debug)]
pub struct EmulateMessage {
    code: String,
    qubits: usize,
    depth: usize,
    shots: usize,
    mode: Option<TaskMode>,
}

/// ## Task ID
/// The task ID is used to get the task status by task id. The user can get the
/// task status by sending a GET request with the task id.
/// - `task_id`: The task id that the user wants to get the status of.
#[derive(Deserialize)]
pub struct TaskID {
    task_id: Uuid,
}

/// ## Merge simulation results
/// Merge the previous result with the new result. This function will merge the
/// content of `Memory` field in the result. For the same key (same state), the
/// values will be added together. If the key does not exist in the previous
/// result, it will be added to the previous result.
fn merge_and_add(v1: &mut Value, v2: &Value) {
    let v1_memory = v1.get_mut("Memory").unwrap();
    let v2_memory = v2.get("Memory").unwrap();

    if v1_memory.is_object() && v2_memory.is_object() {
        let v1_memory_map = v1_memory.as_object_mut().unwrap();
        let v2_memory_map = v2_memory.as_object().unwrap();

        for (k, v2_value) in v2_memory_map {
            match v1_memory_map.entry(k.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(v2_value.clone());
                }
                Entry::Occupied(mut entry) => {
                    let v1_value = entry.get_mut();
                    if let (Some(v1_num), Some(v2_num)) = (v1_value.as_i64(), v2_value.as_i64()) {
                        *v1_value = Value::from(v1_num + v2_num);
                    }
                }
            }
        }
    } else if v1_memory.is_array() && v2_memory.is_array() {
        let v1_memory_array = v1_memory.as_array_mut().unwrap();
        let v2_memory_array = v2_memory.as_array().unwrap();

        v1_memory_array.extend(v2_memory_array.iter().cloned());
    } else {
        error!("Merge simulation results failed: Memory field is not object or array");
        panic!("Merge simulation results failed: Memory field is not object or array");
    }
}

/// ## Invoke the agent
/// According to the agent address, invoke the agent's submit API with the
/// `qasm` and `shots` parameters. The agent will run the task and return the
/// result. The result is like:
/// ```json
/// {
///    "Memory": {
///       "00": 1000,
///       "01": 1000,
///       "10": 1000,
///       "11": 1000
/// }
async fn invoke_agent(
    address: &str,
    qasm: &str,
    shots: i32,
    mode: Option<String>,
) -> Result<Response, reqwest::Error> {
    let mut body: Vec<(&str, String)> =
        vec![("qasm", qasm.to_string()), ("shots", shots.to_string())];
    if mode.is_some() {
        body.push(("mode", mode.unwrap()));
    }

    reqwest::Client::new()
        .post(address)
        .form(&body)
        .send()
        .await
}

/// Internal task submit function
async fn _submit(
    state: ServerState,
    Form(emulate_message): Form<EmulateMessage>,
) -> (StatusCode, Json<Value>) {
    info!("Consume task in submit request");

    let min_vexec_shots = service::task_active::TaskActive::get_min_vexec_shots(&state.db)
        .await
        .unwrap();

    // add this task to the database
    match service::task_active::TaskActive::add_task(
        &state.db,
        entity::task_active::Model {
            id: uuid::Uuid::new_v4(),
            source: emulate_message.code.clone(),
            result: None,
            qubits: emulate_message.qubits as i32,
            depth: emulate_message.depth as i32,
            shots: emulate_message.shots as i32,
            exec_shots: 0,
            v_exec_shots: min_vexec_shots as i32,
            mode: emulate_message.mode.map(|m| match m {
                TaskMode::Sequence => sea_orm_active_enums::TaskMode::Sequence,
                TaskMode::Aggregation => sea_orm_active_enums::TaskMode::Aggregation,
                TaskMode::Max => sea_orm_active_enums::TaskMode::Max,
                TaskMode::Min => sea_orm_active_enums::TaskMode::Min,
            }),
            status: sea_orm_active_enums::TaskActiveStatus::Waiting,
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
            (StatusCode::OK, Json(json!({"task": task})))
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

/// ## Consume Task
/// The consume task function is responsible for submitting the task to the
///  agent. The main steps are:
/// - Calculate the number of shots to run the task according to the task's
/// depth and scheduler configuration. The formula is:
///  `shots = task.depth / sched_min_depth * sched_min_gran`. That is, the
/// deeper the circuit, the less shots will be executed in one run.
/// - Add the [assignment](crate::entity::task_assignment::Model) to the
///   database.
/// - Submit the task to the agent by [invoking](invoke_agent) the agent's
///   submit API.
/// - Update the agent's qubit_idle field in the database.
/// - Depending on the result of the task, update the task's result and status
/// in the database.
///   - If the task is run for the first time, update the task's result and
///     status to Waiting.
///   - If the task is finished, remove the task from the active task list and
///     add it to the task list.
///   - If the task is not finished, [merge](merge_and_add) previous result with
///     the new result and update the task's result and status to Waiting.
///   - If the invocation fails, remove the task from the active task list and
///     add it to the task list with the error message.
pub async fn consume_task(
    db: &DbConn,
    sched_min_depth: f32,
    sched_min_gran: f32,
    task: entity::task_active::Model,
    agent: entity::physical_agent::Model,
) {
    // get exec shots according to the min depth and gran
    let mut exec_shots = (sched_min_depth / task.depth as f32 * sched_min_gran) as i32;
    if task.exec_shots + exec_shots > task.shots {
        exec_shots = task.shots - task.exec_shots;
    }

    info!("Consume task {:?} with {:?} shots", task.id, exec_shots);

    // init add assignment
    let assign = service::task_assignment::TaskAssignment::add_assignment(
        db,
        entity::task_assignment::Model {
            id: uuid::Uuid::new_v4(),
            task_id: task.id,
            agent_id: agent.id,
            shots: Some(exec_shots),
            status: sea_orm_active_enums::AssignmentStatus::Running,
        },
    )
    .await
    .unwrap();

    // run the task
    let result = invoke_agent(
        &format!("http://{}:{}/submit", agent.ip, agent.port),
        &task.source,
        exec_shots,
        task.mode.clone().map(|m| match m {
            sea_orm_active_enums::TaskMode::Sequence => TaskMode::Sequence.to_string(),
            sea_orm_active_enums::TaskMode::Aggregation => TaskMode::Aggregation.to_string(),
            sea_orm_active_enums::TaskMode::Max => TaskMode::Max.to_string(),
            sea_orm_active_enums::TaskMode::Min => TaskMode::Min.to_string(),
        }),
    )
    .await;

    service::physical_agent::PhysicalAgent::update_physical_agent_qubits_idle(
        db,
        agent.id,
        task.qubits as i32,
    )
    .await
    .unwrap();

    if result.is_ok() {
        let task_result = if task.result.is_none() {
            // if the task is run for the first time
            result.unwrap().json::<Value>().await.unwrap()
        } else {
            let mut task_result: Value =
                serde_json::from_str::<Value>(&task.result.unwrap()).unwrap();
            merge_and_add(
                &mut task_result,
                &result.unwrap().json::<Value>().await.unwrap(),
            );
            task_result
        };

        // if the task is finisched
        if task.exec_shots + exec_shots >= task.shots {
            service::task_active::TaskActive::remove_active_task(db, task.id)
                .await
                .unwrap();
            service::task::Task::add_task(
                db,
                entity::task::Model {
                    id: task.id,
                    source: task.source,
                    result: serde_json::to_string_pretty(&task_result).unwrap(),
                    qubits: task.qubits,
                    depth: task.depth,
                    shots: task.shots,
                    mode: task.mode,
                    status: sea_orm_active_enums::TaskStatus::Succeeded,
                    created_time: task.created_time,
                    updated_time: task.updated_time,
                },
            )
            .await
            .unwrap();
        } else {
            // if the task is not finisched
            service::task_active::TaskActive::update_task_result(
                db,
                task.id,
                task.exec_shots + exec_shots,
                task.v_exec_shots + exec_shots,
                Some(serde_json::to_string_pretty(&task_result).unwrap()),
                sea_orm_active_enums::TaskActiveStatus::Waiting,
            )
            .await
            .unwrap();
        }

        // update the assignment status
        service::task_assignment::TaskAssignment::update_assignment_status(
            db,
            assign.id,
            sea_orm_active_enums::AssignmentStatus::Succeeded,
        )
        .await
        .unwrap();
    } else {
        // if the task is failed
        service::task_assignment::TaskAssignment::update_assignment_status(
            db,
            assign.id,
            sea_orm_active_enums::AssignmentStatus::Failed,
        )
        .await
        .unwrap();

        // remove the task from the active task list, and add it to the task list
        let task = service::task_active::TaskActive::remove_active_task(db, task.id)
            .await
            .unwrap();

        service::task::Task::add_task(
            db,
            entity::task::Model {
                id: task.id,
                source: task.source,
                result: serde_json::to_string_pretty(
                    &json!({"Error": format!("{}", result.unwrap_err())}),
                )
                .unwrap(),
                qubits: task.qubits,
                depth: task.depth,
                shots: task.shots,
                mode: task.mode,
                status: sea_orm_active_enums::TaskStatus::Failed,
                created_time: task.created_time,
                updated_time: task.updated_time,
            },
        )
        .await
        .unwrap();
    }
}

/// ## Submit task
/// Add the task to the [task_active](crate::entity::task_active::Model) table
/// if the qubits and depth are less than the agent's qubit_count and circuit
/// depth. Then, retrieve the virtual execution shots from the task_active
/// table. If these conditions are not met, return an error message.
pub async fn submit(
    State(state): State<ServerState>,
    request: Request,
) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        // If the content type is correct, consume the task
        Some(content_type) => match content_type.to_str().unwrap() {
            "application/x-www-form-urlencoded" => {
                let Form(emulate_message) = request.extract().await.unwrap();
                _submit(state, Form(emulate_message)).await
            }
            "application/json" => {
                let Json::<EmulateMessage>(emulate_message) = request.extract().await.unwrap();
                _submit(state, Form(emulate_message)).await
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

/// Internal get task function
async fn _get_task(db: &DbConn, task_id: Uuid) -> (StatusCode, Json<Value>) {
    info!("Get task status by task id: {:?}", task_id);
    let db = db;
    match service::task_active::TaskActive::get_task(db, task_id).await {
        Ok(task) => match task {
            Some(task) => {
                info!("Task {:?} is running", task.id);
                (StatusCode::OK, Json(json!({"task": task})))
            }
            None => match service::task::Task::get_task(db, task_id).await {
                Ok(task) => match task {
                    Some(task) => match task.status {
                        sea_orm_active_enums::TaskStatus::Failed => {
                            info!("Task {:?} is failed", task.id);
                            (StatusCode::OK, Json(json!({"task": task})))
                        }
                        sea_orm_active_enums::TaskStatus::Succeeded => {
                            info!("Task {:?} is succeeded", task.id);
                            (StatusCode::OK, Json(json!({"task": task})))
                        }
                    },
                    None => {
                        info!("Task with id {:?} not found", task_id);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(json!({
                                "task_id": task_id,
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
                    "Error": format!("{}", err)
                })),
            )
        }
    }
}

/// ## Get task by query parameter
/// Get the task status by task id. First, check if the task is in the
/// [task_active](crate::entity::task_active::Model) table. If the task is
/// running/waiting, return the task status. If the task is not in the
/// task_active table, check if the task is in the
/// [task](crate::entity::task::Model) table. If the task is Failed/Succeeded,
/// return the task status. If the task is not in the task table, return an
/// error message.
pub async fn get_task(
    State(state): State<ServerState>,
    // query only support following format, Query<Uuid> is wrong
    Query(query_message): Query<TaskID>,
) -> (StatusCode, Json<Value>) {
    _get_task(&state.db, query_message.task_id).await
}

/// ## Get task by url path
/// Please ref to the [get_task] function
pub async fn get_task_with_id(
    State(state): State<ServerState>,
    Path(task_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    _get_task(&state.db, task_id).await
}
