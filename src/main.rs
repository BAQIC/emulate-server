#![allow(deprecated)]
use axum::{routing, Router};
use log::info;
use migration::{Migrator, MigratorTrait};
use router::consume_task;
pub use sea_orm::{ConnectOptions, Database, DbConn};
use std::sync::Arc;
use tokio::sync::RwLock;
pub mod config;
pub mod entity;
pub mod router;
pub mod service;
pub mod task;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let schd_conf = config::QSchedulerConfig::default();
    let schd_conf_cons = schd_conf.clone();
    let schd_conf_axum = schd_conf.clone();

    // Start a thread to consume waiting tasks, and submit them to idle agents
    std::thread::spawn(move || {
        info!("Consume waiting task thread started");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // connect to the database
            if std::path::Path::new(".env").exists() {
                info!("Load .env file from current directory");
                dotenv::from_filename(".env").ok();
            }
            let base_url =
                std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

            info!("Consume waiting task thread connect database: {}", base_url);

            // disable sqlx logging
            let mut connection_options = ConnectOptions::new(base_url);
            connection_options.sqlx_logging(false);

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let db: DbConn = Database::connect(connection_options).await.unwrap();

            loop {
                let waiting_tasks = service::task_active::TaskActive::get_asc_tasks(&db)
                    .await
                    .unwrap();
                
                // todo: if the device is idle, run one task concurrently
                for waiting_task in waiting_tasks {
                    match service::physical_agent::PhysicalAgent::get_least_available_physical_agent(
                        &db,
                        waiting_task.qubits as u32,
                        waiting_task.depth as u32,
                    ).await {
                        Ok(Some(agent)) => {
                            service::physical_agent::PhysicalAgent::update_physical_agent_qubits_idle(
                                &db,
                                agent.id,
                                agent.qubit_idle - waiting_task.qubits as i32,
                            ).await.unwrap();

                            let db = db.clone();

                            tokio::spawn(async move {
                                consume_task(&db, schd_conf_cons.shed_min_depth as f32, schd_conf_cons.shed_min_gran as f32, waiting_task, agent).await
                            });
                        }
                        Ok(None) => { break; }
                        Err(err) => {
                            info!("Error: {}", err);
                            break;
                        }
                    }
                }

                // every 1 seconds to check if there are waiting tasks
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    });

    let axum_rt = tokio::runtime::Runtime::new().unwrap();
    axum_rt.block_on(async move {
        info!("Axum server started");
        // connect to the database
        if std::path::Path::new(".env").exists() {
            info!("Load .env file from current directory");
            dotenv::from_filename(".env").ok();
        }
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

        info!("Axum server connect database: {}", base_url);

        // disable sqlx logging
        let mut connection_options = ConnectOptions::new(base_url);
        connection_options.sqlx_logging(false);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let db: DbConn = Database::connect(connection_options).await.unwrap();
        // drop all tables and re-create them
        Migrator::fresh(&db).await.unwrap();

        // todo: read config from yaml file
        let state = Arc::new(RwLock::new(router::ServerState {
            db,
            config: schd_conf_axum,
        }));

        // Start the web server
        let emulator_router = Router::new()
            .route("/init", routing::get(router::add_physical_agent))
            .route("/submit", routing::post(router::submit))
            .route("/get_task", routing::get(router::get_task))
            .route("/get_task/:id", routing::get(router::get_task_with_id))
            .with_state(state.clone());

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!(
            "Axum server listening on: {}",
            listener.local_addr().unwrap()
        );
        axum::serve(listener, emulator_router).await.unwrap();
    })
}
