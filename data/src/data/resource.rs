#![allow(clippy::ptr_arg)]

use crate::utils::{self, consts, get_body_from_info};
use crate::utils::{generate_xor_key_from_seed, xor_data};
use std::io::{Read, Seek};
use std::io::SeekFrom;
use anyhow::Result;
use binrw::{binrw, BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt};
use bytes::Bytes;
use derivative::Derivative;
use secrecy::Secret;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor};
use std::path::PathBuf;
use indexmap::IndexMap;

use super::helper::{KBuf, KString};


static mut FILE_ENTRY_COUNTER: u32 = 0;

#[binrw]
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub uid: u32,

    pub base: KString,

    pub name: KString,

    pub offset: u32,
    pub size: u32,

    pub real_offset: u32,
}

impl FileEntry {
    pub fn new(base: String, name: String, offset: u32, size: u32) -> Self {
        let idx = unsafe { FILE_ENTRY_COUNTER += 1; FILE_ENTRY_COUNTER};

        Self {
            uid: idx,
            name: name.into(),
            base: base.into(),
            offset,
            size,
            real_offset: 0,
        }

    }
}


/*
    ref: motion/ac_logo.psb.m

    base_name: motion
    filename: ac_logo.psb.m

    offset: 0xaaaa
    size:   0xbbbb
    -> uniq_id: 0xccc

    file -> uniq_id
 */

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct Resource {
    #[bw(assert(is_finished == b"DAT1"))]
    pub is_finished: [u8; 4],

    // "motion" => name idx
    #[brw(ignore)]
    pub base_files: IndexMap<String, PathBuf>,

    #[br(parse_with = Resource::read_key)]
    #[bw(write_with = Resource::write_key)]
    pub key: String,

    #[bw(calc = files.len() as u32)]
    pub file_cnt: u32,

    /// motion/ac_logo.psb.m => FileEntry
    #[bw(args(&key))]
    #[bw(write_with = Resource::write_files)]
    #[br(args(&key, file_cnt as usize))]
    #[br(parse_with = Resource::read_files)]
    pub files: IndexMap<String, FileEntry>,

    #[bw(args(&key, &base_files, &files))]
    #[bw(write_with = Resource::write_data)]
    pub raw_data: (),
}

impl Resource {
    pub fn add_base(&mut self, base_file_name: String, base_file_path: PathBuf) {
        self.base_files.insert(base_file_name, base_file_path);
    }

    pub fn calc_offsets(&mut self) -> u32 {
        self.is_finished.copy_from_slice(b"DAT1");

        let mut offset = 0;
        for v in self.files.values_mut() {
            v.real_offset = offset;
            offset += v.size;
        }

        offset
    }
}

impl Resource {
    #[binrw::writer(writer, endian)]
    fn write_key(key: &String) -> BinResult<()> {
        let mut buf = key.as_bytes().to_vec();
        let keys = generate_xor_key_from_seed(consts::LOGO, 233).expect("get xor key failed");
        xor_data(&mut buf, &keys);
        (buf.len() as u32).write_le(writer)?;
        buf.write_le(writer)?;
        Ok(())
    }

    #[binrw::parser(reader, endian)]
    fn read_key() -> BinResult<String> {
        let sz = <u32>::read_options(reader, endian, ())? as usize;
        let mut buf = vec![0u8; sz];
        reader.read_exact(&mut buf)?;

        let keys = generate_xor_key_from_seed(consts::LOGO, 233).expect("get xor key failed");
        xor_data(&mut buf, &keys);

        let ret = String::from_utf8(buf).expect("invalid data");

        Ok(ret)
    }

    #[binrw::writer(writer, endian)]
    fn write_files(files: &IndexMap<String, FileEntry>, key: &str) -> BinResult<()> {
        let keys = generate_xor_key_from_seed(key, 114514).expect("Cannot generate key");

        let mut offset = 0;
        for (key, value) in files.iter() {
            let key = KString::from(key.clone());
            let mut buf = Vec::new();
            let mut bw = Cursor::new(&mut buf);

            key.write_le(&mut bw)?;
            value.write_le(&mut bw)?;

            xor_data(&mut buf, &keys);

            writer.write_le(&KBuf::from(buf))?;
        }

        Ok(())
    }

    #[binrw::parser(reader, endian)]
    fn read_files(key: &String, cnt: usize) -> BinResult<IndexMap<String, FileEntry>> {
        let keys = generate_xor_key_from_seed(key, 114514).expect("Cannot generate key");

        let mut ret = IndexMap::new();

        for _ in 0..cnt {
            let mut buf = KBuf::read_le(reader)?;
            xor_data(&mut buf.data, &keys);

            let mut br = Cursor::new(&mut buf.data);

            let key = <KString>::read_le(&mut br)?;
            let value = <FileEntry>::read_le(&mut br)?;
            ret.insert(key.data, value);
        }

        Ok(ret)
    }

    #[binrw::writer(writer, endian)]
    fn write_data(
        _: &(),
        key: &String,
        base_files: &IndexMap<String, PathBuf>,
        files: &IndexMap<String, FileEntry>
    ) -> BinResult<()> {
        let keys = generate_xor_key_from_seed(key, 114514).expect("Cannot generate key");

        for (file, entry) in files.iter() {
            let FileEntry { uid, base, name, offset, size, mut real_offset } = &entry;

            let path = base_files.get(&base.data).expect("base file not found");
            let path = get_body_from_info(path).unwrap();

            let mut file = File::open(&path)?;
            file.seek(SeekFrom::Start(*offset as u64))?;
            let mut buf = vec![0u8; *size as usize];
            file.read_exact(&mut buf)?;
            xor_data(&mut buf, &keys);
            writer.write_le(&buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::context::Context;
    use crate::data::mdf::Mdf;
    use crate::data::psb::Psb;
    use binrw::io::BufReader;
    use std::io::{BufWriter, Cursor};
    use dbg_hex::dbg_hex;

    #[test]
    fn test_rw() -> Result<()> {
        let key = "「How's it going to end?」";

        let mut mdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        mdf_path.push("resources");
        mdf_path.push("motion_info.psb.m");

        let file = std::fs::File::open(&mdf_path).unwrap();
        let mut buf = BufReader::new(file);

        let mdf = Mdf::read(&mut buf)?;

        let mut ctx = Context {
            key: "5fWhAHt4zVn2X",
            mdf_key: Some("5fWhAHt4zVn2Xmotion_info.psb.m".to_owned()),
            ..Default::default()
        };

        let psb = mdf.convert_to_psb(&mut ctx, true)?;
        let mut cursor = Cursor::new(psb);
        let psb = Psb::read(&mut cursor)?;

        let mut res_path = PathBuf::from("resources");
        res_path.push("resource.bin");

        let mut resource = Resource {
            key: key.to_string(),
            ..Default::default()
        };

        {
            resource.add_base("motion".to_string(), mdf_path);
            utils::collect_files("motion", &psb.entries, &mut resource)?;

            let file = std::fs::File::create(&res_path)?;
            let mut writer = BufWriter::new(file);

            resource.calc_offsets();
            resource.write(&mut writer)?;
        }

        let res2 = {
            let file = std::fs::File::open(&res_path)?;
            let mut reader = BufReader::new(file);
            Resource::read(&mut reader)?
        };

        Ok(())
    }
}
