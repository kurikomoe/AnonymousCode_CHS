use std::collections::HashMap;
use std::io::SeekFrom;
use std::sync::Arc;

use binrw::{BinRead, BinResult};
use derivative::Derivative;

use crate::data::psb::{PsbNames, PsbArray, PsbEntry, entry::PsbEntryBinReadArgs};


#[derive(BinRead, Clone)]
#[derive(Derivative)]
#[derivative(Debug)]
#[br(import(global_names: Arc <PsbNames>))]
pub struct PsbDict {
    #[derivative(Debug = "ignore")]
    pub names: PsbArray,

    #[derivative(Debug = "ignore")]
    offsets: PsbArray,

    #[br(args(& names, & offsets, Arc::clone(& global_names)))]
    #[br(parse_with = PsbDict::parser)]
    pub data: HashMap<String, PsbEntry>,
}

impl PsbDict {
    #[binrw::parser(reader, endian)]
    fn parser(names: &PsbArray, offsets: &PsbArray, global_names: Arc<PsbNames>) -> BinResult<HashMap<String, PsbEntry>> {
        let cur_pos = reader.stream_position()?;

        let args = PsbEntryBinReadArgs::builder()
            .global_names(Arc::clone(&global_names))
            .finalize();

        let mut mm = HashMap::with_capacity(names.len());

        for i in 0..names.len() {
            let name_idx = names[i] as usize;
            assert!(name_idx < global_names.len(), "name index out of range");
            let name = &global_names[name_idx];

            assert!(i < offsets.len(), "offset index out of range");
            let offset = offsets[i] as u64;

            reader.seek(SeekFrom::Start(cur_pos + offset))?;

            let obj = PsbEntry::read_options(reader, endian, args.clone())?;

            mm.insert(name.clone(), obj);
        }

        Ok(mm)
    }
}
