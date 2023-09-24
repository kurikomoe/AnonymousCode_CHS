use binrw::{BinRead, BinResult};
use derivative::Derivative;
use num_traits::FromBytes;

use crate::data::read_and_unpack;


#[derive(BinRead, Clone, Debug)]
#[br(import { size: usize})]
pub struct PsbNumber<T> where
    T: FromBytes<Bytes=[u8; std::mem::size_of::<T>()]>
{
    #[br(args(size))]
    #[br(parse_with = PsbNumber::parser)]
    pub data: T,
}

impl<T: FromBytes<Bytes=[u8; std::mem::size_of::<T>()]>> PsbNumber<T> {
    #[binrw::parser(reader, endian)]
    fn parser(size: usize) -> BinResult<T> {
        let ret = read_and_unpack(reader, size).unwrap();
        Ok(ret)
    }
}
