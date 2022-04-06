use crate::errors::Error;
use crate::{Key, Value};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum LogItems<S> {
    Single(S),
    Batch(Vec<S>),
}

pub trait SchemaEvent<K: Key, V: Value> {
    type LogEntry: Serialize;

    fn insert(k: K, v: V) -> Self::LogEntry;
    fn delete(k: K) -> Self::LogEntry;
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum TableEvent<K: Key, V: Value> {
    Insert(K, V),
    Delete(K),
}

pub trait Reader<OnDisk: DeserializeOwned, InMemory> {
    fn open_file<P>(dir: P) -> Result<File, Error>
    where
        P: AsRef<Path>,
    {
        let mut path = PathBuf::new();
        path.push(dir);

        fs::create_dir_all(&path).unwrap();

        let schema_name = std::any::type_name::<InMemory>().replace(':', "_");

        path.push(schema_name);

        OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|err| {
                Error::OsError(
                    format!(
                        "While opening {:?} during startup, we received an error from the OS: {}",
                        path, err
                    ),
                    err,
                )
            })
    }

    fn parse_log(file: &mut File) -> Result<(Vec<OnDisk>, bool), Error> {
        let mut buffer: Vec<u8> = Vec::new();
        file
            .read_to_end(&mut buffer)
            .map_err(|err| Error::OsError(format!("After having opened the db file successfully, we were unable to read it into a buffer: {}", err), err))?;

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

            let data = &buffer[index..index + (size as usize)];

            let parsed: LogItems<OnDisk> = bincode::deserialize(data).map_err(|err| Error::LogParseError(format!(
                "While parsing the log we were looking for {} bytes for the next entry, we found \
                that many bytes, but they failed to deserialize into the type {}. This could \
                indicate a Schema Data mismatch, or a corrupted log. There are {} bytes left in the \
                log after this entry. Bincode error: {}",
                size,
                std::any::type_name::<OnDisk>(),
                (buffer.len() as i64) - ((index + (size as usize)) as i64),
                err
            ), err))?;
            match parsed {
                LogItems::Single(entry) => log_entries.push(entry),
                LogItems::Batch(entries) => log_entries.extend(entries),
            }

            index += size as usize;
        }

        Ok((log_entries, false))
    }

    fn incomplete_write(&self) -> bool;

    fn init<P: AsRef<Path>>(path: P) -> Result<InMemory, Error>;
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

    pub fn append<S: Serialize>(&self, data: &S) -> Result<(), Error> {
        let mut data = bincode::serialize(&LogItems::Single(data))
            .map_err(|err| Error::serialize(std::any::type_name::<LogItems<S>>(), err))?;
        let size = data.len() as u32;

        let mut to_write = size.to_be_bytes().to_vec();
        to_write.append(&mut data);
        self.file
            .lock()
            .map_err(|err| Error::LockError(format!("Writer lock poisoned, this suggest an internal, unexpected, database error. Error: {}", err)))?
            .write_all(&to_write)
            .map_err(|err| Error::OsError(format!("Failed to append {} bytes to the log, error: {}", to_write.len(), err), err))?;

        Ok(())
    }

    pub fn append_all<S: Serialize>(&self, data: Vec<S>) -> Result<(), Error> {
        let mut data = bincode::serialize(&LogItems::Batch(data))
            .map_err(|err| Error::serialize(std::any::type_name::<LogItems<S>>(), err))?;
        let size = data.len() as u32;

        let mut to_write = size.to_be_bytes().to_vec();
        to_write.append(&mut data);
        self.file
            .lock()
            .map_err(|err| Error::LockError(format!("Writer lock poisoned, this suggest an internal, unexpected, database error. Error: {}", err)))?
            .write_all(&to_write)
            .map_err(|err| Error::OsError(format!("Failed to append {} bytes to the log for a transaction, error: {}", to_write.len(), err), err))?;

        Ok(())
    }
}
