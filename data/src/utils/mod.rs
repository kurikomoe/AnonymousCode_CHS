use std::io::Cursor;
use std::path::PathBuf;
use log::{debug, error, warn};
use anyhow::{anyhow, Result};
use binrw::BinReaderExt;
use md5::{Digest, Md5};
use md5::digest::FixedOutput;
use regex::Regex;
use crate::data::psb::{PsbEntry, PsbObject};
use crate::data::psb::PsbObject::*;
use crate::data::resource::{FileEntry, FSType, Resource};

pub mod consts;
pub mod file_lists;
use file_lists::*;

pub fn collect_files(base_name: &str, entry: &PsbEntry, mm: &mut Resource, file_list: &mut ListType) -> Result<()> {
    if file_list == &ListType::None { return Ok(()); }

    let suffix = entry
        .get_entry_by_path("expire_suffix_list")
        .and_then(|e| e.get_list())
        .and_then(|e| {
            // FIXME(kuriko): assume that only one suffix exists.
            assert_eq!(e.len(), 1);
            e.get(0).ok_or(anyhow!("suffix not found"))
        })
        .and_then(|e| e.get_string())
        .unwrap_or_else(|e| {
            error!("{:?}", e);
            "".to_owned()
        });

    let item = entry.get_entry_by_path("file_info")?;

    let data = item.get_dict()?;

    for (file, value) in data.iter() {
        // Concat file with extension
        let file = format!("{}{}", file, suffix);
        if let ListType::List(lst) = file_list {
            let idx = lst.iter().position(|e| *e == file);
            if let Some(idx) = idx {
                lst.remove(idx);
            } else {
                warn!("Ignore file: {file}");
                continue;
            }
        }

        let value = value.get_list()?;
        let [offset, length] = &value[0..=1] else { panic!("Not enough values") };
        let offset = offset.get_number()? as u32;
        let size = length.get_number()? as u32;

        let name = format!("{}/{}", base_name, file);

        let file_entry = FileEntry::new(FSType::Embedded, base_name.into(), file, offset, size);

        mm.files.insert(name.clone(), file_entry);
    }

    if let ListType::List(lst) = file_list {
        if !lst.is_empty() {
            error!("Some files are missing:");
            lst.iter().for_each(|e| error!("- {}", e));
        }
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
        *d ^= keys[idx];
        idx = (idx + 1) % keys.len();
    }
}

pub fn get_body_from_info(path: &PathBuf) -> Result<PathBuf> {
    let mut ret = path.to_owned();

    let cur = ret.file_name().unwrap().to_str().unwrap();

    let pat = Regex::new(r"(.+)_info\.psb\.m$").unwrap();

    let name = pat.captures(cur).unwrap().get(1).unwrap().as_str();
    let new_name = format!("{}_body.bin", name) ;

    ret.pop();

    ret.push(new_name);

    Ok(ret)
}
