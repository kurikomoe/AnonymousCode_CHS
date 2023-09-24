use anyhow::Result;
use std::io::{Read, Seek};
use binrw::BinRead;
use byteorder::ReadBytesExt;
use num_traits::FromBytes;
use std::convert::TryFrom;
use dbg_hex::dbg_hex;

pub mod context;
pub mod mdf;
pub mod psb;


/// Read a byte as size, then read that many bytes and convert to u32 via little endian.
pub fn read_and_unpack<const N: usize, T, R>(br: &mut R, n: usize) -> Result<T>
where
    R: Read + Seek,
    T: FromBytes<Bytes = [u8; N]>,
{
    let mut buf = vec![0u8; n];
    br.read_exact(&mut buf)?;

    if *buf.last().unwrap() >= 0b100_00000 {  // negative
        buf.resize(N, 0xFF);
    } else {
        buf.resize(N, 0);
    }

    let ret = T::from_le_bytes(buf.as_slice().try_into().unwrap());

    Ok(ret)
}
