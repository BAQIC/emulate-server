//! The module that contains the physical agent router. The physical agent
//! router is used to add, get, and update the physical agent information.

use crate::entity;
use crate::entity::sea_orm_active_enums;
use crate::service;
use axum::extract::{Query, Request};
use axum::{extract::State, http::StatusCode, Json};
use axum::{Form, RequestExt};
use dns_lookup::lookup_host;
use http::header;
use log::{error, info};
use sea_orm::DbConn;
use serde_json::{json, Value};

use super::physical_agent_utils::{AgentAddress, AgentInfo, AgentInfoUpdate, AgentStatus, Agents};
use super::ServerState;

/// ## Get Agent Info
/// Get the agent information from the given path and return the Agents struct.
/// If the file does not exist, return an empty Agents struct.
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

/// Internal function to add a physical agent to the database
async fn _add_physical_agent(
    state: ServerState,
    Form(mut query_message): Form<AgentInfo>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Init physical agent (qubits: {}, circuit_depth: {}) with {}:{:?} (hostname: {:?})",
        query_message.qubit_count,
        query_message.circuit_depth,
        query_message.ip,
        query_message.port,
        query_message.hostname
    );
    let db = &state.db;

    // if the ip is empty, use the hostname to get the ip address
    if query_message.ip == "" {
        info!("Using hostname to get the ip address");
        // not check hostname is none
        match lookup_host(&query_message.hostname.unwrap()) {
            Ok(ips) => {
                if ips.len() == 0 {
                    error!("Get ip address failed: No ip address found");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"Error": "No ip address found"})),
                    );
                }
                info!("Get ip address successfully: {:?}", ips);
                let ip = ips[0].to_string();
                query_message.ip = ip;
            }
            Err(err) => {
                error!("Get ip address failed: {}", err);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"Error": format!("{}", err)})),
                );
            }
        }
    }

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

/// ## Add Physical Agent
/// Add a physical agent to the database. The agent information is passed in the
/// request body. The request body can be either JSON or form-urlencoded. If the
/// agent ip and port is the same as the existing agent, the post request will
/// return an error.
///
/// If the ip is empty, use the hostname to get the ip address.
pub async fn add_physical_agent(
    State(state): State<ServerState>,
    request: Request,
) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) => match content_type.to_str().unwrap() {
            "application/json" => {
                let Json::<AgentInfo>(message) = request.extract().await.unwrap();
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

/// ## Add Physical Agent From File
/// Add physical agents to the database from the given file. The file should
/// contain the agent information in JSON format. This function is used by the
/// consume task thread.
///
/// If the ip is empty, use the hostname to get the ip address.
pub async fn add_physical_agent_from_file(db: &DbConn, agents: Agents) {
    for mut agent in agents.agents {
        // if the ip is empty, use the hostname to get the ip address
        if agent.ip == "" {
            info!("Using hostname to get the ip address");
            // not check hostname is none
            match lookup_host(&agent.hostname.unwrap()) {
                Ok(ips) => {
                    if ips.len() == 0 {
                        error!("Get ip address failed: No ip address found");
                        continue;
                    }
                    info!("Get ip address successfully: {:?}", ips);
                    let ip = ips[0].to_string();
                    agent.ip = ip;
                }
                Err(err) => {
                    error!("Get ip address failed: {}", err);
                    continue;
                }
            }
        }
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

/// ## Get Physical Agent By Address
/// Get the physical agent by the given ip and port. If the port is not
/// provided, return all the agents with the given ip. If the ip is empty, use
/// the hostname to get the ip address.
pub async fn get_physical_agent_by_address(
    State(state): State<ServerState>,
    Query(mut query_message): Query<AgentAddress>,
) -> (StatusCode, Json<Value>) {
    info!("Get physical agent by address: {:?}", query_message);

    let db = &state.db;

    if query_message.ip == "" {
        info!("Using hostname to get the ip address");
        // not check hostname is none
        match lookup_host(&query_message.hostname.unwrap()) {
            Ok(ips) => {
                if ips.len() == 0 {
                    error!("Get ip address failed: No ip address found");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"Error": "No ip address found"})),
                    );
                }
                info!("Get ip address successfully: {:?}", ips);
                let ip = ips[0].to_string();
                query_message.ip = ip;
            }
            Err(err) => {
                error!("Get ip address failed: {}", err);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"Error": format!("{}", err)})),
                );
            }
        }
    }

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

