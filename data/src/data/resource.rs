#![allow(clippy::ptr_arg)]

use crate::utils::{self, consts};
use crate::utils::{generate_xor_key_from_seed, xor_data};
use anyhow::Result;
use binrw::{binrw, BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt};
use bytes::Bytes;
use derivative::Derivative;
use secrecy::Secret;
use std::collections::HashMap;
use std::io::{BufWriter, Cursor};
use std::path::PathBuf;

use super::helper::{KBuf, KString};

#[binrw]
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub base_file: KString,
    pub name: KString,
    pub offset: i32,
    pub size: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default)]
pub struct Resource {
    #[brw(ignore)]
    pub base_dirs: Vec<PathBuf>,

    #[brw(ignore)]
    pub base_file: HashMap<String, usize>,

    #[br(parse_with = Resource::read_key)]
    #[bw(write_with = Resource::write_key)]
    pub key: String,

    #[bw(calc = files.len() as u32)]
    pub file_cnt: u32,

    #[bw(args(&key))]
    #[bw(write_with = Resource::write_files)]
    #[br(args(&key, file_cnt as usize))]
    #[br(parse_with = Resource::read_files)]
    pub files: HashMap<String, FileEntry>,

    #[brw(ignore)]
    pub files_in_resource: HashMap<String, FileEntry>,
}

impl Resource {
    pub fn add_new_path(&mut self, path: PathBuf) -> usize {
        self.base_dirs.push(path);
        self.base_dirs.len() - 1
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
    fn write_files(files: &HashMap<String, FileEntry>, key: &str) -> BinResult<()> {
        let keys = utils::generate_xor_key_from_seed(key, 114514).expect("Cannot generate key");

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
    fn read_files(key: &String, cnt: usize) -> BinResult<HashMap<String, FileEntry>> {
        let keys = utils::generate_xor_key_from_seed(key, 114514).expect("Cannot generate key");

        let mut ret = HashMap::new();

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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::context::Context;
    use crate::data::mdf::Mdf;
    use crate::data::psb::Psb;
    use binrw::io::BufReader;
    use std::io::{BufWriter, Cursor};

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
            utils::collect_files(resource.add_new_path(mdf_path), &psb.entries, &mut resource)?;

            let file = std::fs::File::create(&res_path)?;
            let mut writer = BufWriter::new(file);
            resource.write(&mut writer)?;
        }

        let res2 = {
            let file = std::fs::File::open(&res_path)?;
            let mut reader = BufReader::new(file);
            Resource::read(&mut reader)?
        };

        dbg!(&res2);

        Ok(())
    }
}
