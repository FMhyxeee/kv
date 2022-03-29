
use anyhow::Result;
use kv::{ServiceInner, memory::MemTable, Service, ProstServerStream};
use tokio::net::TcpListener;
use tracing::info;


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:9527";
    let service: Service = ServiceInner::new(MemTable::new()).into();
    let listener = TcpListener::bind(addr).await?;
    info!("start server at {}", addr);
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client connected: {}", addr);
        let stream = ProstServerStream::new(stream, service.clone());
        tokio::spawn(async move {
            stream.process().await;
        });
    }
}