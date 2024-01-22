use axum::{
    debug_handler,
    http::{header, HeaderMap, StatusCode},
    routing, Json, Router,
};
use serde::Serialize;
use std::process::Command;

#[derive(Serialize)]
pub struct EmulateResponse {
    result: String,
}

#[debug_handler]
pub async fn emulate() -> String {
    let output = Command::new(
        "/home/lucky/Code/cuda-quantum/docs/sphinx/examples/cpp/providers/out-emulate.x",
    )
    .arg("/home/lucky/Code/cuda-quantum/docs/sphinx/examples/cpp/providers/emulate_message")
    .output()
    .expect("failed to execute process");

    match output.status.code() {
        Some(0) => String::from_utf8_lossy(&output.stdout).to_string(),
        Some(_) => String::from_utf8_lossy(&output.stderr).to_string(),
        None => "Error: terminated by signal".to_string(),
    }
}

#[debug_handler]
pub async fn emulate_message(headers: HeaderMap) -> (StatusCode, Json<EmulateResponse>) {
    // using test: curl -v -H "Content-Type: application/x-protobuf" -X POST 10.31.4.69:3000/emulate
    match headers.get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == "application/x-protobuf" => {
            let output = Command::new(
                "/home/lucky/Code/cuda-quantum/docs/sphinx/examples/cpp/providers/out-emulate.x",
            )
            .arg("/home/lucky/Code/cuda-quantum/docs/sphinx/examples/cpp/providers/emulate_message")
            .output()
            .expect("failed to execute process");

            match output.status.code() {
                Some(0) => (
                    StatusCode::OK,
                    Json(EmulateResponse {
                        result: String::from_utf8_lossy(&output.stdout).to_string(),
                    }),
                ),
                Some(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(EmulateResponse {
                        result: String::from_utf8_lossy(&output.stderr).to_string(),
                    }),
                ),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(EmulateResponse {
                        result: "Error: terminated by signal".to_string(),
                    }),
                ),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(EmulateResponse {
                result: "Error: content type not specified / not support".to_string(),
            }),
        ),
    }
}

#[tokio::main]
async fn main() {
    let emulator_router = Router::new()
        .route("/", routing::get(emulate))
        .route("/emulate", routing::post(emulate_message));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, emulator_router).await.unwrap();
}
