use crate::ws;
use crate::{AppState, Client};
use axum::{
    extract::{ws::Message, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(Serialize, Clone)]
pub struct RegisterResponse {
    url: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Event {
    topic: String,
    user_id: Option<usize>,
    message: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TopicsRequest {
    topics: Vec<String>,
}
pub async fn publish_handler(
    State(state): State<AppState>,
    body: axum::extract::Json<Event>,
) -> impl IntoResponse {
    state
        .clients
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, client)| match &body.user_id {
            Some(v) => client.user_id == *v,
            None => true,
        })
        .filter(|(_, client)| client.topics.contains(&body.topic))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::Text(body.message.clone())));
            }
        });
    println!("{:?}", state.clients);
    Response::builder()
        .status(StatusCode::OK)
        .body("Published event".to_string())
        .unwrap()
}

pub async fn register_handler(
    State(state): State<AppState>,
    body: axum::extract::Json<RegisterRequest>,
) -> impl IntoResponse {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();
    println!("{:?}: {:?}", user_id, uuid);
    register_client(uuid.clone(), user_id, state).await;
    Json(RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    })
}

async fn register_client(id: String, user_id: usize, state: AppState) {
    state.clients.lock().unwrap().insert(
        id,
        Client {
            user_id,
            topics: vec![String::from("cats")],
            sender: None,
        },
    );
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let client = state.clients.lock().unwrap().get(&id).cloned().unwrap();
    ws.on_upgrade(move |socket| ws::client_connection(socket, id.clone(), state, client))
}

pub async fn unregister_handler(
    State(state): State<AppState>,
    Path(param): Path<String>,
) -> Response<String> {
    let removed = state.clients.lock().unwrap().remove(&param);
    match removed {
        Some(_) => Response::builder()
            .status(StatusCode::OK)
            .body(format!("Client removed with id {}", param))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(format!("Client not found with id {}", param))
            .unwrap(),
    }
}

pub async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "data":{
            "message": "Route Working Properly"
        }
    }))
}
