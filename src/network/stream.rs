use std::{marker::PhantomData, task::Poll};

use bytes::BytesMut;

use futures::{Stream, Sink, ready, FutureExt};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{FrameCoder, KvError, network::frame::read_frame};



/// 处理Kv Server prost frame 的 stream
pub struct ProstStream<S, In, Out>  {
    // inner stream
    stream: S,
    // 写缓存
    wbuf: BytesMut,
    // 读缓存
    rbuf: BytesMut,

    // 类型占位符
    _in: PhantomData<In>,
    _out: PhantomData<Out>,
}

impl<S, In, Out> Stream for ProstStream<S, In, Out>
where 
    S: AsyncRead + AsyncWrite + Unpin + Send,
    In: Unpin + Send + FrameCoder,
    Out: Unpin + Send,
{
    // 调用next() 时候得到 Result<In, KvError>
    type Item = Result<In, KvError>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        // 上次调用结束后,rbuf应该为空
        assert!(self.rbuf.len() == 0);
        // 从rbuf中分离rest
        let mut rest = self.rbuf.split_off(0);
        // 使用read_frame来获取数据
        let fut = read_frame(&mut self.stream, &mut rest);
        ready!(Box::pin(fut).poll_unpin(cx))?;

        self.rbuf.unsplit(rest);

        Poll::Ready(Some(In::decode_frame(&mut self.rbuf)))

    }
}

impl<S, In, Out> Sink<Out> for ProstStream<S, In, Out>
where 
    S: AsyncRead + AsyncWrite + Unpin,
    In: Unpin + Send,
    Out: Unpin + Send + FrameCoder,
{
    type Error = KvError;

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Out) -> Result<(), Self::Error> {
        todo!()
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

}