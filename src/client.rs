
use kv::{ProstClientStream, CommandRequest};
use tokio::net::TcpStream;
use tracing::info;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:9527";
    let stream = TcpStream::connect(addr).await?;

    let mut client = ProstClientStream::new(stream);

    //生成一个Hset命令
    let cmd = CommandRequest::new_hset("t1", "k1", "v1".into());

    //发送Hset，等待相应
    let res = client.execute(cmd).await?;
    info!("res: {:?}", res);

    Ok(())
}