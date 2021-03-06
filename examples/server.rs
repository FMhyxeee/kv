use anyhow::Result;
use async_prost::AsyncProstStream;
use futures::StreamExt;
use kv::{CommandRequest, CommandResponse, memory::MemTable, ServiceInner, Service};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let service: Service = ServiceInner::new(MemTable::new()).into();

    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("client {:?} connected", addr);
        let svc = service.clone();
        tokio::spawn(async move {
            let mut stream =
                AsyncProstStream::<_, CommandRequest, CommandResponse, _>::from(stream).for_async();
            while let Some(Ok(cmd)) = stream.next().await {
                info!("Gto a new command {:?}", cmd);
                let res = svc.execute(cmd);
                println!("{res:?}")
            }
            info!("Client {:?} dealed", addr);
        });
    }
}
