use axum::extract::ws::CloseFrame;
use axum::Router;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};

// WebSocketUpgrade: Extractor for establishing WebSocket connections.
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // Finalize upgrading the connection and call the provided callback with the stream.
    ws.on_failed_upgrade(|error| println!("Error upgrading websocket: {}", error))
        .on_upgrade(handle_socket)
}

// WebSocket: A stream of WebSocket messages.
async fn handle_socket(mut socket: WebSocket) {
    // Returns `None` if the stream has closed.
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(utf8_bytes) => {
                    println!("Text received: {}", utf8_bytes);
                    let result = socket
                        .send(Message::Text(
                            format!("Echo back text: {}", utf8_bytes).into(),
                        ))
                        .await;
                    if let Err(error) = result {
                        println!("Error sending: {}", error);
                        send_close_message(socket, 1011, &format!("Error occured: {}", error))
                            .await;
                        break;
                    }
                }
                Message::Binary(bytes) => {
                    println!("Received bytes of length: {}", bytes.len());
                    let result = socket
                        .send(Message::Text(
                            format!("Received bytes of length: {}", bytes.len()).into(),
                        ))
                        .await;
                    if let Err(error) = result {
                        println!("Error sending: {}", error);
                        send_close_message(socket, 1011, &format!("Error occured: {}", error))
                            .await;
                        break;
                    }
                }
                // Close, Ping, Pong will be handled automatically
                // Message::Close
                // After receiving a close frame, axum will automatically respond with a close frame if necessary (you do not have to deal with this yourself).
                // After sending a close frame, you may still read messages, but attempts to send another message will error.
                // Since no further messages will be received, you may either do nothing or explicitly drop the connection.
                _ => {}
            }
        } else {
            println!("Error receiving message: {:?}", msg.err());
            return;
        }
    }
}

// We MAY “uncleanly” close a WebSocket connection at any time by simply dropping the WebSocket, ie: Break out of the recv loop.
// However, you may also use the graceful closing protocol, in which
// peer A sends a close frame, and does not send any further messages;
// peer B responds with a close frame, and does not send any further messages;
// peer A processes the remaining messages sent by peer B, before finally
// both peers close the connection.
//
// Close Code: https://kapeli.com/cheat_sheets/WebSocket_Status_Codes.docset/Contents/Resources/Documents/index
async fn send_close_message(mut socket: WebSocket, code: u16, reason: &str) {
    _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: code,
            reason: reason.into(),
        })))
        .await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/web_socket", get(websocket_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
