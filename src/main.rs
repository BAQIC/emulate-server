use axum::{routing, Router};
use dotenv;
use migration::{Migrator, MigratorTrait};
pub use sea_orm::{Database, DbConn};

pub mod entity;
pub mod router;
pub mod service;

fn main() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            dotenv::from_filename(".env").ok();
            let base_url =
                std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

            let db: DbConn = Database::connect(base_url).await.unwrap();
            Migrator::fresh(&db).await.unwrap();

            loop {
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

                for waiting_task in waiting_tasks {
                    let db = db.clone();
                    tokio::spawn(async move {
                        router::consume_task_back(&db, waiting_task).await;
                    });
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
    });

    let axum_rt = tokio::runtime::Runtime::new().unwrap();
    axum_rt.block_on(async {
        dotenv::from_filename(".env").ok();
        let base_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

        let db: DbConn = Database::connect(base_url).await.unwrap();
        Migrator::fresh(&db).await.unwrap();

        let state = router::ServerState { db };

        let emulator_router = Router::new()
            .route("/", routing::get(router::root))
            .route("/init", routing::get(router::init_db))
            .route("/submit", routing::post(router::submit))
            .route("/emulate", routing::post(router::emulate))
            .route("/get_task", routing::get(router::get_task))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, emulator_router).await.unwrap();
    })
}
