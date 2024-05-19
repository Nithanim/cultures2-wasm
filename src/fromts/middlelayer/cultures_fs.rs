use std::collections::HashMap;
use std::io;
use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};
use web_sys::Blob;

use crate::fromts::middlelayer::file_interface::FileAbstraction;
use crate::fromts::util::read_normal_string;

pub struct FSHeader {
    pub version: u32,
    pub num_dirs: u32,
    pub num_files: u32,
}

pub struct DirInfo {
    path: String,
    depth: u32,
}

pub struct FileInfo {
    path: String,
    offset: u32,
    length: u32,
}


async fn load_fs(datafile: Blob) -> io::Result<CulturesFS> {
    let fa = FileAbstraction::new(datafile).await;
    let mut view = fa.get_as_cursor_partial(0, 250 * 1024).await;

    let header = getHeader(&mut view).await?;
    let dirs = getDirs(header.num_dirs, &mut view).await;
    let files = getFiles(header.num_files, &mut view).await;

    return CulturesFS::new(fa, dirs, files);
}


async fn getHeader(c: &mut Cursor<Box<[u8]>>) -> io::Result<FSHeader> {
    //let mut c = blob.get_as_cursor(0, 3 * 4).await;
    return Ok(FSHeader {
        version: c.read_u32()?,
        num_dirs: c.read_u32()?,
        num_files: c.read_u32()?,
    });
}


async fn getFiles(n: u32, view: &mut Cursor<Box<[u8]>>) -> Box<[FileInfo]> {
    let mut files = Vec::new();
    for _ in 0..n {
        files.push(FileInfo {
            path: read_normal_string(view),
            offset: view.read_u32::<LittleEndian>().unwrap(),
            length: view.read_u32::<LittleEndian>().unwrap(),
        });
    }

    return files.into_boxed_slice();
}

async fn getDirs(n: u32, view: &mut Cursor<Box<[u8]>>) -> Box<[DirInfo]> {
    let mut dirs = Vec::new();
    for _ in 0..n {
        dirs.push(DirInfo {
            path: read_normal_string(view),
            depth: view.read_u32::<LittleEndian>().unwrap(),
        });
    }
    return dirs.into_boxed_slice();
}


pub struct CulturesFS<'a> {
    datafile: FileAbstraction,
    files: HashMap<String, FileInfo>,
}

impl<'a> CulturesFS<'a> {
    pub async fn new(fa: FileAbstraction, dirs: Box<[DirInfo]>, files: Box<[FileInfo]>) -> Self {
        let f: HashMap<String, FileInfo> = files.iter().map(|e| (&e.path, e)).collect();

        Self {
            datafile: fa,
            files: f,
        }
    }

    pub fn ls(&self) -> &'a HashMap<String, FileInfo> {
        &self.files
    }

    pub fn stats(&self, path: String) -> FileInfo {
        match self.files.get(path.toLowerCase()) {
            Ok(o) => return o,
            Err(_) => panic!(format!("Path not found: {}", path))
        }
    }

    pub fn open(&self, path: String) -> FileAbstraction {
        let fi = self.stats(path);

        return self.datafile.slice(fi.offset as u64, fi.length as u64);
    }
}
