use crate::{AppState, Client};
use axum::extract::ws::{Message, WebSocket};
use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use tokio_stream::wrappers::UnboundedReceiverStream;
#[derive(Deserialize, Debug)]
pub struct TopicRequest {
    topics: Vec<String>,
}

pub async fn client_connection(ws: WebSocket, id: String, state: AppState, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = tokio::sync::mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));
    client.sender = Some(client_sender);
    state.clients.lock().unwrap().insert(id.clone(), client);
    println!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &state).await;
    }
    state.clients.lock().unwrap().remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, state: &AppState) {
    println!("received msg from {}: {:?}", id, msg);
    let message = match msg.to_text() {
        Ok(v) => v,
        Err(_) => return,
    };
    if message == "ping" || message == "ping\n" {
        return;
    }
    let topics_req: Result<TopicRequest, _> = serde_json::from_str(message);
    match topics_req {
        Ok(req) => {
            // Update client's topic in AppState
            if let Some(client) = state.clients.lock().unwrap().get_mut(id) {
                client.topics = req.topics;
            }
        }
        Err(e) => {
            eprintln!("error while parsing message to topics request: {}", e);
        }
    }
}
