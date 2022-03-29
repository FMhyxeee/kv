use crate::{CommandService, Hget, Hgetall, Hset, KvError, Value, Hmget, Hdel, Hmdel, Hexist, Hmexist, Hmset};

impl CommandService for Hget {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        match store.get(&self.table, &self.key) {
            Ok(Some(v)) => v.into(),
            Ok(None) => KvError::NotFound(self.table, self.key).into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hgetall {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        match store.get_all(&self.table) {
            Ok(v) => v.into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hset {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        match self.pair {
            Some(v) => match store.set(&self.table, v.key, v.value.unwrap_or_default()) {
            Ok(Some(v)) => v.into(),
            Ok(None) => Value::default().into(),
            Err(e) => e.into(),
            },
            None => Value::default().into(),
        }
    }
}

impl CommandService for Hmset {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        let pairs = self.pairs;
        let table = self.table;
        pairs
            .into_iter()
            .map(|pair| {
                let result = store.set(&table, pair.key, pair.value.unwrap_or_default());
                match result {
                    Ok(Some(v)) => v,
                    _ => Value::default(),
                }
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl CommandService for Hmget {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        self.keys
            .iter()
            .map(|key| match store.get(&self.table, key) {
                Ok(Some(v)) => v,
                _ => Value::default(),
            })
            .collect::<Vec<_>>()
            .into()
        
    }
}

impl CommandService for Hdel {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        match store.del(&self.table, &self.key) {
            Ok(Some(v)) => v.into(),
            Ok(None) => Value::default().into(),
            Err(e) => e.into()
        }
    }
}

impl CommandService for Hmdel {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        self.keys
            .iter()
            .map(|key| match store.del(&self.table, key) {
                Ok(Some(v)) => v,
                _ => Value::default(),
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl CommandService for Hexist {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        match store.contains(&self.table, &self.key) {
            Ok(v) => Value::from(v).into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hmexist {
    fn execute(self, store: &impl crate::Storage) -> crate::CommandResponse {
        self.keys
            .iter()
            .map(|key| match store.contains(&self.table, key) {
                Ok(v) => v.into(),
                Err(_) => Value::default(),
            })
            .collect::<Vec<Value>>()
            .into()
    }
}


#[cfg(test)]
mod tests{
    use crate::{CommandRequest, CommandResponse, Storage, command_request::RequestData, Kvpair, memory::MemTable};

    use super::*;
    
    #[test]
    fn hset_should_work() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hset("t1", "hello", "world".into());
        let res = dispatch(cmd.clone(), &store);
        assert_res_ok(res, &[Value::default()], &[]);

        let res = dispatch(cmd, &store);
        assert_res_ok(res, &["world".into()], &[]);
    }

    #[test]
    fn hget_should_work() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hset("score", "u1", 10.into());
        dispatch(cmd, &store);
        let cmd = CommandRequest::new_hget("score", "u1");
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &[10.into()], &[]);
    }

    #[test]
    fn hget_with_non_exist_key_should_return_404() {
        let store = MemTable::new();
        let cmd = CommandRequest::new_hget("score", "u1");
        let res = dispatch(cmd, &store);
        assert_res_error(res, 404, "Not found");
    }

    #[test]
    fn hgetall_should_work() {
        let store = MemTable::new();
        let cmds = vec![
            CommandRequest::new_hset("score", "u1", 10.into()),
            CommandRequest::new_hset("score", "u2", 8.into()),
            CommandRequest::new_hset("score", "u3", 11.into()),
            CommandRequest::new_hset("score", "u1", 6.into()),
        ];
        for cmd in cmds {
            dispatch(cmd, &store);
        }

        let cmd = CommandRequest::new_hgetall("score");
        let res = dispatch(cmd, &store);
        let pairs = &[
            Kvpair::new("u1", 6.into()),
            Kvpair::new("u2", 8.into()),
            Kvpair::new("u3", 11.into()),
        ];
        assert_res_ok(res, &[], pairs);
    }

    #[test]
    fn hmset_should_work() {
        let store = MemTable::new();
        set_key_pairs("t1", vec![("u1", "world")], &store);
        let pairs = vec![
            Kvpair::new("u1", 10.1.into()),
            Kvpair::new("u2", 8.1.into()),
        ];
        let cmd = CommandRequest::new_hmset("t1", pairs);
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &["world".into(), Value::default()], &[]);
    }

    #[test]
    fn hdel_should_work() {
        let store = MemTable::new();
        set_key_pairs("t1", vec![("u1", "v1")], &store);
        let cmd = CommandRequest::new_hdel("t1", "u2");
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &[Value::default()], &[]);

        let cmd = CommandRequest::new_hdel("t1", "u1");
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &["v1".into()], &[]);
    }

    #[test]
    fn hmdel_should_work() {
        let store = MemTable::new();
        set_key_pairs("t1", vec![("u1", "v1"), ("u2", "v2")], &store);

        let cmd = CommandRequest::new_hmdel("t1", vec!["u1".into(), "u3".into()]);
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &["v1".into(), Value::default()], &[]);
    }

    #[test]
    fn hexist_should_work() {
        let store = MemTable::new();
        set_key_pairs("t1", vec![("u1", "v1")], &store);
        let cmd = CommandRequest::new_hexist("t1", "u2");
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &[false.into()], &[]);

        let cmd = CommandRequest::new_hexist("t1", "u1");
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &[true.into()], &[]);
    }

    #[test]
    fn hmexist_should_work() {
        let store = MemTable::new();
        set_key_pairs("t1", vec![("u1", "v1"), ("u2", "v2")], &store);

        let cmd = CommandRequest::new_hmexist("t1", vec!["u1".into(), "u3".into()]);
        let res = dispatch(cmd, &store);
        assert_res_ok(res, &[true.into(), false.into()], &[]);
    }

    fn set_key_pairs<T: Into<Value>>(table: &str, pairs: Vec<(&str, T)>, store: &impl Storage) {
        pairs
            .into_iter()
            .map(|(k, v)| CommandRequest::new_hset(table, k, v.into()))
            .for_each(|cmd| {
                dispatch(cmd, store);
            });
    }




    // 从 Request 中得到 Response
    fn dispatch(cmd: CommandRequest, store: &impl Storage) -> CommandResponse {
        match cmd.request_data.unwrap() {
            RequestData::Hget(v) => v.execute(store),
            RequestData::Hgetall(v) => v.execute(store),
            RequestData::Hset(v) => v.execute(store),
            RequestData::Hmget(v) => v.execute(store),
            RequestData::Hmset(v) => v.execute(store),
            RequestData::Hdel(v) => v.execute(store),
            RequestData::Hmdel(v) => v.execute(store),
            RequestData::Hexist(v) => v.execute(store),
            RequestData::Hmexist(v) => v.execute(store),
            
        }
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
    fn assert_res_error(res: CommandResponse, code: u32, msg: &str) {
        assert_eq!(res.status, code);
        assert!(res.message.contains(msg));
        assert_eq!(res.values, &[]);
        assert_eq!(res.pairs, &[]);
    }
    
}

