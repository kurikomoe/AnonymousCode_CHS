#![allow(non_upper_case_globals)]

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Index;
use std::sync::Arc;
use std::iter::Iterator;

use anyhow::Result;
use binrw::{BinRead, BinReaderExt, BinResult, binrw, BinWrite, BinWriterExt, until_eof};
use byteorder::{ReadBytesExt, WriteBytesExt};
use derivative::Derivative;
use itertools::Itertools;
use nom::AsBytes;
use valued_enums::*;
use num_traits::FromBytes;
use crate::data::read_and_unpack;

pub mod header;
pub use header::PsbHeader;

pub mod entry;
pub use entry::{PsbEntry, PsbObject};

pub mod array;
pub use array::PsbArray;

pub mod names;
pub use names::PsbNames;

pub mod resource;
pub use resource::PsbResources;

pub mod data;


py_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
    PsbEnum(u8):
        None = 0x0
        Null = 0x1
        False = 0x2
        True = 0x3

        //int
        NumberN0 = 0x4
        NumberN1 = 0x5
        NumberN2 = 0x6
        NumberN3 = 0x7
        NumberN4 = 0x8
        NumberN5 = 0x9
        NumberN6 = 0xA
        NumberN7 = 0xB
        NumberN8 = 0xC

        //array N(NUMBER) is count mask
        ArrayN1 = 0xD
        ArrayN2 = 0xE
        ArrayN3 = 0xF
        ArrayN4 = 0x10
        ArrayN5 = 0x11
        ArrayN6 = 0x12
        ArrayN7 = 0x13
        ArrayN8 = 0x14

        //index of key name only used in PSBv1 (according to GMMan's doc)
        KeyNameN1 = 0x11
        KeyNameN2 = 0x12
        KeyNameN3 = 0x13
        KeyNameN4 = 0x14

        //index of strings table
        StringN1 = 0x15
        StringN2 = 0x16
        StringN3 = 0x17
        StringN4 = 0x18

        //resource of thunk
        ResourceN1 = 0x19
        ResourceN2 = 0x1A
        ResourceN3 = 0x1B
        ResourceN4 = 0x1C

        //fpu value
        Float0 = 0x1D
        Float = 0x1E
        Double = 0x1F

        //objects
        List = 0x20     //object list
        Objects = 0x21  //object dictionary

        ExtraChunkN1 = 0x22
        ExtraChunkN2 = 0x23
        ExtraChunkN3 = 0x24
        ExtraChunkN4 = 0x25

        //used by compiler it's fake
        Integer = 0x80
        String = 0x81
        Resource = 0x82
        Decimal = 0x83
        Array = 0x84
        Boolean = 0x85
        BTree = 0x86
}




#[derive(BinRead, Derivative, Clone)]
#[br(little, magic = b"PSB\0")]
#[derivative(Debug)]
pub struct Psb {
    pub header: PsbHeader,

    #[derivative(Debug = "ignore")]
    #[br(seek_before = SeekFrom::Start(header.offset_strings as u64))]
    string_offsets: PsbArray,
    // strings: Vec<String>,

    // FIXME(kuriko): We currently not support old versions.
    #[br(assert(header.version != 1))]
    #[br(seek_before = SeekFrom::Start(header.offset_names as u64))]
    pub names: PsbNames,

    #[br(args(header.offset_chunk_offsets as u64, header.offset_chunk_lengths as u64))]
    pub resources: PsbResources,

    #[br(seek_before = SeekFrom::Start(header.offset_entries as u64))]
    #[br(args { global_names: Arc::new(names.clone()) })]
    pub entries: PsbEntry,
}


#[cfg(test)]
mod test {
    use std::io::BufReader;
    use std::path::PathBuf;

    use dbg_hex::dbg_hex;

    use super::*;

    #[test]
    fn test_psb() -> Result<()> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources");
        d.push("motion_info.psb.m.raw");

        let file = std::fs::File::open(&d).unwrap();
        let mut buf = BufReader::new(file);

        let psb = Psb::read(&mut buf)?;

        dbg_hex!(&psb);


        Ok(())
    }
}