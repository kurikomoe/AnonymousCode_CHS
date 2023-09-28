use std::collections::HashMap;
use std::ffi::CString;
use std::io::SeekFrom;
use std::sync::Arc;

use binrw::{BinRead, BinResult};
use derivative::Derivative;
use valued_enums::ValuedEnum;
use crate::data::psb::{PsbHeader, SharedData};
use crate::data::psb::PsbArray;
use crate::data::read_and_unpack;
use dbg_hex::dbg_hex;
use num_traits::FromBytes;
use anyhow::{anyhow, Result};
use cxx::T;

use super::PsbEnum;
use super::names::PsbNames;
use super::data::PsbNumber;
use super::data::PsbList;
use super::data::PsbDict;


#[derive(BinRead, Clone, Debug)]
#[br(import {
    shared: Arc<SharedData>,
})]
pub struct PsbEntry {
    #[br(parse_with = PsbEntry::parse_type)]
    pub ty: PsbEnum,

    #[br(args {
        ty,
        shared: shared.clone(),
    })]
    pub obj: PsbObject,
}

impl PsbEntry {
    #[binrw::parser(reader, endian)]
    fn parse_type() -> BinResult<PsbEnum> {
        let ty = <u8>::read_options(reader, endian, ())?;
        let ty = PsbEnum::from_value(&ty).unwrap();
        Ok(ty)
    }
}


#[derive(BinRead, Clone, Debug)]
#[br(import {
    ty: PsbEnum,
    shared: Arc<SharedData>,
})]
pub enum PsbObject {
    #[br(pre_assert(ty == PsbEnum::None))]
    None,

    #[br(pre_assert(ty == PsbEnum::Null))]
    Null,

    #[br(pre_assert(ty == PsbEnum::False || ty == PsbEnum::True))]
    Bool(
        #[br(calc(ty == PsbEnum::True))]
        bool
    ),

    #[br(pre_assert(ty == PsbEnum::NumberN0))]
    Zero,  // Avoid compare between 0 and others.

    // Numbers
    #[br(pre_assert(PsbEnum::NumberN0 < ty && ty <= PsbEnum::NumberN4))]
    Int32(
        #[br(args((ty.value() - PsbEnum::NumberN0.value()) as usize))]
        #[br(parse_with = PsbObject::parser)]
        i32
    ),

    #[cfg(target_pointer_width = "64")]
    #[br(pre_assert(PsbEnum::NumberN4 <= ty && ty <= PsbEnum::NumberN8))]
    Int64(
        #[br(args((ty.value() - PsbEnum::NumberN0.value()) as usize))]
        #[br(parse_with = PsbObject::parser)]
        i64
    ),

    // Floats
    #[br(pre_assert(ty == PsbEnum::Float0 || ty == PsbEnum::Float))]
    Float(
        #[br(args(4))]
        #[br(parse_with = PsbObject::parser)]
        f32
    ),

    #[cfg(target_pointer_width = "64")]
    #[br(pre_assert(ty == PsbEnum::Double))]
    Double(
        #[br(args(8))]
        #[br(parse_with = PsbObject::parser)]
        f64
    ),

    #[br(pre_assert(PsbEnum::StringN1 <= ty && ty <= PsbEnum::StringN4))]
    String(
        #[br(args(ty, shared.clone()))]
        #[br(parse_with = PsbObject::parse_string)]
        String
    ),

    // Datastructures
    #[br(pre_assert(ty == PsbEnum::List))]
    List(
        #[br(args(shared.clone()))]
        #[br(parse_with = PsbObject::parse_array)]
        Vec<PsbEntry>
    ),

    #[br(pre_assert(ty == PsbEnum::Objects))]
    Dict(
        // #[br(args{
        //     shared: shared.clone()
        // })]
        // PsbDict

        #[br(args(shared.clone()))]
        #[br(parse_with = PsbObject::parse_dict)]
        HashMap<String, PsbEntry>,
    ),

    Unknown,
}

impl PsbEntry {
    pub fn get_entry_by_path(&self, path: &str) -> Result<&PsbEntry> {
        let path = path.split('.');
        let mut entry = self;

        for key in path {
            let tmp = entry.get_dict()?;
            entry = tmp.get(key).unwrap();
        }

        Ok(entry)
    }
}

impl PsbEntry {
    pub fn get_dict(&self) -> Result<&HashMap<String, PsbEntry>> {
        match &self.obj {
            PsbObject::Dict(v) => Ok(v),
            _ => Err(anyhow!("Not a dict: {self:?}")),
        }
    }

