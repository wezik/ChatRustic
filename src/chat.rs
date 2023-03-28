use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Message {
    author: String,
    text: String,
}

pub async fn start_server(addr: &str) {
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TcpListener");

    let (message_sender, _) = broadcast::channel(10);

    start_chat(listener, message_sender).await;
}

async fn start_chat(listener: TcpListener, message_sender: Sender<Message>) {
    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .expect("Failed to accept connection");

        connect_client(socket, &message_sender, addr).await;
    }
}

async fn connect_client(mut socket: TcpStream, sender: &Sender<Message>, addr: SocketAddr) {
    let sender = sender.clone();
    let mut receiver = sender.subscribe();

    tokio::spawn(async move {
        println!("[{:?}] connected", addr);
        let (read, mut write) = socket.split();
        let mut reader = BufReader::new(read);
        let mut line = String::new();

        loop {
            tokio::select! {
                result = reader.read_line(&mut line) => {
                    if result.unwrap() == 0 {
                        //disconnects
                        println!("[{:?}] disconnected", addr);
                        break;
                    }
                    let msg = serde_json::from_str(&line).expect("Incorrect message");
                    sender.send(msg).expect("Failed to deserialize message");
                    line.clear();
                },
                result = receiver.recv() => {
                    let msg: Message = result.expect("Failed to receive message");
                    let jsoned = serde_json::to_string(&msg).expect("Failed to serialize message");
                    write.write_all(&jsoned.into_bytes()).await.expect("Failed to write received message");
                }
            }
        }
    });
}
