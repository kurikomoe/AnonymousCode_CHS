use std::sync::Arc;

use binrw::{BinRead, BinResult};
use derivative::Derivative;
use valued_enums::ValuedEnum;

use super::PsbEnum;
use super::names::PsbNames;
use super::data::PsbNumber;
use super::data::PsbList;
use super::data::PsbDict;


#[derive(BinRead, Clone, Debug)]
#[br(import {
    global_names: Arc<PsbNames>
})]
pub struct PsbEntry {
    #[br(parse_with = PsbEntry::parse_type)]
    pub ty: PsbEnum,

    #[br(args {
    ty,
    global_names: Arc::clone(& global_names)
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
    global_names: Arc <PsbNames>
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
        #[br(args{ size: (ty.value() - PsbEnum::NumberN0.value()) as usize})]
        PsbNumber<i32>
    ),

    #[cfg(target_pointer_width = "64")]
    #[br(pre_assert(PsbEnum::NumberN4 <= ty && ty <= PsbEnum::NumberN8))]
    Int64(
        #[br(args{ size: (ty.value() - PsbEnum::NumberN0.value()) as usize})]
        PsbNumber<i64>
    ),

    // Floats
    #[br(pre_assert(ty == PsbEnum::Float0 || ty == PsbEnum::Float))]
    Float(
        #[br(args{ size: 4})]
        PsbNumber<f32>
    ),

    #[cfg(target_pointer_width = "64")]
    #[br(pre_assert(ty == PsbEnum::Double))]
    Double(
        #[br(args{ size: 8})]
        PsbNumber<f64>
    ),

    // Datastructures
    #[br(pre_assert(ty == PsbEnum::List))]
    List(
        #[br(args(Arc::clone(& global_names)))]
        PsbList
    ),

    #[br(pre_assert(ty == PsbEnum::Objects))]
    Dict(
        #[br(args(Arc::clone(& global_names)))]
        PsbDict
    ),

    Unknown,
}
