use std::collections::HashSet;
use std::io::{Cursor, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::fromts::util::{is_eof, read_file, read_fixed_string, read_short_string};

pub struct CommonDecoded {
    pub unk1: u8,
    pub unk_len: u32, // = header.section_length - 5
    pub unk_magic: String,
    pub length: u32, // width * height
    pub unk_len_dup: u32, // = header.section_length - 5
    /** @type {Uint8Array=} */
    pub data: Box<[u8]>,
    pub range: HashSet<u8>, // type yolo?
}

pub struct CommonDecoded2 {
    pub unk1: u8,
    pub unk_len: u32, // = header.section_length - 5
    pub unk_magic: String,
    pub length: u32, // width * height
    pub unk_len_dup: u32, // = header.section_length - 5
    /** @type {Uint16Array=} */
    pub data: Box<[u16]>,
    pub range: HashSet<u8>,
}

pub struct RawDecoded {
    pub   unk1: u8,
    pub   data_len: u32, // = header.section_length - 5
    pub   unk_magic: String,
    pub   length: u32, // width * height
    pub   unk_len_dup: u32, // = header.section_length - 5
    /** @type {Uint8Array=} */
    pub  data: Box<[u8]>,
    pub   range: HashSet<u8>,
}

pub fn common_decoding(view: &mut Cursor<Vec<u8>>) -> std::io::Result<CommonDecoded> {
    let mut content = CommonDecoded {
        unk1: view.read_u8()?,
        unk_len: view.read_u32::<LittleEndian>()?, // = header.section_length - 5
        unk_magic: read_fixed_string(view, 8),
        length: view.read_u32::<LittleEndian>()?, // width * height
        unk_len_dup: view.read_u32::<LittleEndian>()?, // = header.section_length - 5
        /** @type {Uint8Array=} */
        data: Box::new([]),
        range: HashSet::new(),
    };

    let mut count = 0;
    let mut data = vec![0u8; content.length as usize].into_boxed_slice();

    while !is_eof(&view) {
        let head = view.read_u8()?;

        if head > 0x80 {
            let value = view.read_u8()?;
            for i in 0..head - 0x80 {
                data[count] = value;
                count += 1;
            }
        } else {
            for i in 0..head {
                data[count] = view.read_u8()?;
                count += 1;
            }
        }
    }

    content.data = data;

    return Ok(content);
}

pub fn common_decoding2(view: &mut Cursor<Vec<u8>>) -> std::io::Result<CommonDecoded2> {
    let mut content = CommonDecoded2 {
        unk1: view.read_u8()?,
        unk_len: view.read_u32::<LittleEndian>()?, // = header.section_length - 5
        unk_magic: read_fixed_string(view, 8),
        length: view.read_u32::<LittleEndian>()? / 2, // width * height
        unk_len_dup: view.read_u32::<LittleEndian>()?, // = header.section_length - 5
        /** @type {Uint16Array=} */
        data: Box::new([]),
        range: HashSet::new(),
    };

    let mut count = 0;
    let mut data = vec![0u16; content.length as usize].into_boxed_slice();

    while is_eof(&view) {
        let head = view.read_u8().unwrap();

        if head > 0x80 {
            let value = view.read_u16::<LittleEndian>()?;
            for i in 0..head - 0x80 {
                data[count] = value;
                count += 1;
            }
        } else {
            for i in 0..head {
                data[count] = view.read_u16::<LittleEndian>()?;
                count += 1;
            }
        }
    }

    content.data = data;

    return Ok(content);
}

pub fn dictionary(view: &mut Cursor<Vec<u8>>) -> std::io::Result<Box<[String]>> {
    let len = view.read_u32::<LittleEndian>()?;

    let mut dictionary = Vec::new();
    for i in 0..len {
        let str = read_short_string(view);
        view.seek(SeekFrom::Current(1)).unwrap();
        dictionary.push(str);
    }

    return Ok(dictionary.into_boxed_slice());
}

pub fn raw(view: &mut Cursor<Vec<u8>>) -> std::io::Result<RawDecoded> {
    let mut content = RawDecoded {
        unk1: view.read_u8()?,
        data_len: view.read_u32::<LittleEndian>()?, // = header.section_length - 5
        unk_magic: read_fixed_string(view, 8),
        length: view.read_u32::<LittleEndian>()? / 2, // width * height
        unk_len_dup: view.read_u32::<LittleEndian>()?, // = header.section_length - 5
        /** @type {Uint8Array=} */
        data: Box::new([]),
        range: HashSet::new(),
    };

    let mut data = vec![0u8; content.length as usize].into_boxed_slice();
    view.read_exact(data.as_mut()).unwrap();
    content.data = data;

    return Ok(content);
}

pub fn hoixzisl_parse(view: &mut Cursor<Vec<u8>>) -> std::io::Result<HoixzislData> {
    return Ok(HoixzislData {
        width: view.read_u32::<LittleEndian>()?,
        height: view.read_u32::<LittleEndian>()?,
    });
}

pub struct HoixzislData {
    pub width: u32,
    pub height: u32,
}
