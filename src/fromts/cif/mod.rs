mod parsed;
pub mod definitions;
#[cfg(test)]
mod tests;

use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};
use std::io::SeekFrom::Current;
use byteorder::{LittleEndian, ReadBytesExt};
use web_sys::Blob;
use regex::{Captures, Regex};
use crate::fromts::cif::definitions::IniCategory;
use crate::fromts::cif::parsed::reduce_sections;
use crate::fromts::middlelayer::file_interface::FileAbstraction;
use crate::fromts::util::{read_file, read_zero_terminated_string};

#[allow(non_snake_case)]
fn decode_cif(data: &mut [u8]) {
    let mut B = 0;
    let mut C = 71;
    let mut D = 126;

    for d in data {
        B = *d - 1;
        B = B ^ C;
        C += D;
        D += 33;

        *d = B;
    }
}

struct Section {
    name: String,
    items: Vec<Item>,
}

struct Item {
    key: String,
    value: String,
}

fn read_3fd_cif(view: &mut Cursor<Box<[u8]>>) -> std::io::Result<Vec<IniCategory> > {
    view.seek(SeekFrom::Current(12)).unwrap();
    let header = Header {
        NrOfEntries: view.read_u32::<LittleEndian>()?,
        NrOfEntries_dup1: view.read_u32::<LittleEndian>()?,
        NrOfEntries_dup2: view.read_u32::<LittleEndian>()?,
        SizeOfTextTable: view.read_u32::<LittleEndian>()?,
        Unk2: view.read_u32::<LittleEndian>()?,
        Unk3: view.read_u32::<LittleEndian>()?,
        SizeOfIndexTable: view.read_u32::<LittleEndian>()?,
    };

    let mut index_table = vec![0u8; header.SizeOfIndexTable as usize];
    view.read_exact(index_table.as_mut_slice()).unwrap();
    decode_cif(index_table.as_mut_slice());

    view.seek(Current(1 + 4 + 4 + 4)).unwrap();

    let mut text_table = vec![0u8; header.SizeOfTextTable as usize];
    view.read_exact(text_table.as_mut_slice()).unwrap();
    decode_cif(text_table.as_mut_slice());

    let mut sections: Vec<Section> = Vec::new();
    for _ in 0..header.NrOfEntries {
        let level = view.read_u8()?;
        if level == 1 {
            let name = read_zero_terminated_string(view);
            sections.push(Section {
                name,
                items: Vec::new(),
            });
        } else {
            if let Some(line) = parse(read_zero_terminated_string(view)) {
                sections.last_mut().unwrap().items.push(line);
            }
        }
    }

   Ok(reduce_sections(sections))
}

fn parse(line: String) -> Option<Item> {
    let regex = Regex::new("^([a-zA-Z0-9]+)((?:(?: \"[^\"]+\")|(?: [0-9]+))+)$").unwrap();

    match regex.captures(line.as_str()) {
        None => None,
        Some(data) => Some(Item {
            key: data.get(1).unwrap().as_str().to_string(),
            value: data.get(2).unwrap().as_str().to_string().trim().to_string(),
        })
    }
}


pub async fn read_cif(blob: FileAbstraction) -> std::io::Result<Vec<IniCategory> > {
    let mut view = blob.get_as_cursor().await;

    let magic = view.read_u16::<LittleEndian>()?;
    match magic {
        0x03FD => read_3fd_cif(&mut view),
        _ => panic!("Unknown CIF file!")
    }
}

#[allow(non_snake_case)]
struct Header {
    NrOfEntries: u32,
    NrOfEntries_dup1: u32,
    NrOfEntries_dup2: u32,
    SizeOfTextTable: u32,
    Unk2: u32,
    Unk3: u32,
    SizeOfIndexTable: u32,
}