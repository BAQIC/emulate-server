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
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct EmulateMessage {
    code: String,
    qubits: usize,
    depth: usize,
    shots: usize,
}

#[derive(Deserialize)]
pub struct TaskID {
    task_id: Uuid,
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

// merge simulation results from different shots
fn merge_and_add(v1: &mut Value, v2: &Value) {
    let v1_memory_map = v1.get_mut("Memory").unwrap().as_object_mut().unwrap();
    let v2_memory_map = v2.get("Memory").unwrap().as_object().unwrap();

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
}

// invoke the agent to run the task
pub async fn invoke_agent(
    address: &str,
    qasm: &str,
    shots: i32,
) -> Result<Response, reqwest::Error> {
    let body = [("qasm", qasm.to_string()), ("shots", shots.to_string())];

    reqwest::Client::new()
        .post(address)
        .form(&body)
        .send()
        .await
}

// add the task to the database
pub async fn add_task(
    Form(emulate_message): Form<EmulateMessage>,
    State(state): State<ServerState>,
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
pub async fn consume_task(
    db: &DbConn,
    sched_min_depth: f32,
    sched_min_gran: f32,
    task: entity::task_active::Model,
    agent: entity::physical_agent::Model,
) {
    // get exec shots according to the min depth and gran
    let mut exec_shots = (task.depth as f32 / sched_min_depth * sched_min_gran) as i32;
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
    )
    .await;

    service::physical_agent::PhysicalAgent::update_physical_agent_qubits_idle(
        db,
        agent.id,
        agent.qubit_idle + task.qubits as i32,
    )
    .await
    .unwrap();

    if result.is_ok() {
        // if the task is run for the first time
        if task.result.is_none() {
            service::task_active::TaskActive::update_task_result(
                db,
                task.id,
                task.exec_shots + exec_shots,
                task.v_exec_shots + exec_shots,
                Some(
                    serde_json::to_string_pretty(&result.unwrap().json::<Value>().await.unwrap())
                        .unwrap(),
                ),
                sea_orm_active_enums::TaskActiveStatus::Waiting,
            )
            .await
            .unwrap();
        } else {
            let mut task_result = serde_json::from_str::<Value>(&task.result.unwrap()).unwrap();
            merge_and_add(
                &mut task_result,
                &result.unwrap().json::<Value>().await.unwrap(),
            );

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
        }
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
                status: sea_orm_active_enums::TaskStatus::Failed,
                created_time: task.created_time,
                updated_time: task.updated_time,
            },
        )
        .await
        .unwrap();
    }
}

/// Submit the task to the server
pub async fn submit(state: State<ServerState>, request: Request) -> (StatusCode, Json<Value>) {
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

/// Get the task status by task id
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
                        "result": serde_json::from_str::<Value>(&task.result.unwrap()).unwrap(),
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
                                    &serde_json::from_str::<Value>(&task.result).unwrap(),
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
                                    &serde_json::from_str::<Value>(&task.result).unwrap(),
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
    State(state): State<ServerState>,
    // query only support following format, Query<Uuid> is wrong
    Query(query_message): Query<TaskID>,
) -> (StatusCode, Json<Value>) {
    _get_task(&state.db, query_message.task_id).await
}

/// Get the task status by task id
pub async fn get_task_with_id(
    State(state): State<ServerState>,
    Path(task_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    _get_task(&state.db, task_id).await
}
