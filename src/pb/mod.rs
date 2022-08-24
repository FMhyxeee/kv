pub mod abi;

use std::vec;
use bytes::Bytes;
use prost::Message;
use reqwest::StatusCode;
use crate::{*, command_request::RequestData};








impl CommandRequest {
    // 创建 HSET 命令
    pub fn new_hset(table: impl Into<String>, key: impl Into<String>, value: Value) -> Self {
        Self { 
            request_data: Some(RequestData::Hset(Hset {
                table: table.into(),
                pair: Some(Kvpair::new(key, value)),
            })),
        }
    }
    // 创建 HGET 命令
    pub fn new_hget(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self { request_data: 
            Some(RequestData::Hget(
                Hget {
                    table: table.into(),
                    key: key.into(),
                }
            ))
        }
    }
    // 创建 HGETALL 命令
    pub fn new_hgetall(table: impl Into<String>) -> Self {
        Self { request_data:
            Some(RequestData::Hgetall(
                Hgetall {
                    table: table.into(),
                }
            ))
        }
    }

    // 创建 HMGET 命令
    pub fn new_hmget(table: impl Into<String>, keys: Vec<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hmget(Hmget {
                table: table.into(),
                keys,
            }))
        }
    }

    // 创建 HMSet 命令
    pub fn new_hmset(table: impl Into<String>, pairs: Vec<Kvpair>) -> Self {
        Self { 
            request_data: Some(RequestData::Hmset(
            Hmset {
                table: table.into(),
                pairs,
            }
        )) }
    }

    // 创建 HDEL 命令
    pub fn new_hdel(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self { request_data: Some(
            RequestData::Hdel(
                Hdel {
                    table: table.into(),
                    key: key.into(),
                }
            )
        ) }
    }

    // 创建 HMDEL 命令
    pub fn new_hmdel(table: impl Into<String>, keys: Vec<String>) -> Self {
        Self { request_data: Some(RequestData::Hmdel(
            Hmdel {
                table:table.into(),
                keys,
            }
        )) }
    }

    // 创建 HEXIST 命令
    pub fn new_hexist(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self { request_data: Some(RequestData::Hexist(
            Hexist{
                table: table.into(),
                key: key.into(),
            }
        )) }
    }

    // 创 HMEXIST 命令
    pub fn new_hmexist(table: impl Into<String>, keys: Vec<String>) -> Self {
        Self { request_data: Some(RequestData::Hmexist(
            Hmexist {
                table: table.into(),
                keys,
            }
        )) }
    }
}

impl Kvpair {
     // 创建一个新的 kv pair
     pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self { key: key.into(), value: Some(value) }
     }
}


// 从 String 转化为 Value
impl From<String> for Value {
    fn from(s: String) -> Self {
        Self { 
            value: Some(value::Value::String(s)),
         }
    }
}

// 从&str 转化为 Value
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self { 
            value: Some(value::Value::String(s.into())) 
        }
    }
}

// 从 i64 转化为 Value
impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self {
            value: Some(value::Value::Integer(i)),
        }
    }
}

// 从 bool 转化为 Value
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self {
            value: Some(value::Value::Bool(b)),
        }
    }
}

// 从 f64 转化为 Value
impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self { 
            value: Some(value::Value::Float(f)) 
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Integer(i)) => Ok(i),
            _ => Err(KvError::ConvertError(v, "Integer")),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Float(f)) => Ok(f),
            _ => Err(KvError::ConvertError(v, "Float")),
        }
    }
}

impl TryFrom<Value> for Bytes {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Binary(b)) => Ok(b),
            _ => Err(KvError::ConvertError(v, "Binary")),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Bool(b)) => Ok(b),
            _ => Err(KvError::ConvertError(v, "Boolean")),
        }
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = KvError;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        let mut buf = Vec::with_capacity(v.encoded_len());
        v.encode(&mut buf)?;
        Ok(buf)
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = KvError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let msg = Value::decode(data)?;
        Ok(msg)
    }
}


// 从 Value 转化成 CommandResponse
impl From<Value> for CommandResponse {
    fn from(v: Value) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: vec![v],
            ..Default::default()
        }
    }
}

// 从 Vec<Kvpair> 转化成 CommandResponse
impl From<Vec<Kvpair>> for CommandResponse {
    fn from(v: Vec<Kvpair>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            pairs: v,
            ..Default::default()
        }
    }
}

// 从 KvError 转化成 CommandResponse
impl From<KvError> for CommandResponse {
    fn from(e: KvError) -> Self {
        let mut result = Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as _,
            message: e.to_string(),
            values: vec![],
            pairs: vec![],
        };

        match e {
            KvError::NotFound(_, _) => result.status = StatusCode::NOT_FOUND.as_u16() as _,
            KvError::InvalidCommand(_) => result.status = StatusCode::BAD_REQUEST.as_u16() as _,
            _ => {}
        }

        result
    }
}

// 从Vec<Value> 转化为 CommandResponse
impl From<Vec<Value>> for CommandResponse {
    fn from(value: Vec<Value>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: value,
            ..Default::default()
        }
    }
}



// 从 (String, Value) 转化为 Kvpair 
impl From<(String, Value)> for Kvpair {
    fn from(data: (String, Value)) -> Self {
        Kvpair::new(data.0, data.1)
    }
}
