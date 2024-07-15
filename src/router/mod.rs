use sea_orm::DbConn;

pub mod physical_agent;
pub mod task;

#[derive(Clone)]
pub struct ServerState {
    pub db: DbConn,
    pub config: super::config::QSchedulerConfig,
}

