use axum::{extract::State, Json};
use http::StatusCode;
use log::{error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::DbConn;
use serde_json::{json, Value};

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

pub async fn fresh_db(State(state): State<ServerState>) -> (StatusCode, Json<Value>) {
    match Migrator::fresh(&state.db).await {
        Ok(_) => {
            info!("fresh database success: drop all tables from the database, then reapply all migrations.");
            (
                StatusCode::OK,
                Json(
                    json!({ "result": "drop all tables from the database, then reapply all migrations." }),
                ),
            )
        }
        Err(e) => {
            error!("fresh database error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        }
    }
}
