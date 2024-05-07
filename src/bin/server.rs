use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;
use gethostname::gethostname;

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ws_stream
        .send(Message::Text("Welcome to chat! Type a message".to_string()))
        .await?;
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Message::Text(text) = msg {
                            println!("From client {}: {}", addr, text);
                            bcast_tx.send(text.clone())?;
                            bcast_tx.send(format!("{} : {}", addr, text))?;
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::Text(msg?)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on port 8080");
    let hostname = gethostname().into_string().unwrap_or_else(|_| "unknown".to_string());

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}'s Computer {}", hostname, addr);

        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            let ws_stream = accept_async(socket).await.unwrap();
            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}
