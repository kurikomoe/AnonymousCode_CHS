#![allow(clippy::ptr_arg)]

use std::fs::File;
use std::io::{Read, Seek};
use std::io::{BufWriter, Cursor};
use std::io::SeekFrom;
use std::mem::size_of_val;
use std::path::PathBuf;

use anyhow::Result;
use binrw::{BinRead, BinReaderExt, BinResult, binrw, BinWrite, BinWriterExt};
use indexmap::IndexMap;

use crate::utils::{self, consts, get_body_from_info, get_entry_key};
use crate::utils::{generate_xor_key_from_seed, xor_data};

use super::helper::{KBuf, KString};

static mut FILE_ENTRY_COUNTER: u32 = 0;

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[brw(repr = u8)]
pub enum FSType {
    Embedded = 0,
    Unpack,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub uid: u32,

    pub ty: FSType,

    pub base: KString,

    pub name: KString,

    pub offset: u32,
    pub size: u32,

    pub real_offset: u32,
}

impl FileEntry {
    pub fn new(ty: FSType, base: String, name: String, offset: u32, size: u32) -> Self {
        let idx = unsafe {
            FILE_ENTRY_COUNTER += 1;
            FILE_ENTRY_COUNTER
        };

        Self {
            uid: idx,
            ty,
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
#[derive(Debug, Clone)]
pub struct Resource {
    #[bw(assert(is_finished.starts_with(consts::RESOURCE_DAT_MAGIC)))]
    pub is_finished: [u8; consts::RESOURCE_DAT_MAGIC.len()],

    // "motion" => name idx
    #[brw(ignore)]
    pub base_files: IndexMap<String, PathBuf>,

    #[br(parse_with = Resource::read_key)]
    #[bw(write_with = Resource::write_key)]
    pub key: String,

    #[bw(calc = files.len() as u32)]
    pub file_cnt: u32,

    /// motion/ac_logo.psb.m => FileEntry
    #[bw(args(& key))]
    #[bw(write_with = Resource::write_files)]
    #[br(args(& key, file_cnt as usize))]
    #[br(parse_with = Resource::read_files)]
    pub files: IndexMap<String, FileEntry>,

    #[bw(write_with = Resource::write_current_position)]
    pub end_of_header: u64,

    #[bw(write_with = Resource::write_data)]
    #[bw(args(& key, & base_files, & files))]
    pub raw_data: (),
}

impl Default for Resource {
    fn default() -> Self {
        Self {
            is_finished: [0u8; consts::RESOURCE_DAT_MAGIC.len()],
            base_files: IndexMap::new(),
            key: String::new(),
            files: IndexMap::new(),
            end_of_header: 0,
            raw_data: (),
        }
    }
}

impl Resource {
    pub fn add_base(&mut self, base_file_name: String, base_file_path: PathBuf) {
        self.base_files.insert(base_file_name, base_file_path);
    }

    pub fn calc_offsets(&mut self) -> u32 {
        self.is_finished.copy_from_slice(consts::RESOURCE_DAT_MAGIC);

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
        let keys = generate_xor_key_from_seed(key, 114514)
            .expect("Cannot generate key");

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
        files: &IndexMap<String, FileEntry>,
    ) -> BinResult<()> {
        for (file, entry) in files.iter() {
            let keys = generate_xor_key_from_seed(&get_entry_key(key, entry.uid), 114514).expect("Cannot generate key");

            let FileEntry { uid, ty, base, name, offset, size, mut real_offset } = &entry;

            let mut file = if let Some(v) = base_files.get(&base.data) {
                let path = get_body_from_info(v).unwrap();
                File::open(&path)?
            } else {
                // Base file not record, thus the raw binary file
                File::open(&base.data)?
            };

            file.seek(SeekFrom::Start(*offset as u64))?;

            let mut buf = vec![0u8; *size as usize];
            file.read_exact(&mut buf)?;

            xor_data(&mut buf, &keys);
            writer.write_le(&buf)?;

        }

        Ok(())
    }

    #[binrw::writer(writer, endian)]
    fn write_current_position(_: &u64) -> BinResult<()> {
        let ret = writer.stream_position()? + std::mem::size_of::<u64>() as u64;
        writer.write_le(&ret)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufWriter, Cursor};

    use binrw::io::BufReader;

    use crate::data::context::Context;
    use crate::data::mdf::Mdf;
    use crate::data::psb::Psb;
    use crate::utils::file_lists::ListType;

    use super::*;

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
            utils::collect_files("motion", &psb.entries, &mut resource, &mut ListType::All)?;

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
