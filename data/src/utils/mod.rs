use std::io::Cursor;
use std::path::PathBuf;
use log::debug;
use anyhow::Result;
use binrw::BinReaderExt;
use md5::{Digest, Md5};
use md5::digest::FixedOutput;
use crate::data::psb::{PsbEntry, PsbObject};
use crate::data::psb::PsbObject::*;
use crate::data::resource::{FileEntry, Resource};

pub mod consts;

pub fn collect_files(base_file_path_idx: usize, entry: &PsbEntry, mm: &mut Resource) -> Result<()> {
    let base_file_path = &mm.base_dirs[base_file_path_idx];
    let base_file = base_file_path.file_name().unwrap().to_str().unwrap().to_owned();

    let item = entry.get_entry_by_path("file_info")?;

    let data = item.get_dict()?;

    for (file, value) in data.iter() {
        let value = value.get_list()?;
        let [offset, length] = &value[0..=1] else { panic!("Not enough values") };
        let offset = offset.get_number()?;
        let size = length.get_number()?;

        let name = file.to_string();

        let file_entry = FileEntry {
            base_file: base_file.clone().into(),
            name: name.clone().into(),
            offset,
            size,
        };

        mm.base_file.insert(name.clone(), base_file_path_idx);
        mm.files.insert(name.clone(), file_entry);
    }

    Ok(())
}

pub fn generate_xor_key_from_seed(key: &str, length: usize) -> Result<Vec<u8>> {
    let (cow, encoding_used, had_errors) = encoding_rs::UTF_8.encode(key);
    assert!(!had_errors);

    let mut hasher = Md5::new();
    hasher.update(key);

    let seed: [u32; 4] = Cursor::new(hasher.finalize()).read_le()?;

    let mut rng = rand_mt::Mt19937GenRand32::new_with_key(seed.to_vec());
    let mut keys = vec![0u8; length];
    rng.fill_bytes(&mut keys);

    Ok(keys)
}

pub fn xor_data(data: &mut [u8], keys: &[u8]) {
    let mut idx = 0;
    for d in data.iter_mut() {
        *d = *d ^ keys[idx];
        idx = (idx + 1) % keys.len();
    }
}