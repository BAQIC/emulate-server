use crate::entity;
use crate::entity::sea_orm_active_enums;
use crate::service;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, Json};
use log::{error, info};
use sea_orm::DbConn;
use serde::{de, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fmt, str::FromStr};

use super::ServerState;

#[derive(Serialize, Deserialize, Debug)]
pub enum AgentStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "down")]
    Down,
}

#[derive(Deserialize, Debug)]
pub struct AgentInfo {
    pub ip: String,
    pub port: u32,
    pub qubit_count: u32,
    pub circuit_depth: u32,
}

#[derive(Deserialize, Debug)]
pub struct Agents {
    pub agents: Vec<AgentInfo>,
}

#[derive(Deserialize, Debug)]
pub struct AgentAddress {
    pub ip: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub port: Option<u32>,
}

fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

pub fn get_agent_info(path: &str) -> Agents {
    let agent_info = std::fs::read_to_string(path).unwrap();
    let agent_info: Agents = serde_json::from_str(&agent_info).unwrap();
    agent_info
}

/// Initialize the qthread with num of physical agents
pub async fn add_physical_agent(
    State(state): State<ServerState>,
    Query(query_message): Query<AgentInfo>,
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

pub async fn add_physical_agent_from_file(db: &DbConn, agents: Agents) {
    for agent in agents.agents {
        info!(
            "Init physical agent (qubits: {}, circuit_depth: {}) with {}:{:?}",
            agent.qubit_count, agent.circuit_depth, agent.ip, agent.port
        );
        service::physical_agent::PhysicalAgent::add_physical_agent(
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
        .unwrap();
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

pub async fn update_physical_agent_status(
    State(state): State<ServerState>,
    Query(query_message): Query<(uuid::Uuid, AgentStatus)>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Update physical agent {:?} status: {:?}",
        query_message.0, query_message.1
    );

    let db = &state.db;

    match service::physical_agent::PhysicalAgent::update_physical_agent_status(
        db,
        query_message.0,
        match query_message.1 {
            AgentStatus::Running => sea_orm_active_enums::PhysicalAgentStatus::Running,
            AgentStatus::Down => sea_orm_active_enums::PhysicalAgentStatus::Down,
        },
    )
    .await
    {
        Ok(_) => {
            info!("Update physical agent status successfully");
            (
                StatusCode::OK,
                Json(json!({"Message": "Update physical agent status successfully"})),
            )
        }
        Err(err) => {
            error!("Update physical agent status failed: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}

pub async fn update_physical_agent(
    State(state): State<ServerState>,
    Query(query_message): Query<(uuid::Uuid, AgentInfo)>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Update physical agent {:?} with {:?}",
        query_message.0, query_message.1
    );

    let db = &state.db;

    match service::physical_agent::PhysicalAgent::update_physical_agent(
        db,
        query_message.0,
        entity::physical_agent::Model {
            id: query_message.0,
            status: sea_orm_active_enums::PhysicalAgentStatus::Running,
            ip: query_message.1.ip.clone(),
            port: query_message.1.port as i32,
            qubit_count: query_message.1.qubit_count as i32,
            qubit_idle: query_message.1.qubit_count as i32,
            circuit_depth: query_message.1.circuit_depth as i32,
        },
    )
    .await
    {
        Ok(_) => {
            info!("Update physical agent successfully");
            (
                StatusCode::OK,
                Json(json!({"Message": "Update physical agent successfully"})),
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
