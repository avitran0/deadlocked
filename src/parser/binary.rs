use std::io::Read;
use bytemuck::Pod;

pub fn read<T: Pod>(reader: &mut impl Read) -> T {
    let mut buffer = vec![0u8; std::mem::size_of::<T>()];
    reader.read_exact(&mut buffer).unwrap();
    *bytemuck::from_bytes(&buffer)
}

pub fn read_string(reader: &mut impl Read) -> String {
    let mut buffer = Vec::with_capacity(8);
    let mut byte = [0u8; 1];

    loop {
        reader.read_exact(&mut byte).unwrap();
        if byte[0] == 0 {
            break;
        }
        buffer.push(byte[0]);
    }
    String::from_utf8(buffer).unwrap()
}

pub fn read_bytes(reader: &mut impl Read, count: usize) -> Vec<u8> {
    let mut buf = vec![0u8; count];
    reader.read_exact(&mut buf).unwrap();
    buf
}
