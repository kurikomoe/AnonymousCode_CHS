#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Result;
use binrw::{BinRead, BinWrite};
use binrw::io::BufReader;
use bytes::Bytes;
use clap::Parser;
use dbg_hex::dbg_hex;
use derivative::Derivative;
use log::{debug, info};
use regex::Regex;

use data::{mdf, psb};
use data::context::Context;
use data::psb::PsbObject;
use data::resource::{FileEntry, Resource};
use utils::{consts, file_lists::*};
use crate::data::helper::KString;
use crate::data::resource::FSType;

pub mod data;
pub mod utils;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Key for psb files
    #[arg(short, long)]
    key: String,

    /// Key for output file
    #[arg(short, long)]
    encrypt_key: Option<String>,

    /// *_config.psb.m to be packed together
    inputs: Vec<PathBuf>,

    /// Output file
    #[arg(short, long, default_value_os_t = PathBuf::from(consts::RES_PATH))]
    out: PathBuf,

    /// Only pack files in this list
    #[arg(short, long)]
    file_lists: Option<PathBuf>,
}

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let key = args.key;

    // Parse the file lists
    let mut file_lists: Option<FileLists> = args.file_lists.map(|file_lists| {
        let mut file = std::fs::File::open(file_lists).unwrap();
        serde_json::from_reader(file).unwrap()
    });

    let encrypt_key = args
        .encrypt_key.unwrap_or_else(|| "[HIDDEN]".to_string());

    let mut resource = Resource {
        key: encrypt_key,
        ..Default::default()
    };

    let pat = Regex::new(r"(.+)_info\.psb\.m$")?;

    for input in args.inputs {
        // motion.psb.m
        let file = input.file_name().unwrap().to_str().unwrap();

        if let Some(caps) = pat.captures(file) {
            let base_name = caps.get(1).unwrap().as_str();
            debug!("Processing {file}, base: {base_name}");

            // Check psb
            let mut ctx = Context {
                key: &key,
                mdf_key: Some(format!("{}{}", key, file)),
                ..Default::default()
            };

            let mut buf = BufReader::new(std::fs::File::open(&input)?);

            let mdf = mdf::Mdf::read(&mut buf)?;
            let mut psb = mdf.convert_to_psb(&mut ctx, true)?;
            let mut br = Cursor::new(&mut psb);
            let psb = psb::Psb::read(&mut br)?;

            // let base = input.file_name().unwrap().to_str().unwrap();
            let mut just_none = ListType::None;
            let file_list = file_lists
                .as_mut()
                .map(|mut file_lists| file_lists.get_mut(base_name).unwrap())
                .unwrap_or(&mut just_none);

            resource.add_base(base_name.to_string(), input.clone());
            utils::collect_files(
                base_name,
                &psb.entries,
                &mut resource,
                file_list,
            )?;
        } else {
            info!("binary file, just add it: {:?}", &input);
            let bin = std::fs::File::open(&input)?;
            let size = bin.metadata()?.len();
            let entry = FileEntry::new(
                FSType::Unpack,
                input.to_str().unwrap().to_string(),
                file.to_string(),
                0,
                size as u32,
            );
            resource.files.insert(file.to_string(), entry);

        }
    }

    resource.calc_offsets();
    // dbg!(&resource);

    let mut out = std::fs::File::create(&args.out)?;
    let mut writer = std::io::BufWriter::new(&mut out);

    dbg!(&resource);

    resource.write(&mut writer)?;

    Ok(())
}