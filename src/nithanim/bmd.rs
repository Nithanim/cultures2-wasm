use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

type Palette = Box<[u8]>;

fn read_bmd(cursor: &mut Cursor<Box<[u8]>>, palette: &Palette, t4i: Type4AlphaInterpretation) -> Vec<ExtractedFrame> {
    let raw_bmd_file = read_bmd_file_raw(cursor);
    read_bmd_(&raw_bmd_file, palette, t4i)
}


fn read_bmd_(raw_bmd_file: &RawBmdFile, palette: &Palette, t4i: Type4AlphaInterpretation) -> Vec<ExtractedFrame> {
    raw_bmd_file.frame_infos.iter().map(|fi| extract_frame(raw_bmd_file, fi, palette, t4i)).collect()
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum Type4AlphaInterpretation {
    Alpha,
    Ignore,
}

// TODO Generify to decode without palette to allow easy palette swapping
fn extract_frame(raw_bmd_file: &RawBmdFile, bmd_frame_info: &BmdFrameInfo, palette: &Palette, t4i: Type4AlphaInterpretation) -> ExtractedFrame {
    let frame_type = bmd_frame_info.type_;
    let frame_start = bmd_frame_info.off;
    let frame_count = bmd_frame_info.len;
    let width = bmd_frame_info.width;

    let pixels = &raw_bmd_file.pixels;
    let mut pixel_pointer = raw_bmd_file.row_infos.get(frame_start as usize).unwrap().offset;

    let height = frame_count + 1;
    let data = vec![width * height].into_boxed_slice();

    for row_number in 0..frame_count {
        let row = raw_bmd_file.row_infos.get((row_number + frame_start) as usize).unwrap();
        if is_empty(row) {
            continue;
        }

        let indent = row.indent;
        let mut i = indent;

        let pixel_block_length = pixels[pixel_pointer] & 0xFF;
        pixel_pointer += 1;

        while pixel_block_length != 0 {
            if pixel_block_length < 0x80 {
                let mut z = 0;
                loop {
                    if !(z < pixel_block_length) {
                        break;
                    }
                    let color: Color;
                    let alpha: u8;
                    if frame_type == BmdFrameType::Extended {
                        color = get_from_palette(palette, pixels[pixel_pointer] & 0xFF);
                        pixel_pointer += 1;
                        if t4i == Type4AlphaInterpretation::Alpha {
                            alpha = pixels[pixel_pointer] & 0xFF;
                            pixel_pointer += 1;
                        } else {
                            alpha = 0xFF;
                            pixel_pointer += 1;
                        }
                    } else if frame_type == BmdFrameType::Normal {
                        alpha = 0xFF;
                        color = get_from_palette(palette, pixels[pixel_pointer] & 0xFF);
                        pixel_pointer += 1;
                    } else if frame_type == BmdFrameType::Shadow {
                        alpha = 0x80; // Or 0x50?
                        color = Color {
                            r: 0,
                            g: 0,
                            b: 0,
                        };
                    } else {
                        // If we land here, we either have an unknown BMD format...
                        // Or something is broken. Either crash or just paint a red image instead?
                        alpha = 0xFF;
                        color = Color {
                            r: 0xFF,
                            g: 0,
                            b: 0,
                        }
                    }

                    {
                        // bmp.setPixel(i++, row, color | (alpha << 3 * 8));
                        let pos = i * 4;
                        data[pos + 0] = alpha;
                        data[pos + 1] = color.r;
                        data[pos + 2] = color.g;
                        data[pos + 3] = color.b;

                        i += 1;
                    }


                    z += 1;
                }
            } else {
                i += (pixel_block_length - 0x80);
            }
            pixel_block_length = pixels[pixel_pointer] & 0xFF;
            pixel_pointer += 1;
        }
    }

    ExtractedFrame {
        width: width as u32,
        height: (frame_count + 1) as u32,
        data,
    }
}

fn is_empty(row: &BmdFrameRow) -> bool {
    return row.offset == 0b00111111_11111111_11111111 && row.indent == 0b00000011_11111111;
}

struct ExtractedFrame {
    width: u32,
    height: u32,
    data: Box<[u8]>,
}

fn get_from_palette(palette: &Palette, idx: u32) -> Color {
    let pointer = idx * 3;
    Color {
        r: palette[pointer] & 0xFF,
        g: palette[pointer + 1] & 0xFF,
        b: palette[pointer + 2] & 0xFF,
    }
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
}


pub fn read_bmd_file_raw(cursor: &mut Cursor<Box<[u8]>>) -> RawBmdFile {
    let header = read_bmd_file_header(cursor);

    let frame_infos = read_frame_infos(cursor);
    let pixels = read_pixels(cursor);
    let row_infos = read_frame_rows(cursor);

    RawBmdFile {
        header,
        frame_infos,
        pixels,
        row_infos,
    }
}

struct RawBmdFile {
    header: BmdFileHeader,
    frame_infos: Vec<BmdFrameInfo>,
    pixels: Box<[u8]>,
    row_infos: Vec<BmdFrameRow>,
}

fn read_frame_rows(cursor: &mut Cursor<Box<[u8]>>) -> Vec<BmdFrameRow> {
    let section_header = read_bmd_section_header(cursor);
    let n_rows = section_header.length / 4;
    let mut rows = Vec::with_capacity(n_rows as usize);
    for _ in 0..n_rows {
        rows.push(read_frame_row(cursor));
    }
    rows
}

fn read_frame_row(cursor: &mut Cursor<Box<[u8]>>) -> BmdFrameRow {
    let data = cursor.read_u32::<LittleEndian>().unwrap();
    BmdFrameRow {
        // the 22 lower bits
        offset: data & 0b00111111_11111111_11111111u32,
        // the remaining 10 high-bits
        indent: (data >> 22) & 0b11_11111111u32,
    }
}

pub struct BmdFrameRow {
    // both read as one, first 22 bits are offset; remaining 10 are indent
    offset: u32,
    indent: u32,
}


fn read_pixels(cursor: &mut Cursor<Box<[u8]>>) -> Box<[u8]> {
    let section_header = read_bmd_section_header(cursor);
    let mut buf = vec![0u8; section_header.length as usize].into_boxed_slice();
    cursor.read_exact(buf.as_mut_slice())
}


fn read_frame_infos(cursor: &mut Cursor<Box<[u8]>>) -> Vec<BmdFrameInfo> {
    let section_header = read_bmd_section_header(cursor);
    let n_frames = section_header.length / 6;
    let mut frame_infos = Vec::with_capacity(n_frames as usize);
    for _ in 0..n_frames {
        frame_infos.push(read_frame_info(cursor));
    }
    frame_infos
}

fn read_frame_info(cursor: &mut Cursor<Box<[u8]>>) -> BmdFrameInfo {
    BmdFrameInfo {
        type_: get_bmd_frame_type(cursor.read_u8().unwrap()),
        dx: cursor.read_u8().unwrap(),
        dy: cursor.read_u8().unwrap(),
        width: cursor.read_u8().unwrap(),
        len: cursor.read_u8().unwrap(),
        off: cursor.read_u8().unwrap(),
    }
}

struct BmdFrameInfo {
    type_: BmdFrameType,
    dx: u8,
    dy: u8,
    width: u8,
    len: u8,
    off: u8,
}


fn get_bmd_frame_type(id: u8) -> BmdFrameType {
    match id {
        1 => BmdFrameType::Normal,
        2 => BmdFrameType::Shadow,
        4 => BmdFrameType::Extended,
        _ => panic!("Unknown BMD frame type {}", id)
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum BmdFrameType {
    Normal,
    Shadow,
    Extended,
}


fn read_bmd_section_header(cursor: &mut Cursor<Box<[u8]>>) -> BmdFileSectionHeader {
    BmdFileSectionHeader {
        magic: cursor.read_u32::<LittleEndian>().unwrap(),
        zero0: cursor.read_u32::<LittleEndian>().unwrap(),
        length: cursor.read_u32::<LittleEndian>().unwrap(),
    }
}

pub struct BmdFileSectionHeader {
    pub magic: u32,
    pub zero0: u32,
    pub length: u32,
}


pub struct BmdFileHeader {
    pub magic: u32,
    pub zero0: u32,
    pub zero1: u32,
    pub num_frames: u32,
    pub num_pixels: u32,
    pub num_rows: u32,
    pub unknown0: u32,
    pub unknown1: u32,
    pub zero2: u32,
}

pub fn read_bmd_file_header(cursor: &mut Cursor<Box<[u8]>>) -> BmdFileHeader {
    BmdFileHeader {
        magic: cursor.read_u32::<LittleEndian>().unwrap(),
        zero0: cursor.read_u32::<LittleEndian>().unwrap(),
        zero1: cursor.read_u32::<LittleEndian>().unwrap(),
        num_frames: cursor.read_u32::<LittleEndian>().unwrap(),
        num_pixels: cursor.read_u32::<LittleEndian>().unwrap(),
        num_rows: cursor.read_u32::<LittleEndian>().unwrap(),
        unknown0: cursor.read_u32::<LittleEndian>().unwrap(),
        unknown1: cursor.read_u32::<LittleEndian>().unwrap(),
        zero2: cursor.read_u32::<LittleEndian>().unwrap(),
    }
}