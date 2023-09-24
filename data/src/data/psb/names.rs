use std::ops::Index;

use binrw::{BinRead, BinResult};
use derivative::Derivative;

use super::array::PsbArray;

#[derive(BinRead, Clone)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct PsbNames {
    #[derivative(Debug = "ignore")]
    charset: PsbArray,
    #[derivative(Debug = "ignore")]
    names_data: PsbArray,
    #[derivative(Debug = "ignore")]
    name_indexes: PsbArray,

    #[br(args(& charset, & names_data, & name_indexes))]
    #[br(parse_with = PsbNames::load_names)]
    pub names: Vec<String>,
}

impl Index<usize> for PsbNames {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.names[index]
    }
}

impl PsbNames {
    pub(crate) fn len(&self) -> usize { self.names.len() }
    #[binrw::parser(reader, endian)]
    fn load_names(charset: &PsbArray, names_data: &PsbArray, name_indexes: &PsbArray) -> BinResult<Vec<String>> {
        let mut names = Vec::with_capacity(name_indexes.len());

        // WTF? How can these people reverse this algorithm out?
        // Let's think about how to build this in the forward direction.
        for index in name_indexes.data.iter() {
            let mut buf = Vec::new();
            let mut chr = names_data[*index as usize];
            while chr != 0 {
                // print!("{}->", chr);
                let code = names_data.data[chr as usize];
                let d = charset[code as usize];
                let real_chr = chr - d;
                chr = code;
                buf.push(real_chr as u8);
            }
            // println!("->0");
            buf.reverse();
            let ss = String::from_utf8(buf).unwrap();
            names.push(ss);
        }

        Ok(names)
    }
}