    pub fn get_list(&self) -> Result<&Vec<PsbEntry>> {
        match &self.obj {
            PsbObject::List(v) => Ok(v),
            _ => Err(anyhow!("Not a List: {self:?}")),
        }
    }

    #[cfg(not(target_pointer_width = "64"))]
    pub fn get_number(&self) -> Result<i32> {
        match &self.obj {
            PsbObject::Zero => Ok(0),
            PsbObject::Int32(v) => Ok(*v),
            _ => Err(anyhow!("Not a i32: {self:?}"))
        }
    }

    #[cfg(not(target_pointer_width = "64"))]
    pub fn get_float(&self) -> Result<f32> {
        match &self.obj {
            PsbObject::Float(v) => Ok(*v),
            _ => Err(anyhow!("Not a f32: {self:?}"))
        }
    }

    #[cfg(target_pointer_width = "64")]
    pub fn get_number(&self) -> Result<i64> {
        match &self.obj {
            PsbObject::Int32(v) => Ok(*v as i64),
            PsbObject::Int64(v) => Ok(*v),
            _ => Err(anyhow!("Not a i64: {self:?}"))
        }
    }

    #[cfg(target_pointer_width = "64")]
    pub fn get_float(&self) -> Result<f64> {
        match &self.obj {
            PsbObject::Float(v) => Ok(*v as f64),
            PsbObject::Double(v) => Ok(*v),
            _ => Err(anyhow!("Not a f64: {self:?}"))
        }
    }
}



impl PsbObject {
    #[binrw::parser(reader, endian)]
    fn parse_string(ty: PsbEnum, shared: Arc<SharedData>) -> BinResult<String> {
        let SharedData {
            header,
            string_offsets,
            names,
        } = &*shared;

        let sz = ty.value() - PsbEnum::StringN1.value() + 1;
        let idx: usize = read_and_unpack(reader, sz as usize).unwrap();
        let offset = (header.offset_strings_data + string_offsets.data[idx]) as usize;

        reader.seek(SeekFrom::Start(offset as u64))?;

        let mut ss = Vec::new();
        while let Ok(value) = <u8>::read_options(reader, endian, ()) {
            if value == 0 {
                break;
            } else {
                ss.push(value);
            }
        }
        let ss = String::from_utf8(ss).unwrap();

        Ok(ss)
    }


    #[binrw::parser(reader, endian)]
    fn parser<T>(size: usize) -> BinResult<T>
    where
        T: FromBytes<Bytes=[u8; std::mem::size_of::<T>()]>
    {
        let ret = read_and_unpack(reader, size).unwrap();
        Ok(ret)
    }


    #[binrw::parser(reader, endian)]
    fn parse_array(shared: Arc<SharedData>) -> BinResult<Vec<PsbEntry>> {
        let offsets = <PsbArray>::read_options(reader, endian, ())?;

        let cur_pos = reader.stream_position()?;

        let mut arr = Vec::with_capacity(offsets.len());

        let args = PsbEntryBinReadArgs::builder()
            .shared(shared)
            .finalize();

        for i in 0..offsets.len() {
            let offset = offsets[i] as u64;
            reader.seek(SeekFrom::Start(cur_pos + offset))?;

            let obj = PsbEntry::read_options(reader, endian, args.clone())?;

            arr.push(obj);
        }

        Ok(arr)
    }


    #[binrw::parser(reader, endian)]
    fn parse_dict(shared: Arc<SharedData>) -> BinResult<HashMap<String, PsbEntry>> {
        let names = <PsbArray>::read_options(reader, endian, ())?;
        let offsets = <PsbArray>::read_options(reader, endian, ())?;

        let cur_pos = reader.stream_position()?;

        let args = PsbEntryBinReadArgs::builder()
            .shared(shared.clone())
            .finalize();

        let mut mm = HashMap::with_capacity(names.len());

        for i in 0..names.len() {
            let name_idx = names[i] as usize;
            assert!(name_idx < shared.names.len(), "name index out of range");
            let name = &shared.names[name_idx];

            assert!(i < offsets.len(), "offset index out of range");
            let offset = offsets[i] as u64;

            reader.seek(SeekFrom::Start(cur_pos + offset))?;

            let obj = PsbEntry::read_options(reader, endian, args.clone())?;

            mm.insert(name.clone(), obj);
        }

        Ok(mm)
    }
}
