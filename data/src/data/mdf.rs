use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use anyhow::Result;
use binrw::{BinRead, BinReaderExt, binrw, BinWrite, BinWriterExt, helpers::until_eof};
use byteorder::{ReadBytesExt, WriteBytesExt};
use dbg_hex::dbg_hex;
use encoding_rs::*;
use flate2::read::DeflateDecoder;
use md5::{Digest, Md5};
use md5::digest::FixedOutput;
use nom::AsBytes;

use crate::data::context::Context;
use crate::utils;

#[binrw]
#[br(little)]
pub struct Mdf {
    #[br(assert(magic == b"mdf\0".as_bytes()))]
    magic: [u8; 4],
    size: u32,
    #[br(parse_with = until_eof)]
    raw_data: Vec<u8>,
}

impl Mdf {
    fn decrypt_data(&self, ctx: &mut Context, keep_header: bool) -> Result<Vec<u8>> {
        assert!(ctx.mdf_key.is_some());

        let mut br = Cursor::new(&self.raw_data);

        // FIXME(kuriko): use actual size
        let mut buf = Vec::with_capacity(self.raw_data.len());
        let mut bw = Cursor::new(&mut buf);

        if keep_header {
            bw.write_le(&self.magic)?;
            bw.write_le(&self.size)?;
        }

        let mut keys = utils::generate_xor_key_from_seed(ctx.mdf_key.as_ref().unwrap(), ctx.mdf_key_length)?;

        let mut idx = 0;
        while let Ok(data) = br.read_u8() {
            bw.write_u8(data ^ keys[idx % ctx.mdf_key_length])?;
            idx += 1;
        }

        Ok(buf)
    }

    pub fn convert_to_psb(&self, ctx: &mut Context, keep_header: bool) -> Result<Vec<u8>> {
        let mut data = self.decrypt_data(ctx, keep_header)?;
        let mut br = Cursor::new(&mut data);

        br.seek(SeekFrom::Start(4))?;
        let size = br.read_le::<u32>()? as usize;

        br.seek(SeekFrom::Current(1))?;
        ctx.is_psb_zlib_fast_compress = Some(br.read_u8()? == 0x9c);

        br.seek(SeekFrom::Start(0))?;
        let data = self.decompress_zlib(ctx, &mut br, size)?;

        Ok(data)
    }

    fn decompress_zlib<T: Read + Seek>(&self, ctx: &mut Context, br: &mut T, size: usize) -> Result<Vec<u8>> {
        br.seek(SeekFrom::Start(10))?;

        let mut buf = {
            let mut decoder = DeflateDecoder::new(br);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf)?;
            buf
        };

        Ok(buf)
    }
}


#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use binrw::io::BufReader;

    use super::*;

    #[test]
    fn parse_mdf_test() -> Result<()> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources");
        d.push("motion_info.psb.m");

        let file = std::fs::File::open(&d).unwrap();
        let mut buf = BufReader::new(file);

        let mdf = Mdf::read(&mut buf)?;

        let mut ctx = Context {
            key: "5fWhAHt4zVn2X",
            mdf_key: Some("5fWhAHt4zVn2Xmotion_info.psb.m".to_owned()),
            ..Default::default()
        };

        let psb = mdf.convert_to_psb(&mut ctx, true)?;

        d.pop();
        d.push("motion_info.psb.m.raw");
        let file = std::fs::File::create(&d)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(&psb)?;

        Ok(())
    }
}