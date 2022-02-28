pub mod abi;

use abi::{command_request::RequestData, *};
use http::StatusCode;
use crate::KvError;

impl CommandRequest {

    // 创建HSET命令
    pub fn new_hset(table: impl Into<String>, key: impl Into<String>, value: Value) -> Self {
        Self {
            request_data: Some(RequestData::Hset(Hset {
                table: table.into(),
                pair: Some(Kvpair::new(key, value)),
            }))
        }
    }
    // 创建HMSET命令
    pub fn new_hmset(table: impl Into<String>, kvpairs: Vec<Kvpair>) -> Self {
        //TODO: 修改hmset命令
        Self {
            request_data:Some(RequestData::Hmset(Hmset{
                table: table.into(),
                pairs: kvpairs,
            })),
        }
    }
    
    
    // 创建HGET命令
    pub fn new_hget(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hget(Hget{
                table:  table.into(),
                key: key.into(),
            })),
        }
    }

    // 创建HGETALL命令
    pub fn new_hgetall(table: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(Hgetall{
                table:  table.into(),
            })),
        }
    }

    
}


impl Kvpair {
    // 创建一个 kv pair
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
        }
    }
}

// 从String转化为Value
impl From<String> for Value {
    fn from(s: String) -> Self {
        Self {
            value: Some(value::Value::String(s)),
        }
    }
}

// 从&str转化为Value
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self {
            value: Some(value::Value::String(s.to_string())),
        }
    }
}

// 从i64转换成Value
impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self {
            value: Some(value::Value::Integer(i)),
        }
    }
}


/// 从 Value 转换成 CommandResponse
impl From<Value> for CommandResponse {
    fn from(v: Value) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: vec![v],
            ..Default::default()
        }
    }
}

/// 从 Vec<Kvpair> 转换成 CommandResponse
impl From<Vec<Kvpair>> for CommandResponse {
    fn from(v: Vec<Kvpair>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            pairs: v,
            ..Default::default()
        }
    }
}

/// 从 KvError 转换成 CommandResponse
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

