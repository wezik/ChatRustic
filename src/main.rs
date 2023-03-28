mod chat;

#[tokio::main]
async fn main() {
    chat::start_server("localhost:8080").await;
}
