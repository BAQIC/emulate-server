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
use uuid::Uuid;

use super::ServerState;

#[derive(Serialize, Deserialize, Debug)]
pub enum AgentStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "down")]
    Down,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseAgentStatusError;

impl fmt::Display for ParseAgentStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid agent status")
    }
}

impl FromStr for AgentStatus {
    type Err = ParseAgentStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(AgentStatus::Running),
            "down" => Ok(AgentStatus::Down),
            _ => Err(ParseAgentStatusError),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AgentInfo {
    pub ip: String,
    pub port: u32,
    pub qubit_count: u32,
    pub circuit_depth: u32,
}

#[derive(Deserialize, Debug)]
pub struct AgentInfoUpdate {
    pub id: Uuid,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub ip: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub port: Option<u32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub qubit_count: Option<u32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub circuit_depth: Option<u32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub status: Option<AgentStatus>,
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

/// Update the physical agent with the given id
/// TODO: update the physical agent after tasks are done
pub async fn update_physical_agent(
    State(state): State<ServerState>,
    Query(query_message): Query<AgentInfoUpdate>,
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
