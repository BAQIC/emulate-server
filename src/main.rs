use axum::{routing, Router};
use log::info;
use migration::{Migrator, MigratorTrait};
pub use sea_orm::{ConnectOptions, Database, DbConn};
pub mod config;
pub mod entity;
pub mod router;
pub mod service;
pub mod task;
use router::{
    physical_agent::{add_physical_agent_from_file, get_agent_info},
    task::consume_task,
};

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    // Start a thread to consume waiting tasks, and submit them to idle agents
    std::thread::spawn(move || {
        info!("Consume waiting task thread started");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // connect to the database
            if std::path::Path::new(".env").exists() {
                info!("Load db.env file from config directory");
                dotenv::from_filename(".env").ok();
            }
            let base_url =
                std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

            info!("Consume waiting task thread connect database: {}", base_url);

            // read sheduler config and agents infomation from json file
            let sched_conf = config::get_qsched_config("config/qsched.json");
            let agents = get_agent_info("config/agents.json");

            // disable sqlx logging
            let mut connection_options = ConnectOptions::new(base_url);
            connection_options.sqlx_logging(false);

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let db: DbConn = Database::connect(connection_options).await.unwrap();

            // add agents to the database
            add_physical_agent_from_file(&db, agents).await;

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
                                consume_task(&db, sched_conf.sched_min_depth as f32, sched_conf.sched_min_gran as f32, waiting_task, agent).await
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
            info!("Load .env file from config directory");
            dotenv::from_filename(".env").ok();
        }
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

        info!("Axum server connect database: {}", base_url);

        // read sheduler config from json file
        let sched_conf = config::get_qsched_config("config/qsched.json");

        // disable sqlx logging
        let mut connection_options = ConnectOptions::new(base_url);
        connection_options.sqlx_logging(false);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let db: DbConn = Database::connect(connection_options).await.unwrap();
        // drop all tables and re-create them
        Migrator::fresh(&db).await.unwrap();

        // todo: read config from yaml file
        let state = router::ServerState {
            db,
            config: sched_conf,
        };

        // Start the web server
        let emulator_router = Router::new()
            .route(
                "/add_agent",
                routing::get(router::physical_agent::add_physical_agent),
            )
            .route(
                "/get_agents",
                routing::get(router::physical_agent::get_physical_agent_by_address),
            )
            .route(
                "/update_agent_status",
                routing::get(router::physical_agent::update_physical_agent_status),
            )
            .route(
                "/update_agent",
                routing::post(router::physical_agent::update_physical_agent),
            )
            .route("/submit", routing::post(router::task::submit))
            .route("/get_task", routing::get(router::task::get_task))
            .route(
                "/get_task/:id",
                routing::get(router::task::get_task_with_id),
            )
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!(
            "Axum server listening on: {}",
            listener.local_addr().unwrap()
        );
        axum::serve(listener, emulator_router).await.unwrap();
    })
}
