mod comments;
mod context;
mod stomp;

use context::ApplicationContext;
use stomp::{StompClientFrame, StompFrame};

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt, TryFutureExt};

use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use warp::{
    self,
    filters::ws::{WebSocket, Ws, Message},
    Filter,
};

#[tokio::main]
async fn main() {
    let context = Arc::new(ApplicationContext::new(&"localhost:9092"));
    let context_clone = context.clone();

    tokio::task::spawn(async move {
        context_clone.listen_blocking().await
    });

    let context_filter_wrapper = warp::any().map(move || context.clone());

    let ws_endpoint = warp::path("ws")
        .and(warp::ws())
        .and(context_filter_wrapper)
        .map(|ws: Ws, context| ws.on_upgrade(move |socket| handle_connection(socket, context)));

    warp::serve(ws_endpoint).run(([127, 0, 0, 1], 5050)).await;
}

async fn handle_connection(ws: WebSocket, context: Arc<ApplicationContext>) {
    // Split user socket to receiving and producing parts
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Create buffer channel for outgoing comments
    let (mut tx, mut rx) = mpsc::unbounded_channel::<StompFrame>();
    let mut rx = UnboundedReceiverStream::new(rx);

    // Create async task that will listen for outgoing comments and push them to the websocket buffer
    tokio::task::spawn(async move {
        while let Some(comment) = rx.next().await {
            user_ws_tx
                .send(Message::text(comment))
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Register user to context in order to obtain ID
    let user_id = context.add_user(tx).await;

    // Loop for icoming messages from them socket
    while let Some(result) = user_ws_rx.next().await {
        if let Ok(msg) = result {
            if let Ok(frame) = StompClientFrame::new(msg) {
                if let StompClientFrame::DISCONNECT = frame {
                    break; // wow... ugly as fuck...
                } else if let Err(msg_handling_err) = context.handle_client_frame(user_id, frame).await {
                    todo!("Handle msg handling errors: {:?}", msg_handling_err);
                }
            } else {
                todo!("Handle parsing errors");
            }
        } else {
            todo!("Handle receiving errors");
            break;
        }
    }

    context.remove_user(user_id).await;
}
