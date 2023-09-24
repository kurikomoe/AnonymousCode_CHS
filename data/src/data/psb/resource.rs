use std::io::SeekFrom;

use binrw::BinRead;
use derivative::Derivative;

use super::array::PsbArray;


#[derive(BinRead, Clone)]
#[derive(Derivative)]
#[derivative(Debug)]
#[br(import(offset_chunk_offsets: u64, offset_chunk_lengths: u64))]
pub struct PsbResources {
    #[br(seek_before = SeekFrom::Start(offset_chunk_offsets))]
    chunk_offsets: PsbArray,
    #[br(seek_before = SeekFrom::Start(offset_chunk_lengths))]
    chunk_lengths: PsbArray,

    // #[br(count = length)]
    // data: Vec<Vec<u8>>,
}
