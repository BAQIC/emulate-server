//! The module that contains some struct definitions for the physical agent
//! router. It is used to deserialize the post request body from the user.
//! - `AgentStatus`: The enum that represents the status of the agent. It can be
//!  either `running` or `down`.
//! - `AgentInfo`: The struct that represents the information of the agent. The
//! user can add a new agent with the given information.
//! - `AgentInfoUpdate`: The struct that represents the information of the agent
//! that the user wants to update. The user can update the agent with the given
//! information.
//! - `AgentAddress`: The struct that represents the address of the agent. The
//! user can get the agent information by the address.
//! - `Agents`: The struct that represents the list of agents. This struct is
//! used to deserialize the agents from the file. This struct is used to add the
//! agents for consume task thread at the beginning.
//! - `empty_string_as_none`: The function that converts an empty string to
//! `None` when deserializing the optional field.

use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

/// ## Agent Status
/// The struct that represents the status of the agent. It can be either
/// `running` or `down`. This struct is used for the user to update the status
/// of the agent.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AgentStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "down")]
    Down,
}

/// For the `AgentStatus` enum, we need to implement the `FromStr` trait to
/// convert the string to the enum. This is used when the user sends the status
/// in the form of a string.
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

/// ## Agent Info
/// The struct that represents the information of the agent. The user can add a
/// new agent with the given information.
/// - `ip`: The IP address of the agent.
/// - `hostname`: The host name of the agent, optional.
/// - `port`: The port number of the agent.
/// - `qubit_count`: The number of qubits the agent has.
/// - `circuit_depth`: The circuit depth of the agent can run.
#[derive(Deserialize, Debug)]
pub struct AgentInfo {
    pub ip: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub hostname: Option<String>,
    pub port: u32,
    pub qubit_count: u32,
    pub circuit_depth: u32,
}

/// ## Agent Info Update
/// The struct that represents the information of the agent that the user wants
/// to update. The user can update the agent with the given information.
/// - `id`: The ID of the agent.
/// - `ip`: The IP address of the agent, optional.
/// - `port`: The port number of the agent, optional.
/// - `qubit_count`: The number of qubits the agent has, optional.
/// - `circuit_depth`: The circuit depth of the agent can run, optional.
/// - `status`: The status of the agent, optional. The `status` field is an enum
///   of `AgentStatus` which can be either `running` or `down`.
#[derive(Deserialize, Debug, Clone)]
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

/// ## Agent Address
/// The struct that represents the address of the agent. The user can get the
/// agent information by the address.
/// - `ip`: The IP address of the agent.
/// - `port`: The port number of the agent, optional.
#[derive(Deserialize, Debug)]
pub struct AgentAddress {
    pub ip: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub hostname: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub port: Option<u32>,
}

/// The function that converts an empty string to `None` when deserializing the
/// optional field.
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

/// ## Agents
/// The struct that represents the list of agents. This struct is used to
/// deserialize the agents from the file. This struct is used to add the agents
/// for consume task thread at the beginning.
#[derive(Deserialize, Debug)]
pub struct Agents {
    pub agents: Vec<AgentInfo>,
}
