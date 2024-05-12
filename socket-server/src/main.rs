use axum::extract::ws::Message;
use axum::{
    http::Method,
    routing::{delete, get, post},
    Router,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{net::TcpListener, sync::mpsc};
use tower_http::cors::{Any, CorsLayer};
mod handlers;
mod ws;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, axum::Error>>>,
}

#[derive(Default, Clone)]
struct AppState {
    clients: Arc<Mutex<HashMap<String, Client>>>,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/health", get(handlers::health_handler))
        .route("/publish", post(handlers::publish_handler))
        .nest(
            "/register",
            Router::new()
                .route("/", post(handlers::register_handler))
                .route("/:param", delete(handlers::unregister_handler)),
        )
        .route("/ws/:id", get(handlers::ws_handler))
        .with_state(AppState::default().clone())
        .layer(cors);

    let tcp_listener = TcpListener::bind("127.0.0.1:8000")
        .await
        .expect("Address should be free and valid");
    axum::serve(tcp_listener, app)
        .await
        .expect("Error serving application")
}
