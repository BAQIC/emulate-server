use axum::{
    extract::Request,
    http::{header, StatusCode},
    routing, Form, Json, RequestExt, Router,
};
use qasmsim;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct EmulateMessage {
    qasm: String,
    shots: usize,
    format: String,
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
            Json(serde_json::from_str::<Value>(&qasmsim::print_result(&result, &options)).unwrap()),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"Error": format!("{}", err)})),
        ),
    }
}

async fn consume_body(Form(emulate_message): Form<EmulateMessage>) -> (StatusCode, Json<Value>) {
    let options = qasmsim::options::Options {
        shots: Some(emulate_message.shots),
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
            Json(serde_json::from_str::<Value>(&qasmsim::print_result(&result, &options)).unwrap()),
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

#[tokio::main]
async fn main() {
    let emulator_router = Router::new()
        .route("/", routing::get(root))
        .route("/emulate", routing::post(emulate));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, emulator_router).await.unwrap();
}