/// Internal function to update the physical agent with the given id
async fn _update_physical_agent(
    state: ServerState,
    Form(query_message): Form<AgentInfoUpdate>,
) -> (StatusCode, Json<Value>) {
    info!(
        "Update physical agent {:?} with address {:?}:{:?}, qubit_count {:?}, circuit_depth {:?}, status {:?}",
        query_message.id, query_message.ip, query_message.port, query_message.qubit_count, query_message.circuit_depth, query_message.status
    );

    let db = &state.db;

    match service::physical_agent::PhysicalAgent::update_physical_agent_status(
        db,
        query_message.id,
        sea_orm_active_enums::PhysicalAgentStatus::Down,
    )
    .await
    {
        Ok((_, ori_status)) => {
            info!("Update physical agent status to down before update agent information");
            while !service::physical_agent::PhysicalAgent::check_physical_agent_idle(
                db,
                query_message.id,
            )
            .await
            .unwrap()
            {
                info!(
                    "Waiting for the physical agent {} to be idle",
                    query_message.id
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            // update the agent information after the agent is idle
            match service::physical_agent::PhysicalAgent::update_physical_agent(
                db,
                query_message.id,
                query_message.ip,
                query_message.port.map(|x| x as i32),
                query_message.qubit_count.map(|x| x as i32),
                query_message.circuit_depth.map(|x| x as i32),
                query_message.status.clone().map(|x| match x {
                    AgentStatus::Running => sea_orm_active_enums::PhysicalAgentStatus::Running,
                    AgentStatus::Down => sea_orm_active_enums::PhysicalAgentStatus::Down,
                }),
            )
            .await
            {
                Ok(agent) => {
                    info!("Update physical agent successfully");

                    // if we do not update the agent information by this api, we restore the agent
                    // status to the original status
                    if query_message.status.is_none() {
                        service::physical_agent::PhysicalAgent::update_physical_agent_status(
                            db,
                            query_message.id,
                            ori_status,
                        )
                        .await
                        .unwrap();
                    }

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
        Err(err) => {
            error!("Update physical agent status to down failed: {}", err);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            );
        }
    }
}

/// ## Update Physical Agent
/// Update the physical agent with the given id. The agent information is passed
/// in the request body. The request body can be either JSON or form-urlencoded.
/// Except for the ID, all other fields are optional. Please refer to the
/// [AgentInfoUpdate] struct for more information.
pub async fn update_physical_agent(
    State(state): State<ServerState>,
    request: Request,
) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) => match content_type.to_str().unwrap() {
            "application/json" => {
                let Json::<AgentInfoUpdate>(message) = request.extract().await.unwrap();
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

/// ## Remove Physical Agent
/// Remove the physical agent with the given id.
pub async fn remove_physical_agent(
    State(state): State<ServerState>,
    Query(query_message): Query<AgentInfoUpdate>,
) -> (StatusCode, Json<Value>) {
    info!("Remove physical agent: {:?}", query_message.id);

    let db = &state.db;

    match service::physical_agent::PhysicalAgent::remove_physical_agent(db, query_message.id).await
    {
        Ok(agent) => {
            info!("Remove physical agent {:?} successfully", agent);
            (StatusCode::OK, Json(json!({"agent": agent})))
        }
        Err(err) => {
            error!("Remove physical agent failed: {}", err);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"Error": format!("{}", err)})),
            )
        }
    }
}
