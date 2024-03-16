#![allow(deprecated)]
use axum::{routing, Router};
use log::info;
use migration::{Migrator, MigratorTrait};
pub use sea_orm::{ConnectOptions, Database, DbConn};
pub mod entity;
pub mod router;
pub mod service;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // Start a thread to consume waiting tasks, and submit them to idle agents
    std::thread::spawn(|| {
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
            let agent_addr =
                std::env::var("AGENT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:3004".to_owned());

            info!("Consume waiting task thread connect database: {}", base_url);
            info!("Agent address is: {}", agent_addr);

            // disable sqlx logging
            let mut connection_options = ConnectOptions::new(base_url);
            connection_options.sqlx_logging(false);

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let db: DbConn = Database::connect(connection_options).await.unwrap();

            loop {
                // Retrieve tasks awaiting assignment, the quantity of which is less than or
                // equal to the number of available agents.
                let waiting_tasks = service::task::Task::get_waiting_task(
                    &db,
                    Some(
                        service::resource::Resource::get_idle_agent_num(&db)
                            .await
                            .unwrap(),
                    ),
                )
                .await
                .unwrap();

                // for each waiting task, submit it to an idle agent
                for waiting_task in waiting_tasks {
                    info!(
                        "Consume waiting task thread submit task: {:?}",
                        waiting_task.id
                    );
                    let db = db.clone();
                    let agent_addr = agent_addr.clone();
                    tokio::spawn(async move {
                        router::consume_task_back(&db, &agent_addr, waiting_task).await;
                    });
                }

                // every 1 seconds to check if there are waiting tasks
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    });

    let axum_rt = tokio::runtime::Runtime::new().unwrap();
    axum_rt.block_on(async {
        info!("Axum server started");
        // connect to the database
        if std::path::Path::new(".env").exists() {
            info!("Load .env file from current directory");
            dotenv::from_filename(".env").ok();
        }
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());
        let agent_addr =
            std::env::var("AGENT_ADDR").unwrap_or_else(|_| "http://127.0.0.1:3004".to_owned());

        info!("Axum server connect database: {}", base_url);
        info!("Agent address is: {}", agent_addr);

        // disable sqlx logging
        let mut connection_options = ConnectOptions::new(base_url);
        connection_options.sqlx_logging(false);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let db: DbConn = Database::connect(connection_options).await.unwrap();
        Migrator::fresh(&db).await.unwrap();

        let state = router::ServerState { db, agent_addr };

        // Start the web server
        let emulator_router = Router::new()
            .route("/init", routing::get(router::init_qthread))
            .route("/submit", routing::post(router::submit))
            .route("/get_task", routing::get(router::get_task))
            .route("/get_task/:id", routing::get(router::get_task_with_id))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!(
            "Axum server listening on: {}",
            listener.local_addr().unwrap()
        );
        axum::serve(listener, emulator_router).await.unwrap();
    })
}
