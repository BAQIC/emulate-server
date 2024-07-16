use crate::entity;
use crate::entity::sea_orm_active_enums;
use crate::service;
use axum::extract::{Query, Request};
use axum::{extract::State, http::StatusCode, Json};
use axum::{Form, RequestExt};
use http::header;
use log::{error, info};
use sea_orm::DbConn;
use serde_json::{json, Value};

use super::physical_agent_utils::{AgentAddress, AgentInfo, AgentInfoUpdate, AgentStatus, Agents};
use super::ServerState;

pub fn get_agent_info(path: &str) -> Agents {
    if !std::path::Path::new(path).exists() {
        error!("Agents info file not found");
        Agents { agents: vec![] }
    } else {
        let agent_info = std::fs::read_to_string(path).unwrap();
        let agent_info: Agents = serde_json::from_str(&agent_info).unwrap();
        agent_info
    }
}

/// Initialize the qthread with num of physical agents
async fn _add_physical_agent(
    state: ServerState,
    Form(query_message): Form<AgentInfo>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Init physical agent (qubits: {}, circuit_depth: {}) with {}:{:?}",
        query_message.qubit_count,
        query_message.circuit_depth,
        query_message.ip,
        query_message.port
    );
    let db = &state.db;

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
        Ok(agent) => {
            info!(
                "Add {}:{} physical agents added successfully",
                query_message.ip, query_message.port
            );
            (StatusCode::OK, Json(json!({"agent": agent})))
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

pub async fn add_physical_agent(
    State(state): State<ServerState>,
    request: Request,
) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) => match content_type.to_str().unwrap() {
            "application/json" => {
                let Form(message) = request.extract().await.unwrap();
                _add_physical_agent(state, Form(message)).await
            }
            "application/x-www-form-urlencoded" => {
                let Form(message) = request.extract().await.unwrap();
                _add_physical_agent(state, Form(message)).await
            }
            _ => {
                error!("Add physical agents failed: Invalid content type");
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"Error": "Invalid content type"})),
                )
            }
        },
        None => {
            error!("Add physical agents failed: No content type");
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": "No content type"})),
            )
        }
    }
}

pub async fn add_physical_agent_from_file(db: &DbConn, agents: Agents) {
    for agent in agents.agents {
        match service::physical_agent::PhysicalAgent::add_physical_agent(
            db,
            entity::physical_agent::Model {
                id: uuid::Uuid::new_v4(),
                status: sea_orm_active_enums::PhysicalAgentStatus::Running,
                ip: agent.ip,
                port: agent.port as i32,
                qubit_count: agent.qubit_count as i32,
                qubit_idle: agent.qubit_count as i32,
                circuit_depth: agent.circuit_depth as i32,
            },
        )
        .await
        {
            Ok(a) => {
                info!(
                    "Add {}:{} physical agent with qubit_count: {}, circuit_depth: {}",
                    a.ip, a.port, a.qubit_count, a.circuit_depth
                );
            }
            Err(err) => match err {
                sea_orm::DbErr::Custom(e) => {
                    error!("Add physical agent failed: {}", e);
                }
                _ => {
                    error!("Add physical agent failed: {}", err);
                }
            },
        }
    }
}

pub async fn get_physical_agent_by_address(
    State(state): State<ServerState>,
    Query(query_message): Query<AgentAddress>,
) -> (StatusCode, Json<Value>) {
    info!("Get physical agent by address: {:?}", query_message);

    let db = &state.db;
    if query_message.port.is_none() {
        match service::physical_agent::PhysicalAgent::get_physical_agent_by_ip(
            db,
            query_message.ip.clone(),
        )
        .await
        {
            Ok(agents) => match agents.len() {
                0 => {
                    error!("Get physical agent by address failed: No agent found");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"Error": "No agent found"})),
                    );
                }
                _ => {
                    info!("Get physical agent by address successfully");
                    (
                        StatusCode::OK,
                        Json(json!({
                            "agents": agents,
                        })),
                    )
                }
            },
            Err(err) => {
                error!("Get physical agent by address failed: {}", err);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"Error": format!("{}", err)})),
                )
            }
        }
    } else {
        match service::physical_agent::PhysicalAgent::get_physical_agent_by_address(
            db,
            query_message.ip.clone(),
            query_message.port.unwrap() as i32,
        )
        .await
        {
            Ok(agent) => match agent {
                Some(agent) => {
                    info!("Get physical agent by address successfully");
                    (
                        StatusCode::OK,
                        Json(json!({
                            "agent": agent,
                        })),
                    )
                }
                None => {
                    error!("Get physical agent by address failed: No agent found");
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"Error": "No agent found"})),
                    )
                }
            },
            Err(err) => {
                error!("Get physical agent by address failed: {}", err);
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"Error": format!("{}", err)})),
                )
            }
        }
    }
}

/// Update the physical agent with the given id
/// TODO: update the physical agent after tasks are done
async fn _update_physical_agent(
    state: ServerState,
    Form(query_message): Form<AgentInfoUpdate>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Update physical agent {:?} with address {:?}:{:?}, qubit_count {:?}, circuit_depth {:?}, status {:?}",
        query_message.id, query_message.ip, query_message.port, query_message.qubit_count, query_message.circuit_depth, query_message.status
    );

    let db = &state.db;

    match service::physical_agent::PhysicalAgent::update_physical_agent(
        db,
        query_message.id,
        query_message.ip,
        query_message.port.map(|x| x as i32),
        query_message.qubit_count.map(|x| x as i32),
        query_message.circuit_depth.map(|x| x as i32),
        query_message.status.map(|x| match x {
            AgentStatus::Running => sea_orm_active_enums::PhysicalAgentStatus::Running,
            AgentStatus::Down => sea_orm_active_enums::PhysicalAgentStatus::Down,
        }),
    )
    .await
    {
        Ok(agent) => {
            info!("Update physical agent successfully");
            (
                StatusCode::OK,
                Json(json!({
                    "agent": agent,
                })),
            )
        }
        Err(err) => {
            error!("Update physical agent failed: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}

pub async fn update_physical_agent(
    State(state): State<ServerState>,
    request: Request,
) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) => match content_type.to_str().unwrap() {
            "application/json" => {
                let Form(message) = request.extract().await.unwrap();
                _update_physical_agent(state, Form(message)).await
            }
            "application/x-www-form-urlencoded" => {
                let Form(message) = request.extract().await.unwrap();
                _update_physical_agent(state, Form(message)).await
            }
            _ => {
                error!("Update physical agent failed: Invalid content type");
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"Error": "Invalid content type"})),
                )
            }
        },
        None => {
            error!("Update physical agent failed: No content type");
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": "No content type"})),
            )
        }
    }
}
