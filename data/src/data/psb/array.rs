use std::ops::Index;
use itertools::Itertools;

use binrw::{BinRead, BinResult};
use derivative::Derivative;
use valued_enums::ValuedEnum;

use super::PsbEnum;

#[derive(BinRead, Clone)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct PsbArray {
    #[br(parse_with = PsbArray::get_array_length)]
    length: usize,

    #[br(parse_with = PsbArray::get_entry_length)]
    entry_length: usize,

    #[br(args(length, entry_length))]
    #[br(parse_with = PsbArray::build_array)]
    #[derivative(Debug = "ignore")]
    pub(crate) data: Vec<u32>,
}

impl Index<usize> for PsbArray {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len(), "PsbArray: index out of bounds");
        &self.data[index]
    }
}

impl PsbArray {
    pub fn len(&self) -> usize { self.length }
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    #[binrw::parser(reader, endian)]
    pub fn get_array_length() -> BinResult<usize> {
        let n = <u8>::read_options(reader, endian, ())?;
        let n = n - PsbEnum::ArrayN1.value() + 1;
        assert!(n <= 4, "unsupported large array length {} > (4)u32", n);

        let mut buf = [0u8; 4];
        (0..n as usize).for_each(|i| buf[i] = <u8>::read_le(reader).unwrap());

        let length = u32::from_le_bytes(buf);
        Ok(length as usize)
    }

    #[binrw::parser(reader, endian)]
    pub fn get_entry_length() -> BinResult<usize> {
        let n = <u8>::read_options(reader, endian, ())?;
        let n = n - PsbEnum::NumberN8.value();
        Ok(n as usize)
    }

    #[binrw::parser(reader, endian)]
    pub fn build_array(length: usize, entry_length: usize) -> BinResult<Vec<u32>> {
        assert!(entry_length <= 4, "unsupported large entry length {}", entry_length);

        if length * entry_length == 0 {
            return Ok(Vec::new());
        }

        let mut buf = vec![0u8; length * entry_length];
        reader.read_exact(&mut buf)?;

        let ret = buf.into_iter().chunks(entry_length).into_iter()
            .map(|e| {
                let mut buf = e.into_iter().collect::<Vec<u8>>();
                buf.resize(4, 0); // extend to length of 4
                u32::from_le_bytes(buf.try_into().unwrap())
            })
            .collect();

        Ok(ret)
    }
}
