mod decoding_functions;
mod decodings;

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use web_sys::Blob;
use crate::fromts::map::decodings::{hoix1tme, hoix2tme, hoix3tme, hoix4tme, hoixalme, hoixapme, hoixbpme, hoixdlae, hoixdpae, hoixdtae, hoixehml, hoixrbme, hoixtlml, hoixvlml, hoixzisl, MapSectionName};
use crate::fromts::util::{is_eof, read_bytes, read_file, read_fixed_string, SequentialDataView};

pub struct CulturesMapData {
    width: u32,
    height: u32,
    elevation: Box<[u8]>,
    lighting: Box<[u8]>,
    tiles_index: Box<[String]>,
    tiles_a: Box<[u16]>,
    tiles_b: Box<[u16]>,
    transitions_index: Box<[String]>,
    trans_a1: Box<[u8]>,
    trans_b1: Box<[u8]>,
    trans_a2: Box<[u8]>,
    trans_b2: Box<[u8]>,

    landscape_index: Box<[String]>,
    landscape_levels: Box<[u8]>,
    landscape_types: Box<[u8]>,
    landscape_job_types: Box<[u8]>,
}

fn read_header(view: &mut SequentialDataView) -> std::io::Result<Header> {
    Ok(Header {
        // map section name, "hoix"
        tag: read_fixed_string(view, 8),
        unk1: view.read_u32::<LittleEndian>()?,
        section_length: view.read_u32::<LittleEndian>()?,
        unk2: view.read_u32::<LittleEndian>()?,
        unk3: view.read_u32::<LittleEndian>()?,
        unk4: view.read_u32::<LittleEndian>()?,
        unk5: view.read_u32::<LittleEndian>()?,
    })
}

struct Header {
    tag: String,
    unk1: u32,
    section_length: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
}

macro_rules! decode_hoix {
    ($func:ident, $from:ident) => {
        $func (&mut Cursor::new($from.remove(&MapSectionName::$func).unwrap())).unwrap()
    }
}


async fn read_map_data(blob: Blob) -> std::io::Result<CulturesMapData> {

    let mut section_headers: HashMap<MapSectionName, Header> = HashMap::new();
    let mut section_datas: HashMap<MapSectionName, Vec<u8>> = HashMap::new();

    let mut cursor = Cursor::new(read_file(blob).await.to_vec());

    loop {
        let mut buf = [0u8; 0x20];
        cursor.read_exact(&mut buf)?;
        let header = read_header(&mut cursor).unwrap();

        let section_name = MapSectionName::from_str(header.tag.as_str());
        if let Some(section_name) = section_name {

            let mut data =  vec![0u8; header.section_length as usize];
            cursor.read_exact(data.as_mut_slice()).unwrap();
            section_datas.insert(section_name, data);
            section_headers.insert(section_name, header);
        } else {
            cursor.seek(SeekFrom::Current(header.section_length as i64)).unwrap();
        }

        if is_eof(&cursor) {
            break;
        }
    }


    let hoixzisl = decode_hoix!(hoixzisl, section_datas);

    return Ok(CulturesMapData {
        width: hoixzisl.width,
        height: hoixzisl.height,
        elevation: decode_hoix!(hoixehml, section_datas).data,
        lighting: decode_hoix!(hoixrbme, section_datas).data,

        tiles_index: decode_hoix!(hoixdpae, section_datas),
        tiles_a: decode_hoix!(hoixapme, section_datas).data,
        tiles_b: decode_hoix!(hoixbpme, section_datas).data,

        transitions_index: decode_hoix!(hoixdtae, section_datas),
        trans_a1: decode_hoix!(hoix1tme, section_datas).data,
        trans_b1: decode_hoix!(hoix2tme, section_datas).data,
        trans_a2: decode_hoix!(hoix3tme, section_datas).data,
        trans_b2: decode_hoix!(hoix4tme, section_datas).data,

        landscape_index: decode_hoix!(hoixdlae, section_datas),

        landscape_job_types: decode_hoix!(hoixtlml, section_datas).data,
        landscape_types: decode_hoix!(hoixalme, section_datas).data,
        landscape_levels: decode_hoix!(hoixvlml, section_datas).data,
    });
}
