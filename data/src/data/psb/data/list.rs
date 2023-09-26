use std::io::SeekFrom;
use std::sync::Arc;

use binrw::{BinRead, BinResult};
use derivative::Derivative;

use crate::data::psb::{PsbArray, PsbHeader, SharedData};
use crate::data::psb::{PsbEntry, entry::PsbEntryBinReadArgs};
use crate::data::psb::PsbNames;


#[derive(BinRead, Clone)]
#[derive(Derivative)]
#[derivative(Debug)]
#[br(import{
    shared: Arc<SharedData>,
})]
pub struct PsbList {
    #[derivative(Debug = "ignore")]
    offsets: PsbArray,

    #[br(args(&offsets, shared))]
    #[br(parse_with = PsbList::parse_array)]
    pub array: Vec<PsbEntry>,
}

impl PsbList {
    #[binrw::parser(reader, endian)]
    fn parse_array(offsets: &PsbArray, shared: Arc<SharedData>) -> BinResult<Vec<PsbEntry>> {
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
}
