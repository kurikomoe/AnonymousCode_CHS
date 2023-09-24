#![allow(dead_code, unused_imports, unused_variables, unused_mut)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::io::Cursor;
use std::path::PathBuf;
use anyhow::Result;
use binrw::BinRead;
use binrw::io::BufReader;
use clap::Parser;
use data::data::context::Context;
use data::data::{mdf, psb};
use data::data::psb::PsbObject;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    inputs: Vec<PathBuf>,

}


fn main() -> Result<()> {
    let args = Args::parse();
    let key = "5fWhAHt4zVn2X";

    for input in args.inputs {

        let mut ctx = Context {
            key,
            mdf_key: Some(format!("{}{}", key, "motion_info.psb.m")),
            ..Default::default()
        };

        let mut buf = BufReader::new(std::fs::File::open(input)?);

        let mdf = mdf::Mdf::read(&mut buf)?;
        let mut psb = mdf.convert_to_psb(&mut ctx ,true)?;
        let mut br = Cursor::new(&mut psb);
        let _psb = psb::Psb::read(&mut br)?;

        dbg!(_psb);
    }


    Ok(())
}