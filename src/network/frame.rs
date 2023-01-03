use std::io::{Write, Read};

use bytes::{BytesMut, BufMut, Buf};
use flate2::{Compression, write::GzEncoder, read::GzDecoder};
use prost::Message;
use tracing::debug;

use crate::{KvError, CommandRequest, CommandResponse};


// 长度占用4个字节
pub const LEN_LEN: usize = 4;
// 长度占 31 bit, 所以最大的 frame 是 2G
const MAX_FRAME: usize = 2 * 1024 * 1024 * 1024;
// 如果 payload 超过 1436 字节， 做压缩
// 这是因为以太网的 MTU 是1500， 除去 IP 头20字节， TCP 头20字节，还有 1460;
// 一般TCP包还会有一个Option(比如timestamp), IP包内也有可能包含，取20字节预留；再减去4字节的长度，就是不用分片的最大消息长度。
const COMPRESSION_LIMIT: usize = 1436;
// 代表压缩位的最高位
const COMPRESSION_BIT: usize = 1 << 31;

// 处理 Frame 的 encode/decode
pub trait FrameCoder
where 
    Self: Message + Sized + Default,
{
    /// 把一个 Message encode 成一个 frame
    fn encode_frame(&self, buf: &mut BytesMut) -> Result<(), KvError> {
        let size = self.encoded_len();

        if size >= MAX_FRAME {
            return Err(KvError::FrameError);
        }

        // 首先在 buf 中写入 payload 长度， 如果需要压缩，再重写压缩的长度
        buf.put_u32(size as _);

        if size > COMPRESSION_LIMIT {
            let mut buf1 = Vec::with_capacity(size);
            self.encode(&mut buf1)?;

            let payload = buf.split_off(LEN_LEN);
            buf.clear();

            let mut encoder = GzEncoder::new(payload.writer(), Compression::default());
            encoder.write_all(&buf1[..])?;

            // 压缩完成后, 从 gzip encoder 中把 BytesMut 写回 buf
            let payload = encoder.finish()?.into_inner();
            debug!("Encode a frame: size {} {}", size, payload.len());

            buf.put_u32((payload.len() | COMPRESSION_BIT) as _);

            buf.unsplit(payload);

            Ok(())
        } else {
            self.encode(buf)?;
            Ok(())
        }
    }

    /// 把一个 frame decode 成一个 Message
    fn decode_frame(buf: &mut BytesMut) -> Result<Self, KvError> {
        // 先取4个字节，拿到长度和压缩位
        let header = buf.get_u32() as usize;
        let (len, compressed) = decode_header(header);
        debug!("Got a frame: size {} {}", len, compressed);

        if compressed {
            // 解压缩
            let mut decoder = GzDecoder::new(&buf[..len]);
            let mut buf1 = Vec::with_capacity(len * 2);
            decoder.read_to_end(&mut buf1)?;
            buf.advance(len);

            // decode 成相应的信息
            Ok(Self::decode(&buf1[..buf1.len()])?)
        } else {
            let msg = Self::decode(&buf[..len])?;
            buf.advance(len);
            Ok(msg)
        }
    }
}

impl FrameCoder for CommandRequest {}
impl FrameCoder for CommandResponse {}

fn decode_header(header: usize) -> (usize, bool) {
    let compressed = header & COMPRESSION_BIT != 0;
    let len = header & !COMPRESSION_BIT;
    (len, compressed)
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::Value;

    use super::*;

    #[test]
    fn command_request_encode_decode_should_work() {
        let mut buf = BytesMut::new();

        let cmd = CommandRequest::new_hdel("t1", "k1");
        cmd.encode_frame(&mut buf).unwrap();

        assert_eq!(is_compressed(&buf), false);

        let cmd1 = CommandRequest::decode_frame(&mut buf).unwrap();
        assert_eq!(cmd, cmd1);
    }

    #[test]
    fn command_response_encode_decode_should_work() {
        let mut buf = BytesMut::new();

        let values: Vec<Value> = vec![1.into(), "hello".into(), b"data".into()];    
        let res: CommandResponse = values.into();
        res.encode_frame(&mut buf).unwrap();

        assert_eq!(is_compressed(&buf), false);

        let res1 = CommandResponse::decode_frame(&mut buf).unwrap();

        assert_eq!(res, res1);
        
    }

    #[test]
    fn command_response_comressed_encode_decode_should_work() {
        let mut buf = BytesMut::new();

        let value: Value = Bytes::from(vec![0u8; COMPRESSION_LIMIT +1]).into(); 
        let res: CommandResponse = value.into();
        res.encode_frame(&mut buf).unwrap();

        assert_eq!(is_compressed(&buf), true);

        let res1 = CommandResponse::decode_frame(&mut buf).unwrap();
        assert_eq!(res, res1);    
    }




    fn is_compressed(data: &[u8]) -> bool {
        if let &[v] = &data[..1] {
            v >> 7 == 1
        } else {
            false
        }
    }
}