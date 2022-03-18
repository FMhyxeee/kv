
use anyhow::Result;
use async_prost::AsyncProstStream;
use futures::{SinkExt, StreamExt};
use kv::{CommandResponse, CommandRequest};
use tokio::net::TcpStream;
use tracing::info;



#[tokio::main]
async fn main() -> Result<()>{
    // 初始化日志
    tracing_subscriber::fmt::init();

    // Tcp 连接
    let addr = "127.0.0.1:9527";
    let stream = TcpStream::connect(addr).await?;

    // AsyncProstStream来处理TCP Frame

    let mut client = 
        AsyncProstStream::<_, CommandResponse, CommandRequest, _>::from(stream).for_async();
    

    // 生成一个HSET命令来测试一下这几个结构体
    let cmd = CommandRequest::new_hset("table1", "hello", "value".into());

    client.send(cmd).await?;

    if let Some(Ok(data)) = client.next().await {
        info!("Got Response {:?}", data)
    }

    Ok(())
}