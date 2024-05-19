use std::io::{Cursor, Read, Seek, SeekFrom};
use std::ops::{Range, RangeInclusive};
use byteorder::{LittleEndian, ReadBytesExt};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{Blob, FileReader, ImageData, js_sys};
use web_sys::js_sys::{ArrayBuffer, Uint8Array, Uint8ClampedArray};
use crate::fromts::util::{read_bytes, read_file};

pub(crate) type JsImageData = web_sys::ImageData;

pub struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

pub struct Image {
    width: u32,
    height: u32,

}

fn read_pixels(cursor: &mut Cursor<Vec<u8>>, width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height) as usize];
    let mut i = 0;

    while i < width * height {
        let mut val = cursor.read_u8().unwrap();
        let mut len = 1;

        if val > 192 {
            len = val - 192;
            val = cursor.read_u8().unwrap();
        }

        for u in (0 + 1..len + 1).rev() {
            pixels[u as usize] = val;
            i += 1;
        }
    }

    return pixels;
}

pub struct Pcx {
    pub width: u32,
    pub height: u32,
    pub data: Box<[u8]>,
}

pub async fn pcx_read(blob: Blob, mask: Option<Blob>) -> Pcx {
    let buf = read_file(blob).await;
    let mut cursor = Cursor::new(buf.to_vec());
    let header = read_header(&mut cursor);
    let width = header.x1 - header.x0 + 1;
    let height = header.y1 - header.y0 + 1;

    let pixels = read_pixels(&mut cursor, width as u32, height as u32);

    let extended_palette_indicator = cursor.read_u8().unwrap();
    let palette = if extended_palette_indicator == 0x0C {
        read_palette(&mut cursor)
    } else {
        panic!("Palette could not be found.");
    };

    let mut alpha = vec![0xFFu8; (width * height) as usize];
    if let Some(mask) = mask {
        let mask_buf = read_file(mask).await.to_vec();
        let mut mask_view = Cursor::new(mask_buf);
        mask_view.seek(SeekFrom::Current(0x80i64)).unwrap();
        alpha = read_pixels(&mut mask_view, width as u32, height as u32);
    }

    let mut img_data = vec![0u8; (width * height) as usize];

    for i in 0..(width * height) as usize {
        img_data[4 * i + 0] = palette[pixels[i] as usize].red;
        img_data[4 * i + 1] = palette[pixels[i] as usize].green;
        img_data[4 * i + 2] = palette[pixels[i] as usize].blue;
        img_data[4 * i + 3] = alpha[i];
    };

    Pcx {
        width: width as u32,
        height: height as u32,
        data: img_data.into_boxed_slice(),
    }
}

type Palette = [RGBColor; 256];

fn read_palette(view: &mut Cursor<Vec<u8>>) -> Palette {
    let mut palette = [RGBColor { red: 0, green: 0, blue: 0 }; 256];

    for i in 0..256 {
        palette[i] = RGBColor {
            red: view.read_u8().unwrap(),
            green: view.read_u8().unwrap(),
            blue: view.read_u8().unwrap(),
        };
    }

    return palette;
}

#[derive(Copy, Clone)]
struct RGBColor {
    red: u8,
    green: u8,
    blue: u8,
}

fn read_header(c: &mut Cursor<Vec<u8>>) -> Header {
    read_header_(c).unwrap()
}

fn read_header_(c: &mut Cursor<Vec<u8>>) -> std::io::Result<Header> {
    return Ok(Header {
        magic: c.read_u8()?,
        version: c.read_u8()?,
        encoding_method: c.read_u8()?,
        bits_per_pixel: c.read_u8()?,
        x0: c.read_u16::<LittleEndian>()?,
        y0: c.read_u16::<LittleEndian>()?,
        x1: c.read_u16::<LittleEndian>()?,
        y1: c.read_u16::<LittleEndian>()?,
        h_dpi: c.read_u16::<LittleEndian>()?,
        v_dpi: c.read_u16::<LittleEndian>()?,
        palette: read_bytes(c, 48)?,
        reserved: c.read_u8()?,
        color_planes: c.read_u8()?,
        bytes_per_color_plane: c.read_u16::<LittleEndian>()?,
        palette_type: c.read_u16::<LittleEndian>()?,
        h_res: c.read_u16::<LittleEndian>()?,
        v_res: c.read_u16::<LittleEndian>()?,
        reserved_block: read_bytes(c, 54)?,
    });
}

struct Header {
    magic: u8,
    version: u8,
    encoding_method: u8,
    bits_per_pixel: u8,
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
    h_dpi: u16,
    v_dpi: u16,
    palette: [u8; 48],
    reserved: u8,
    color_planes: u8,
    bytes_per_color_plane: u16,
    palette_type: u16,
    h_res: u16,
    v_res: u16,
    reserved_block: [u8; 54],
}

