use nom::IResult;
use nom::error::{ParseError, ErrorKind, make_error};
use nom::{do_parse, named, take, tag, many1};
use nom::number::complete::{le_u8, le_u16, le_u32};
use nom::Err;

pub struct PcxHeader<'a> {
  pub magic: u8,
  pub version: u8,
  pub encoding_method: u8,
  pub bits_per_pixel: u8,
  pub x0: usize,
  pub y0: usize,
  pub x1: usize,
  pub y1: usize,
  pub h_dpi: u16,
  pub v_dpi: u16,
  pub palette: &'a[u8],
  pub reserved: u8,
  pub color_planes: u8,
  pub bytes_per_color_plane: u16,
  pub palette_type: u16,
  pub h_res: u16,
  pub v_res: u16,
  // reserved_block: [u8; 54],
}

// pub fn run_length<'a, E: ParseError<&'a [u8]>>(buf: &'a[u8]) -> IResult<&'a[u8], &'a[u8], E> {
//   if buf.len() < 1 {
//     Err(Err::Error(make_error(buf, ErrorKind::Eof)))
//   } else if buf[0] > 192 {
//     Ok((&buf[1..], &vec![buf[1]; buf[0] as usize][..]))
//   } else {
//     Ok((&buf[1..], &[buf[0]]))
//   }
// }

// fn read_pixels<'a, E: ParseError<&'a [u8]>>(buf: &[u8], width: usize, height: usize) -> IResult<&'a[u8], &'a[u8], E> {
//   fold_many1!(call!(run_length), &[], |mut acc: &[u8], v| {
//     acc += v;
//     acc
//   })
// }

fn read_pixels<'a, 'b>(buf: &'a [u8], pixels: &'b mut Vec<u8>) -> IResult<&'a[u8], &'b[u8], ()> {
  let mut i = 0;
  let mut pos = 0;

  while i < pixels.len() {
    let mut val = buf[pos]; pos += 1;
    let mut len = 1;

    if val > 192 {
      len = val - 192;
      val = buf[pos]; pos += 1;
    }

    while len > 0 {
      pixels[i] = val;
      i += 1;
      len -= 1;
    }
  }

  Ok((&buf[pos..], &pixels[..]))
}

pub struct RGBAColor {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
  pub alpha: u8,
}

named!(pcx_palette_rgba<RGBAColor>, do_parse!(
  red: le_u8 >>
  green: le_u8 >>
  blue: le_u8 >>
  (RGBAColor {
    red,
    green,
    blue,
    alpha: 0xFF
  })
));

named!(pcx_palette<Vec<RGBAColor>>, do_parse!(
  tag!([0x0C]) >>
  colors: many1!(pcx_palette_rgba) >>
  (colors)
));

named!(pcx_header<PcxHeader>, do_parse!(
  magic: le_u8 >>
  version: le_u8 >>
  encoding_method: le_u8 >>
  bits_per_pixel: le_u8 >>
  x0: le_u16 >>
  y0: le_u16 >>
  x1: le_u16 >>
  y1: le_u16 >>
  h_dpi: le_u16 >>
  v_dpi: le_u16 >>
  palette: take!(48) >>
  reserved: le_u8 >>
  color_planes: le_u8 >>
  bytes_per_color_plane: le_u16 >>
  palette_type: le_u16 >>
  h_res: le_u16 >>
  v_res: le_u16 >>
  take!(54) >>
  (PcxHeader {
    magic,
    version,
    encoding_method,
    bits_per_pixel,
    x0: x0 as usize,
    y0: y0 as usize,
    x1: x1 as usize,
    y1: y1 as usize,
    h_dpi,
    v_dpi,
    palette,
    reserved,
    color_planes,
    bytes_per_color_plane,
    palette_type,
    h_res,
    v_res,
  })
));

pub fn pcx_read(buf: &[u8], out: &mut [u8]) {
  if let Ok((rest, header)) = pcx_header(buf) {
    let mut pixels = vec![0; (header.x1 - header.x0 + 1) * (header.y1 - header.y0 + 1)];
    let (rest, _) = read_pixels(&rest, &mut pixels).expect("read_pixels failed.");
    if let Ok((rest, palette)) = pcx_palette(rest) {
      for i in 0..pixels.len() {
        out[4 * i + 0] = palette[pixels[i] as usize].red;
        out[4 * i + 1] = palette[pixels[i] as usize].green;
        out[4 * i + 2] = palette[pixels[i] as usize].blue;
        out[4 * i + 3] = palette[pixels[i] as usize].alpha;
      }
    }
  } else {
    panic!("Oh!");
    // Err(Err::Error(make_error(buf, ErrorKind::Eof)))
  }
}

pub fn pcx_texture_array(buf: &[u8], out: &mut [u8]) {

}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::BufReader;
  use std::io::Read;
  
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_read_pcx_header() {
    match File::open("/mnt/c/Users/Abbas/Projects/Personal/cultures2-wasm/tests/tran_desertbrown.pcx") {
      Ok(file) => {
        let mut buf_reader = BufReader::new(file);
        let mut buffer = Vec::new();

        buf_reader.read_to_end(&mut buffer);
        let res = pcx_header(&buffer[..]);
        if let Ok((_i, header)) = res {
            assert_eq!(header.magic, 0x0A);
            assert_eq!(header.version, 0x05);
            assert_eq!(header.encoding_method, 0x01);
            assert_eq!(header.bits_per_pixel, 0x08);
            assert_eq!(header.x0, 0);
            assert_eq!(header.y0, 0);
            assert_eq!(header.x1, 255);
            assert_eq!(header.y1, 255);
        } else {
            panic!("Hey!");
        }
      },
      Err(e) => {
        panic!("File not found!");
      }
    }
  }

  #[test]
  fn test_pcx_read() {
    match File::open("/mnt/c/Users/Abbas/Projects/Personal/cultures2-wasm/tests/tran_desertbrown.pcx") {
      Ok(file) => {
        let mut buf_reader = BufReader::new(file);
        let mut buffer = Vec::new();

        buf_reader.read_to_end(&mut buffer);
        let mut out = [0u8; 256 * 256 * 4];
        pcx_read(&buffer, &mut out);
      },
      Err(e) => {
        panic!("File not found!");
      }
    }
  }
}
