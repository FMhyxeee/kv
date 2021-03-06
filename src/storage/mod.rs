pub mod memory;
pub mod sleddb;

use crate::{KvError, Kvpair, Value};

/// 对存储的抽象， 我们其实并不关心数据存在哪里，只是定义这么和外界存储打交道
/// 就是经典CRUD
pub trait Storage {
    // Retrieve
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    // Contains
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError>;
    // Create
    fn set(&self, table: &str, key: String, value: Value) -> Result<Option<Value>, KvError>;
    // Delete
    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    // Get all
    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError>;
    // Get iterator
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, KvError>;
}


pub struct StorageIter<T> {
    data: T,
}

impl<T> StorageIter<T> {
    pub fn new(data: T) -> Self {
        Self{data}
    }
}

impl<T> Iterator for StorageIter<T>
where 
    T: Iterator,
    T::Item: Into<Kvpair>,
{
    type Item = Kvpair;

    fn next(&mut self) -> Option<Self::Item> {
        self.data.next().map(|v| v.into())
    }
}
