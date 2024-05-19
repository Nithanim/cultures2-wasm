use std::io::Cursor;
use byteorder::ReadBytesExt;
use web_sys::Blob;
use crate::fromts::util::{read_bytes, read_file};

pub struct FileAbstraction {
    blob: Blob,
    size: u64,
}

impl FileAbstraction {
    pub async fn new(blob: Blob) -> Self {
        Self {
            blob,
            size: blob.size() as u64,
        }
    }

    pub async fn get(&self, pos: u64, len: u64) -> Box<[u8]> {
        if pos + len > self.size {
            panic!()
        }
        let slice_blob = self.blob.slice_with_f64_and_f64(pos as f64, (pos + len) as f64).unwrap();
        read_file(slice_blob).await.to_vec().into_boxed_slice()
    }
    pub async fn get_as_cursor(&self) -> Cursor<Box<[u8]>> {
        Cursor::new(self.get(0, self.size).await)
    }

    pub async fn get_as_cursor_partial(&self, pos: u64, len: u64) -> Cursor<Box<[u8]>> {
        Cursor::new(self.get(pos, len).await)
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn slice(&self, pos: u64, len: u64) -> FileAbstraction {
        FileAbstraction {
            size: len,
            blob: self.blob.slice_with_f64_and_f64(pos as f64, (pos + len) as f64).unwrap(),
        }
    }
}
