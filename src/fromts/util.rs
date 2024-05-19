use std::io::{BufRead, Cursor, Read};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use byteorder::{LittleEndian, ReadBytesExt};
use futures_channel::oneshot;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Blob, FileReader};
use web_sys::js_sys::Uint8Array;


pub fn read_normal_string(view: &mut Cursor<Box<[u8]>>) -> String {
    let len = view.read_u32::<LittleEndian>().unwrap();
    read_fixed_string(view, len as usize)
}
pub fn read_short_string(view: &mut Cursor<Box<[u8]>>) -> String {
    let len = view.read_u8().unwrap();
    read_fixed_string(view, len as usize)
}

pub fn read_fixed_string(view: &mut Cursor<Box<[u8]>>, size: usize) -> String {
    // String is fixed length, so we have to add the NULL termination manually
    let mut buffer = vec![0u8; size];
    view.read_exact(buffer.as_mut_slice()).unwrap();

    String::from_utf8(buffer).unwrap()
}

pub fn read_zero_terminated_string(view: &mut Cursor<Box<[u8]>>) -> String {
    let mut buf: Vec<u8> = Vec::new();
    view.read_until(0x0, &mut buf).unwrap();
    if buf.is_empty() || *buf.last().unwrap() != 0u8 {
        panic!("No zero terminated string read for cif!");
    }
    buf.remove(buf.len() - 1);

    String::from_utf8(buf).unwrap().to_owned()
}

pub fn read_bytes<const N: usize>(cursor: &mut Cursor<Box<[u8]>>, size: usize) -> std::io::Result<[u8; N]> {
    let mut buffer = [0u8; N];
    cursor.read_exact(&mut buffer)?;
    Ok(buffer)
}

pub async fn read_file(blob: Blob) -> Uint8Array {
    let file_reader = FileReader::new().unwrap();

    file_reader.read_as_array_buffer(&blob).unwrap();

    let (tx, rx) = oneshot::channel::<Result<Uint8Array, ()>>();
    let fr_c = file_reader.clone();

    let tx = Arc::new(Mutex::new(Some(tx)));

    let onloadend_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
        let array = Uint8Array::new(&fr_c.result().unwrap());

        if let Some(tx) = tx.lock().unwrap().take() { // Take to ensure it's used only once
            tx.send(Ok(array)).unwrap();
        } else {
            panic!("onloadend is called twice!");
        }
    }) as Box<dyn Fn(web_sys::ProgressEvent)>);

    let error_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
        !panic!("Error reading data from blob")
    }) as Box<dyn Fn(web_sys::ProgressEvent)>);

    file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
    file_reader.set_onerror(Some(error_cb.as_ref().unchecked_ref()));
    file_reader.set_onabort(Some(error_cb.as_ref().unchecked_ref()));
    file_reader.read_as_array_buffer(&blob).expect("blob not readable");
    onloadend_cb.forget();
    error_cb.forget();
    rx.await.unwrap().unwrap()
}

pub fn is_eof<A>(cursor: &Cursor<Vec<A>>) -> bool {
    cursor.position() as usize == cursor.get_ref().len()
}

/*
pub fn is_eof<A, B>(cursor: &Cursor<A>) -> bool
    where A: Deref, A::Target: AsRef<[B]> {
    cursor.position() as usize == cursor.get_ref().len()
}
 */
