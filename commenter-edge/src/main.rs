mod comments;
mod commenting;
mod manager;
mod stomp;

use std::{
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering
        }
    },
    collections::HashMap
};

use comments::Comment;
use futures_util::{SinkExt, StreamExt, TryFutureExt};

use stomp::StompFrame;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::sync::{
    mpsc,
    RwLock
};

use warp::{
    self,
    filters::ws::{WebSocket, Ws},
    Filter,
};

// use manager::Manager;

type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Comment>>>>;
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    // let manager = Arc::new(Manager::default());
    // let manager = warp::any().map(move || manager.clone());
    let users = Users::default();
    let users = warp::any().map(move || users.clone());

    let ws_endpoint = warp::path("ws")
        .and(warp::ws())
        .and(users)
        .map(|ws: Ws, users| ws.on_upgrade(move |socket| handle_connection(socket, users)));

    warp::serve(ws_endpoint).run(([127, 0, 0, 1], 5080)).await;
}

async fn handle_connection(ws: WebSocket, users: Users) {
    let user_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    // Split user socket to receiving and producing parts
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Create buffer channel for outgoing comments
    let (mut tx, mut rx) = mpsc::unbounded_channel::<Comment>();
    let mut rx = UnboundedReceiverStream::new(rx);

    // Create async task that will listen for outgoing comments and push them to the websocket buffer
    tokio::task::spawn(async move {
        while let Some(comment) = rx.next().await {

            // let message = comment.into();
            // user_ws_tx
            //     .send(message)
            //     .unwrap_or_else(|e| {
            //         eprintln!("websocket send error: {}", e);
            //     })
            //     .await;
        }
    });

    // Register users in our dict so we could funout the messages
    users.write().await.insert(user_id, tx);

    // Loop for icoming messages from them socket
    while let Some(result) = user_ws_rx.next().await {
        let stomp_frame_parsing_result = match result {
            Ok(msg) =>StompFrame::new(msg),
            Err(err) => {
                println!("Error on WS conn..");
                break;
            }
        };

        if let Err(err) = stomp_frame_parsing_result {
            println!("{}", err);
        }

        // println!("Parsed ok? {}", stomp_frame_parsing_result.is_ok());
    }
}
