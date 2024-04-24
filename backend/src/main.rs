mod agent;
mod data;
mod entities;
mod functions;
mod parser;

use agent::reply;
use axum::{
    body::Bytes,
    extract::{Json, Multipart},
    routing::{get, post},
    Extension, Router,
};
use data::{ChatRequest, CreateSessionRequest, JsonDataResponse};
use entities::{prelude::*, sea_orm_active_enums::MessageType, *};
use erniebot_rs::chat::ChatEndpoint;
use sea_orm::*;
use socketioxide::{
    extract::{Data, SocketRef},
    SocketIo,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{self, info};

use crate::parser::parse_file;

fn ws_handler(s: SocketRef, db: DatabaseConnection, chat_endpoint: ChatEndpoint) {
    s.on(
        "chat",
        |s: SocketRef, Data::<serde_json::Value>(msg)| async move {
            let data = if let Ok(data) = serde_json::from_value::<ChatRequest>(msg) {
                data
            } else {
                let _ = s.emit(
                    "response",
                    JsonDataResponse {
                        code: 500,
                        data: serde_json::json!({
                            "error": "Invalid request"
                        }),
                    },
                );
                return;
            };

            let content = data.content;
            let session_id = data.session_id;
            if MessageType::try_from_value(&data.content_type).is_err() {
                let _ = s.emit(
                    "response",
                    JsonDataResponse {
                        code: 500,
                        data: serde_json::json!({
                            "error": "Invalid content type"
                        }),
                    },
                );
                return;
            };
            let result = reply(&content, session_id, &db, &chat_endpoint).await;
            let response = match result {
                Ok(result) => JsonDataResponse {
                    code: 200,
                    data: serde_json::json!({
                        "response": result
                    }),
                },
                Err(err) => JsonDataResponse {
                    code: 500,
                    data: serde_json::json!({
                        "error": err.to_string()
                    }),
                },
            };
            info!("response: {:?}", response);
            let result = s.emit("response", response);
            if let Err(e) = result {
                info!("emit response failed: {:?}", e);
            } else {
                info!("emit response success");
            }
        },
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let db = Database::connect("mysql://root:123456@localhost:3306/longtext_demo")
        .await
        .unwrap();
    let chat_endpoint = tokio::task::spawn_blocking(|| {
        ChatEndpoint::new_with_custom_endpoint("ernie-4.0-8k-preview")
        //ChatEndpoint::new(erniebot_rs::chat::ChatModel::ErnieBotTurbo)
    })
    .await
    .expect("create chat_endpoint failed")
    .unwrap();
    let (socket_io_layer, io) = SocketIo::new_layer();
    let db2 = db.clone();
    let chat_endpoint2 = chat_endpoint.clone();
    io.ns("/ws", |s: SocketRef| {
        ws_handler(s, db2, chat_endpoint2);
    });

    let app = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/create_session", post(create_session))
        .route("/reply_chat", post(reply_chat))
        .route("/upload", post(upload))
        .layer(Extension(db))
        .layer(Extension(chat_endpoint))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(socket_io_layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();
    println!("Server running on: http://0.0.0.0:8888");
    axum::serve(listener, app).await.unwrap();
}

#[axum_macros::debug_handler]
async fn create_session(
    Extension(db): Extension<DatabaseConnection>,
    Json(data): Json<CreateSessionRequest>,
) -> Json<JsonDataResponse> {
    let user_id = data.user_id;
    let session1 = session::ActiveModel {
        user_id: Set(user_id),
        create_time: Set(chrono::Utc::now()),
        last_update_time: Set(chrono::Utc::now()),
        ..Default::default()
    };
    let res = Session::insert(session1).exec(&db).await;
    match res {
        Ok(res) => {
            let session_id = res.last_insert_id;
            Json(JsonDataResponse {
                code: 200,
                data: serde_json::json!({
                    "session_id": session_id
                }),
            })
        }
        Err(e) => Json(JsonDataResponse {
            code: 500,
            data: serde_json::json!({
                "error": e.to_string()
            }),
        }),
    }
}

#[axum_macros::debug_handler]
async fn reply_chat(
    Extension(db): Extension<DatabaseConnection>,
    Extension(chat_endpoint): Extension<ChatEndpoint>,
    Json(data): Json<ChatRequest>,
) -> Json<JsonDataResponse> {
    let content = data.content;
    let session_id = data.session_id;
    let content_type = MessageType::try_from_value(&data.content_type);
    if content_type.is_err() {
        return Json(JsonDataResponse {
            code: 500,
            data: serde_json::json!({
                "error": "Invalid content type"
            }),
        });
    }
    let result = reply(&content, session_id, &db, &chat_endpoint).await;
    match result {
        Ok(result) => Json(JsonDataResponse {
            code: 200,
            data: serde_json::json!({
                "response": result
            }),
        }),
        Err(error) => Json(JsonDataResponse {
            code: 500,
            data: serde_json::json!({
                "error": error.to_string()
            }),
        }),
    }
}

#[axum_macros::debug_handler]
async fn upload(mut multipart: Multipart) -> Json<JsonDataResponse> {
    let mut session_id: Option<i32> = None;
    let mut content_type: Option<String> = None;
    let mut data: Option<Bytes> = None;
    let mut filename: Option<String> = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("sessionId") => {
                let value = field.text().await.unwrap();
                session_id = Some(value.parse().unwrap());
            }
            Some("file") => {
                content_type = Some(field.content_type().unwrap().to_string());
                filename = Some(field.file_name().unwrap().to_string());
                data = Some(field.bytes().await.unwrap());
            }
            Some(_) => continue,
            None => continue,
        }
    }
    println!("session_id: {:?}", session_id);
    println!("content_type: {:?}", content_type);
    println!("data.is_none(): {:?}", data.is_none());
    println!("filename: {:?}", filename);
    if session_id.is_none() || content_type.is_none() || data.is_none() || filename.is_none() {
        return Json(JsonDataResponse {
            code: 500,
            data: serde_json::json!({
                "error": "Invalid request"
            }),
        });
    }
    let valid_content_type = ["text/plain", "application/pdf"];
    if !valid_content_type.contains(&content_type.as_ref().unwrap().as_str()) {
        return Json(JsonDataResponse {
            code: 500,
            data: serde_json::json!({
                "error": "Invalid content type"
            }),
        });
    }
    let extension_name = filename.as_ref().unwrap().split('.').last().unwrap();
    let filepath = format!("./files/{}.{}", session_id.unwrap(), extension_name);
    //save data to filepath
    let data = data.unwrap();
    tokio::fs::write(&filepath, data).await.unwrap();
    let documents = parse_file(&filepath).unwrap();
    std::fs::write(format!("./files/{}.txt", session_id.unwrap()), documents).unwrap();

    Json(JsonDataResponse {
        code: 200,
        data: serde_json::json!({
            "session_id": session_id,
            "content_type": content_type,
        }),
    })
}
