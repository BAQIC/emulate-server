use axum::{debug_handler, routing, Router};
use std::process::Command;

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
pub async fn emulate_message() -> String {
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

#[tokio::main]
async fn main() {
    let emulator_router = Router::new().route("/", routing::get(emulate));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, emulator_router).await.unwrap();
}
