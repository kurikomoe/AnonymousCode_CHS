#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_size_of_val)]

use std::env::{temp_dir, VarError};
use std::ffi::c_char;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::prelude::MetadataExt;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Result};
use binrw::BinRead;
use binrw::io::BufReader;
use hex_literal::hex;
use md5::{Digest, Md5};
use md5::digest::FixedOutput;
use nom::HexDisplay;
use once_cell::sync::OnceCell;
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use tempfile::TempDir;
use windows_sys::Win32::Foundation::{FALSE, GetLastError, SetLastError};
use windows_sys::Win32::Storage::FileSystem::{FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_SYSTEM, FILE_ATTRIBUTE_TEMPORARY, SetFileAttributesA};


use crate::data::resource::{FileEntry, FSType, Resource};
use crate::utils::{consts, generate_xor_key_from_seed, get_entry_key, xor_data};
use crate::utils::consts::*;

pub mod data;
pub mod utils;


#[cxx::bridge(namespace = "kdata")]
pub mod ffi {
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
        pub uid: u32,
        pub offset: u64,
        pub size: u64,
    }

    extern "Rust" {
        fn load_resource_dat() -> RetCode;
        fn say_hello() -> RetCode;

        pub fn release_resource() -> Result<()>;
        pub fn get_mapping_info(file: &str) -> Result<MappingInfo>;
        pub fn get_mapping_info_by_idx(idx: i64) -> Result<MappingInfo>;
        pub fn get_resource_dat_file() -> String;
        pub fn decrypt_buffer(buf: &mut [u8], info: &MappingInfo) -> Result<()>;
        pub fn get_unpack_dir() -> String;
        pub fn locate_movie(filename: String) -> Result<String>;

        pub fn is_debug_mode() -> bool;
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

use ffi::RetCode;
use ffi::MappingInfo;

static RESOURCE: OnceCell<Resource> = OnceCell::new();
static mut UNPACK_DIR: OnceCell<TempDir> = OnceCell::new();

/// Load resource dat from current folder.
pub fn load_resource_dat() -> RetCode {
    unsafe {
        let Ok(_) = UNPACK_DIR.set({
            let tmp = PathBuf::from("windata/.ac_movie_sc");
            if tmp.exists() {
                ffi::debug("Remove old .ac_movie_ac folder");
                std::fs::remove_dir_all(tmp).ok();
            }

            let Ok(tmp) = tempfile::Builder::new()
                .prefix(".ac_movie_sc")
                .suffix("")
                .rand_bytes(0)
                .tempdir_in("windata")
                else { return RetCode::CreateTempDirFailed; };
            ffi::debug(&format!("Tmp: {:?}", tmp.path()));
            // wait a little bit to make sure folder created.
            let cur = std::env::current_dir().unwrap();
            SetLastError(0);
            for i in 0..10 {
                let rel = tmp.path().relative_to(&cur).unwrap();
                ffi::debug(&format!("set file attribution: {:?}", rel));
                if FALSE == SetFileAttributesA(
                    rel.to_string().as_ptr(),
                    FILE_ATTRIBUTE_HIDDEN|FILE_ATTRIBUTE_SYSTEM
                ) {
                    ffi::debug(&format!("set file attribution failed: {:?}, err code: {}, retrying", tmp.path(), GetLastError()));
                    sleep(Duration::from_millis(200));
                } else {
                    break;
                };
            }
            tmp
        }) else { return RetCode::GlobalInitUnpackDirFailed; };
    }

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

pub fn is_debug_mode() -> bool {
    std::env::var("KDEBUG").is_ok()
}

pub fn release_resource() -> Result<()> {
    unsafe {
        // Clean up temporary dir.
        drop(UNPACK_DIR.take());
    }
    Ok(())
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
        uid: v.uid,
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
        uid: v.uid,
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
     unsafe { UNPACK_DIR.get().unwrap().path().to_str().unwrap().to_string() }
}

pub fn decrypt_buffer(buf: &mut [u8], info: &MappingInfo) -> Result<()> {
    let key = &RESOURCE.get().unwrap().key;
    let keys = generate_xor_key_from_seed(&get_entry_key(key, info.uid), 114514)
        .expect("Cannot generate key");

    xor_data(buf, &keys);
    Ok(())
}

pub fn locate_movie(filename: String) -> Result<String> {
    // if PathBuf::from(format!("movies/{}", &filename)).exists() {
    //     Ok(format!("../movies/{}", &filename))
    // } else if PathBuf::from(&filename).exists() {
    //     Ok(format!("../{}", &filename))
    // } else {
    //     Err(anyhow!("Movie file Not Found"))
    // }

    let mut tmp = unsafe { UNPACK_DIR.get().unwrap().path().to_path_buf() };
    let mut file = tmp.to_path_buf();

    let res_dat = get_resource_dat_file();

    let res = RESOURCE.get().unwrap();

    let key = &res.key;
    let entry = res.files.get(&filename).unwrap();

    let keys = generate_xor_key_from_seed(&get_entry_key(key, entry.uid), 114514).expect("Cannot generate key");

    let mut input = std::fs::File::open(res_dat)?;
    let mut br = BufReader::new(&mut input);
    br.seek(SeekFrom::Start(res.end_of_header + entry.real_offset as u64))?;

    let mut hasher = Md5::new();
    hasher.update(&filename);
    let result = format!("{:x}", hasher.finalize());

    file.push(&result);
    file.set_extension("mzv");

    if !file.exists()
        || file.metadata().map(|m| m.file_size()).unwrap_or(0) != entry.size as u64 {
        let mut buf = vec![0u8; entry.size as usize];
        br.read_exact(&mut buf)?;
        xor_data(&mut buf, &keys);
        buf[0..4].copy_from_slice(b"MZV\0");

        let out = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .attributes(FILE_ATTRIBUTE_HIDDEN)
            .open(&file)?;
        // let out = std::fs::File::create(&file)?;

        let mut bw = BufWriter::new(out);
        bw.write_all(&buf)?;
    } else {
        ffi::debug(&format!("Using cached file: {:?}", &file));
    }

    // tmp.push("windata");
    let mut base = tmp.parent().unwrap().to_path_buf();
    // base.push("windata");

    let rel = file.relative_to(base)?.to_string();
    ffi::debug(&format!("Locate movie: {:?} -> {:?} rel: {:?}", &filename, &file, &rel));
    Ok(rel)
}

pub fn say_hello() -> RetCode {
    ffi::error("Test: Say Hello");
    ffi::warn("Test: Say Hello");
    ffi::info("Test: Say Hello");
    ffi::debug("Test: Say Hello");
    ffi::trace("Test: Say Hello");
    RetCode::Ok
}

