use crate::{Key, Value};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};

#[derive(serde::Serialize)]
pub enum TableEvent<K: Key, V: Value> {
    Insert(K, V),
    Delete(K),
}

pub trait Logger<K: Key, V: Value> {
    type Output: Serialize;

    fn insert(k: K, v: V) -> Self::Output;
    fn delete(k: K) -> Self::Output;
}

pub trait Init<OnDisk, InMemory> {
    fn read_from_disk(path: &str) -> Vec<OnDisk> {
        todo!()
    }

    fn init(path: &str) -> InMemory;
}

pub struct Writer {
    file: Arc<Mutex<File>>,
}

impl Writer {
    pub fn init(path: &str) -> Self {
        let append_only = OpenOptions::new().append(true).open(path).unwrap();

        let file = Arc::new(Mutex::new(append_only));

        Self { file }
    }

    pub fn append(&self, data: &[u8]) {
        self.file.lock().unwrap().write_all(data).unwrap();
    }
}
