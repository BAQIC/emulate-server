use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    routing, Form, Json, RequestExt, Router,
};
use dotenv;
use entity::sea_orm_active_enums;
use migration::{Migrator, MigratorTrait};
use qasmsim;
pub use sea_orm::{Database, DbConn};
use serde::Deserialize;
use serde_json::{json, Value};
pub mod entity;
pub mod service;

#[derive(Deserialize)]
struct EmulateMessage {
    qasm: String,
    shots: usize,
    format: String,
}

#[derive(Clone)]
pub struct ServerState {
    pub db: DbConn,
}

pub async fn root() -> (StatusCode, Json<Value>) {
    let source = "
    OPENQASM 2.0;
    include \"qelib1.inc\";
    qreg q[2];
    creg c[2];
    x q;
    h q;
    measure q -> c;
    ";

    let options = qasmsim::options::Options {
        shots: Some(1000),
        format: qasmsim::options::Format::Json,
        ..Default::default()
    };

    match qasmsim::run(source, options.shots) {
        Ok(result) => (
            StatusCode::OK,
            match options.format {
                qasmsim::options::Format::Json => Json(
                    serde_json::from_str::<Value>(&qasmsim::print_result(&result, &options))
                        .unwrap(),
                ),
                qasmsim::options::Format::Tabular => {
                    Json(json!({"Result": qasmsim::print_result(&result, &options)}))
                }
            },
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}

async fn consume_body(Form(emulate_message): Form<EmulateMessage>) -> (StatusCode, Json<Value>) {
    let options = qasmsim::options::Options {
        shots: if emulate_message.shots == 0 {
            None
        } else {
            Some(emulate_message.shots)
        },
        format: match emulate_message.format.as_str() {
            "json" => qasmsim::options::Format::Json,
            "tabular" => qasmsim::options::Format::Tabular,
            _ => qasmsim::options::Format::Json,
        },
        ..Default::default()
    };

    match qasmsim::run(&emulate_message.qasm, options.shots) {
        Ok(result) => (
            StatusCode::OK,
            match options.format {
                qasmsim::options::Format::Json => Json(
                    serde_json::from_str::<Value>(&qasmsim::print_result(&result, &options))
                        .unwrap(),
                ),
                qasmsim::options::Format::Tabular => {
                    Json(json!({"Result": qasmsim::print_result(&result, &options)}))
                }
            },
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}

pub async fn emulate(request: Request) -> (StatusCode, Json<Value>) {
    // curl -v -H "Content-Type: x-www-form-urlencoded" -X POST 10.31.4.69:3000/emulate -d @bell.qasm -d shots=1000 -d format=json
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == "application/x-www-form-urlencoded" => {
            let Form(emulate_message) = request.extract().await.unwrap();
            consume_body(Form(emulate_message)).await
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"Error": format!("content type {:?} not specified / not support", request.headers().get(header::CONTENT_TYPE).unwrap())}),
            ),
        ),
    }
}

pub async fn init_db(state: State<ServerState>) -> (StatusCode, Json<Value>) {
    let db = &state.db;
    match service::resource::Resource::random_init_physical_agents(db, 10).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"Message": "Physical Agents added successfully"})),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}

async fn consume_task(
    Form(emulate_message): Form<EmulateMessage>,
    state: State<ServerState>,
) -> (StatusCode, Json<Value>) {
    let options = qasmsim::options::Options {
        shots: if emulate_message.shots == 0 {
            None
        } else {
            Some(emulate_message.shots)
        },
        format: match emulate_message.format.as_str() {
            "json" => qasmsim::options::Format::Json,
            "tabular" => qasmsim::options::Format::Tabular,
            _ => qasmsim::options::Format::Json,
        },
        ..Default::default()
    };

    let task = service::qthread::Qthread::submit_task(
        &state.db,
        entity::task::Model {
            id: uuid::Uuid::new_v4(),
            option_id: service::options::Options::add_qasm_options(&state.db, &options)
                .await
                .unwrap()
                .id,
            source: emulate_message.qasm.clone(),
            status: sea_orm_active_enums::TaskStatus::NotStarted,
            result: None,
            created_time: chrono::Utc::now().naive_utc(),
            updated_time: chrono::Utc::now().naive_utc(),
            agent_id: None,
        },
    )
    .await
    .unwrap();

    match task.status {
        sea_orm_active_enums::TaskStatus::Running => {
            match qasmsim::run(&emulate_message.qasm, options.shots) {
                Ok(result) => {
                    service::qthread::Qthread::finish_task(
                        &state.db,
                        task.id,
                        Some(qasmsim::print_result(&result, &options)),
                        sea_orm_active_enums::TaskStatus::Succeeded,
                        sea_orm_active_enums::AgentStatus::Succeeded,
                    )
                    .await
                    .unwrap();
                    (
                        StatusCode::OK,
                        match options.format {
                            qasmsim::options::Format::Json => Json(
                                serde_json::from_str::<Value>(&qasmsim::print_result(
                                    &result, &options,
                                ))
                                .unwrap(),
                            ),
                            qasmsim::options::Format::Tabular => {
                                Json(json!({"Result": qasmsim::print_result(&result, &options)}))
                            }
                        },
                    )
                }
                Err(err) => {
                    service::qthread::Qthread::finish_task(
                        &state.db,
                        task.id,
                        Some(format!("{}", err)),
                        sea_orm_active_enums::TaskStatus::Failed,
                        sea_orm_active_enums::AgentStatus::Failed,
                    )
                    .await
                    .unwrap();
                    (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"Error": format!("{}", err)})),
                    )
                }
            }
        }

        sea_orm_active_enums::TaskStatus::NotStarted => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": "There is no available agent to run the task"})),
        ),
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": "Task status is not valid"})),
        ),
    }
}

pub async fn submit(state: State<ServerState>, request: Request) -> (StatusCode, Json<Value>) {
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == "application/x-www-form-urlencoded" => {
            let Form(emulate_message) = request.extract().await.unwrap();
            consume_task(Form(emulate_message), state).await
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"Error": format!("content type {:?} not specified / not support", request.headers().get(header::CONTENT_TYPE).unwrap())}),
            ),
        ),
    }
}

#[tokio::main]
async fn main() {
    dotenv::from_filename(".env").ok();
    let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_owned());

    let db: DbConn = Database::connect(base_url).await.unwrap();
    Migrator::fresh(&db).await.unwrap();

    let state = ServerState { db };

    let emulator_router = Router::new()
        .route("/", routing::get(root))
        .route("/init", routing::get(init_db))
        .route("/submit", routing::post(submit))
        .route("/emulate", routing::post(emulate))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, emulator_router).await.unwrap();
}
