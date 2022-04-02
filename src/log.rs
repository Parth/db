use crate::errors::ReadError;
use crate::{Key, Value};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum TableEvent<K: Key, V: Value> {
    Insert(K, V),
    Delete(K),
}

pub trait LogFormat<K: Key, V: Value> {
    type LogEntry: Serialize;

    fn insert(k: K, v: V) -> Self::LogEntry;
    fn delete(k: K) -> Self::LogEntry;
}

pub trait Reader<OnDisk: DeserializeOwned, InMemory> {
    fn open_file(path: &str) -> Result<File, ReadError> {
        OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(path)
            .map_err(|err| {
                ReadError::OsError(
                    format!(
                        "While opening {} during startup, we received an error from the OS: {}",
                        path, err
                    ),
                    err,
                )
            })
    }

    fn parse_log(file: &mut File) -> Result<(Vec<OnDisk>, bool), ReadError> {
        let mut buffer: Vec<u8> = Vec::new();
        file
            .read_to_end(&mut buffer)
            .map_err(|err| ReadError::OsError(format!("After having opened the db file successfully, we were unable to read it into a buffer: {}", err), err))?;

        if buffer.is_empty() {
            return Ok((vec![], false));
        }

        let mut log_entries = vec![];

        let mut index = 0;
        while index < buffer.len() {
            if buffer.len() < index + 4 {
                return Ok((log_entries, true));
            }

            let size = u32::from_be_bytes(
                buffer[index..index + 4]
                    .try_into()
                    .expect("slice with incorrect length"),
            );

            index += 4;

            // This cast should be fine on both 32 bit and 64 bit systems, on a less-than 32 bit
            // system, with a size value larger than usize::Max this will overflow or panic
            if buffer.len() < index + (size as usize) {
                return Ok((log_entries, true));
            }

            let data = &buffer[index..(size as usize)];

            let entry: OnDisk = bincode::deserialize(data).map_err(|err| ReadError::LogParseError(format!(
                "While parsing the log we were looking for {} bytes for the next entry, we found \
                that many bytes, but they failed to deserialize into the type {}. This could \
                indicate a Schema Data mismatch, or a corrupted log. There are {} bytes left in the \
                log after this entry. Bincode error: {}",
                size,
                std::any::type_name::<OnDisk>(),
                (buffer.len() as i64) - ((index + (size as usize)) as i64),
                err
            )))?;
            log_entries.push(entry);

            index += size as usize;
        }

        Ok((log_entries, false))
    }

    fn incomplete_write(&self) -> bool;

    fn init(path: &str) -> Result<InMemory, ReadError>;
}

#[derive(Clone)]
pub struct Writer {
    file: Arc<Mutex<File>>,
}

impl Writer {
    pub fn init(file: File) -> Self {
        let file = Arc::new(Mutex::new(file));

        Self { file }
    }

    pub fn append(&self, data: &[u8]) {
        self.file.lock().unwrap().write_all(data).unwrap();
    }
}
