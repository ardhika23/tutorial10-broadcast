use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use futures_util::stream::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    bcast_tx: Sender<(SocketAddr, String)>,
    mut clients: HashMap<SocketAddr, Sender<String>>,
) -> Result<(), tokio_tungstenite::tungstenite::Error> {

    let (client_tx, _client_rx) = channel(16);

    clients.insert(addr, client_tx.clone());

    while let Some(Ok(msg)) = ws_stream.next().await {
        if let Message::Text(text) = msg {
            println!("From client {}: {}", addr, text);
            bcast_tx.send((addr, text)).ok();
        }
    }

    clients.remove(&addr);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (bcast_tx, _) = channel(16);
    let clients: HashMap<SocketAddr, Sender<String>> = HashMap::new(); // Removed `mut` from here

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on port 8080");

    while let Ok((socket, addr)) = listener.accept().await {
        println!("New connection from {}", addr);

        let bcast_tx = bcast_tx.clone();
        let clients_clone = clients.clone();

        tokio::spawn(async move {
            let ws_stream = accept_async(socket).await.unwrap();

            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx, clients_clone).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}
