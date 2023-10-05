#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::env::temp_dir;
use std::ffi::c_char;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use binrw::BinRead;
use binrw::io::BufReader;
use once_cell::sync::OnceCell;
use relative_path::{RelativePath, RelativePathBuf};
use tempfile::TempDir;

use ffi::RetCode;

use crate::data::resource::{FileEntry, FSType, Resource};
use crate::ffi::MappingInfo;
use crate::utils::{consts, generate_xor_key_from_seed, xor_data};
use crate::utils::consts::*;

pub mod data;
pub mod utils;


#[cxx::bridge(namespace = "kdata")]
mod ffi {
    pub enum RetCode {
        Ok = 0,

        ResourceFileNotFound,
        ParseResourceFailed,
        GlobalInitFailed,
        GlobalInitUnpackDirFailed,
        CreateTempDirFailed,

        Unknown = 255,
    }

    #[derive(Debug, Clone)]
    pub struct MappingInfo {
        pub idx: u64,
        pub offset: u64,
        pub size: u64,
    }

    extern "Rust" {
        fn load_resource_dat() -> RetCode;
        fn say_hello() -> RetCode;

        pub fn get_mapping_info(file: &str) -> Result<MappingInfo>;
        pub fn get_mapping_info_by_idx(idx: i64) -> Result<MappingInfo>;
        pub fn get_resource_dat_file() -> String;
        pub fn decrypt_buffer(buf: &mut [u8]) -> Result<()>;
        pub fn get_unpack_dir() -> String;
        pub fn locate_movie(filename: String) -> Result<String>;
    }

    unsafe extern "C++" {
        include!("utils/log.h");
        fn warn(msg: &str);
        fn info(msg: &str);
        fn debug(msg: &str);
        fn trace(msg: &str);

        fn error(msg: &str);
    }
}


static RESOURCE: OnceCell<Resource> = OnceCell::new();
static UNPACK_DIR: OnceCell<TempDir> = OnceCell::new();

/// Load resource dat from current folder.
pub fn load_resource_dat() -> RetCode {
    // let Ok(_) = UNPACK_DIR.set({
    //     let mut cur = RelativePath::new("");
    //     let Ok(tmp) = tempfile::Builder::new()
    //         .prefix(".anonymouscode_chs")
    //         .suffix("")
    //         .rand_bytes(0)
    //         .tempdir_in(cur.to_path("windata"))
    //         else { return RetCode::CreateTempDirFailed; };
    //     ffi::debug(&format!("Tmp: {:?}", tmp.path()));
    //     tmp
    // }) else { return RetCode::GlobalInitUnpackDirFailed; };

    let Ok(file) = std::fs::File::open(RES_PATH)
        else {
            ffi::error(&format!("Failed to open resource file. current dir: {:?}", std::env::current_dir()));
            return RetCode::ResourceFileNotFound;
        };

    let mut reader = BufReader::new(file);
    match Resource::read(&mut reader) {
        Ok(v) => match RESOURCE.set(v) {
            Ok(_) => RetCode::Ok,
            Err(_) => RetCode::GlobalInitFailed
        }
        ,
        Err(_) => {
            ffi::error("Failed to parse resource file.");
            RetCode::ParseResourceFailed
        }
    }
}

pub fn get_mapping_info(file: &str) -> Result<MappingInfo> {
    // ffi::debug(&format!("file: {:?}", file));
    let res = RESOURCE.get().unwrap();

    let v = match res.files.get(file) {
        Some(v) => v,
        None => {
            return Err(anyhow!("Req file not found: {file}"));
        }
    };

    let ret = MappingInfo {
        idx: v.uid as u64,
        offset: res.end_of_header + v.real_offset as u64,
        size: v.size as u64,
    };

    Ok(ret)
}

pub fn get_mapping_info_by_idx(idx: i64) -> Result<MappingInfo> {
    let res = RESOURCE.get().unwrap();
    let (name, v) = match res.files.get_index((idx - 1) as usize) {
        None => return Err(anyhow!("idx {idx} out of range")),
        Some(v) => v,
    };

    let ret = MappingInfo {
        idx: v.uid as u64,
        offset: res.end_of_header + v.real_offset as u64,
        size: v.size as u64,
    };

    ffi::debug(&format!("From idx {idx} get file: {}, {:?}", v.name.data, ret));

    Ok(ret)
}

pub fn get_resource_dat_file() -> String {
    consts::RES_PATH.to_string()
}

pub fn get_unpack_dir() -> String {
    UNPACK_DIR.get().unwrap().path().to_str().unwrap().to_string()
}

pub fn decrypt_buffer(buf: &mut [u8]) -> Result<()> {
    let key = &RESOURCE.get().unwrap().key;
    let keys = generate_xor_key_from_seed(key, 114514).expect("Cannot generate key");
    xor_data(buf, &keys);
    Ok(())
}

pub fn locate_movie(filename: String) -> Result<String> {
    // let res = RESOURCE.get().unwrap();
    // let key = &res.key;
    // let keys = generate_xor_key_from_seed(key, 114514).expect("Cannot generate key");
    // let mut tmp = UNPACK_DIR.get().unwrap().path().to_path_buf();
    // let res_dat = get_resource_dat_file();
    // let (file, entry) = res.files.get_index(idx - 1).unwrap();


    if PathBuf::from(format!("movies/{}", &filename)).exists() {
        Ok(format!("../movies/{}", &filename))
    } else if PathBuf::from(&filename).exists() {
        Ok(format!("../{}", &filename))
    } else {
        Err(anyhow!("Movie file Not Found"))
    }

    // let mut input = std::fs::File::open(res_dat)?;
    // let mut br = BufReader::new(&mut input);
    // br.seek(SeekFrom::Start(res.end_of_header + entry.real_offset as u64))?;
    //
    // let mut buf = vec![0u8; entry.size as usize];
    // br.read_exact(&mut buf)?;
    //
    // xor_data(&mut buf, &keys);
    // buf[0..4].fill(0);
    //
    // tmp.push(file);
    // if let Ok(v) = std::fs::File::create(&tmp) {
    //     let mut bw = BufWriter::new(v);
    //     bw.write_all(&buf)?;
    // };
    //
    // // tmp.pop();
    // let tmp = RelativePath::from_path(&tmp)?;
    // let mut tmp = tmp.relative("windata");
    // tmp.push(file);

}

pub fn say_hello() -> RetCode {
    ffi::error("Test: Say Hello");
    ffi::warn("Test: Say Hello");
    ffi::info("Test: Say Hello");
    ffi::debug("Test: Say Hello");
    ffi::trace("Test: Say Hello");
    RetCode::Ok
}

