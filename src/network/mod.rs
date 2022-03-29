mod frame;


use bytes::BytesMut;
pub use frame::FrameCoder;

use tokio::io::{AsyncWrite, AsyncRead, AsyncWriteExt};
use tracing::info;

use crate::{Service, KvError, CommandResponse, CommandRequest};

use self::frame::read_frame;

/// 处理服务器端某个accept下来的socket的读写
pub struct ProstServerStream<S> {
    inner: S,
    service: Service,
}

///处理客户端的socket读写
pub struct ProstClientStream<S> {
    inner: S,
}

impl<S> ProstServerStream<S> 
where 
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub fn new(stream: S, service: Service) -> Self {
        Self { 
            inner: stream,
            service,
         }
    }

    pub async fn process(mut self) -> Result<(), KvError> {
        while let Ok(cmd) = self.recv().await {
            info!("recv command: {:?}", cmd);
            let res = self.service.execute(cmd);
            self.send(res).await?;
        }

        Ok(())
    }

    async fn send(&mut self, msg: CommandResponse) -> Result<(), KvError> {
        let mut buf = BytesMut::new();
        msg.encode_frame(&mut buf);
        let encoded = buf.freeze();
        self.inner.write_all(&encoded[..]).await?;
        Ok(())
    }
    
    async fn recv(&mut self) -> Result<CommandRequest, KvError> {
        let mut buf = BytesMut::new();
        let stream = &mut self.inner;
        read_frame(stream, &mut buf).await?;
        CommandRequest::decode_frame(&mut buf)
    }
}

impl<S> ProstClientStream<S>
where 
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub fn new(stream: S) -> Self {
        Self { 
            inner: stream,
        }
    }

    pub async fn execute(&mut self, cmd: CommandRequest) -> Result<CommandResponse, KvError> {
        self.send(cmd).await?;
        Ok(self.recv().await?)
    }

    async fn send(&mut self, msg: CommandRequest) -> Result<(), KvError> {
        let mut buf = BytesMut::new();
        msg.encode_frame(&mut buf);
        let encoded = buf.freeze();
        self.inner.write_all(&encoded[..]).await?;
        Ok(())
    }
    
    async fn recv(&mut self) -> Result<CommandResponse, KvError> {
        let mut buf = BytesMut::new();
        let stream = &mut self.inner;
        read_frame(stream, &mut buf).await?;
        CommandResponse::decode_frame(&mut buf)
    }
}


#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use anyhow::{Result, Ok};
    
    use bytes::Bytes;
    use tokio::net::{TcpListener, TcpStream};

    use crate::{ServiceInner, Service, ProstServerStream, memory::MemTable, CommandRequest, Value, ProstClientStream, CommandResponse, Kvpair};

    #[tokio::test]
    async fn client_server_basic_communication_should_work() -> anyhow::Result<()> {
        let addr = start_server().await?;

        let stream = TcpStream::connect(&addr).await?;
        let mut client = ProstClientStream::new(stream);

        //发送Hset，等待相应
        let cmd = CommandRequest::new_hset("t1", "k1", "v1".into());
        let res = client.execute(cmd).await.unwrap();
        assert_res_ok(res, &[Value::default()], &[]);


        Ok(())      
    }

    #[tokio::test]
    async fn client_server_compression_should_workd() -> anyhow::Result<()> {
        let addr = start_server().await?;
        let stream = TcpStream::connect(addr).await?;
        let mut client = ProstClientStream::new(stream);

        let v: Value = Bytes::from(vec![0u8; 16384]).into();
        let cmd = CommandRequest::new_hset("t2", "k2", v.clone().into());
        let res = client.execute(cmd).await.unwrap();
        assert_res_ok(res, &[Value::default()], &[]);

        let cmd = CommandRequest::new_hget("t2", "k2");
        let res = client.execute(cmd).await.unwrap();
        assert_res_ok(res, &[v.into()], &[]);
        Ok(())
    }

    


    async fn start_server() -> Result<SocketAddr> {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let service: Service = ServiceInner::new(MemTable::new()).into();
                let server = ProstServerStream::new(stream, service);
                tokio::spawn(server.process());
            }
        });

        Ok(addr)
    }

    // 测试成功返回的结果
    fn assert_res_ok(mut res: CommandResponse, values: &[Value], pairs: &[Kvpair]) {
        res.pairs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(res.status, 200);
        assert_eq!(res.message, "");
        assert_eq!(res.values, values);
        assert_eq!(res.pairs, pairs);
    }

    // 测试失败返回的结果
    #[allow(dead_code)]
    fn assert_res_error(res: CommandResponse, code: u32, msg: &str) {
        assert_eq!(res.status, code);
        assert!(res.message.contains(msg));
        assert_eq!(res.values, &[]);
        assert_eq!(res.pairs, &[]);
    }
}