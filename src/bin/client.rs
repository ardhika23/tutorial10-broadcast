use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
        .connect()
        .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From server: {}", text);
                        }
                    },
                    Some(Err(err)) => {
                        eprintln!("Error receiving from server: {}", err);
                        return Err(err.into());
                    },
                    None => {
                        println!("Server connection closed.");
                        return Ok(());
                    },
                }
            }
            res = stdin.next_line() => {
                match res {
                    Ok(None) => {
                        println!("Input stream closed.");
                        return Ok(());
                    },
                    Ok(Some(line)) => {
                        if let Err(err) = ws_stream.send(Message::text(line)).await {
                            eprintln!("Error sending message to server: {}", err);
                            return Err(err.into());
                        }
                    },
                    Err(err) => {
                        eprintln!("Error reading from input: {}", err);
                        return Err(err.into());
                    },
                }
            }
        }
    }
}
