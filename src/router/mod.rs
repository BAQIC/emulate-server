use sea_orm::DbConn;

pub mod physical_agent;
pub mod physical_agent_utils;
pub mod task;

/// ## Server State
/// The server state is a struct that holds the database connection and the
/// configuration. This struct is used to pass the database connection and the
/// configuration to the handlers.
#[derive(Clone)]
pub struct ServerState {
    pub db: DbConn,
    pub config: super::config::QSchedulerConfig,
}
