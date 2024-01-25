use axum::{
    body, debug_handler,
    extract::Request,
    http::{header, StatusCode},
    routing, Json, Router,
};
use serde::Serialize;
extern crate qasmsim;

#[derive(Serialize)]
pub struct EmulateResponse {
    result: String,
}

pub async fn root() -> String {
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
        Ok(result) => {
            return qasmsim::print_result(&result, &options);
        }
        Err(err) => {
            return format!("Error: {}", err);
        }
    }
}

pub async fn consume_body(body: body::Body) -> (StatusCode, Json<EmulateResponse>) {
    let body_str = match body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => match String::from_utf8(bytes.to_vec()) {
            Ok(string) => string,
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(EmulateResponse {
                        result: format!("Error: {}", err),
                    }),
                )
            }
        },
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(EmulateResponse {
                    result: format!("Error: {}", err),
                }),
            )
        }
    };

    let parts = body_str.split("&").collect::<Vec<&str>>();
    let options = qasmsim::options::Options {
        shots: match parts[1].split("=").collect::<Vec<&str>>()[1].parse::<usize>() {
            Ok(shots) => Some(shots),
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(EmulateResponse {
                        result: format!("Error: {}", err),
                    }),
                )
            }
        },
        format: match parts[2].split("=").collect::<Vec<&str>>()[1] {
            "json" => qasmsim::options::Format::Json,
            "csv" => qasmsim::options::Format::Tabular,
            _ => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(EmulateResponse {
                        result: "Error: format not specified / not support".to_string(),
                    }),
                )
            }
        },
        ..Default::default()
    };

    match qasmsim::run(parts[0], options.shots) {
        Ok(result) => (
            StatusCode::OK,
            Json(EmulateResponse {
                result: qasmsim::print_result(&result, &options),
            }),
        ),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(EmulateResponse {
                result: format!("Error: {}", err),
            }),
        ),
    }
}

pub async fn emulate(request: Request) -> (StatusCode, Json<EmulateResponse>) {
    // curl -v -H "Content-Type: x-www-form-urlencoded" -X POST 10.31.4.69:3000/emulate -d @bell.qasm -d shots=1000 -d format=json
    match request.headers().get(header::CONTENT_TYPE) {
        Some(content_type) if content_type == "x-www-form-urlencoded" => {
            consume_body(request.into_body()).await
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
        .route("/", routing::get(root))
        .route("/emulate", routing::post(emulate));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, emulator_router).await.unwrap();
}
