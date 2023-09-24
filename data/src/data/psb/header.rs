use binrw::BinRead;
use derivative::Derivative;


#[derive(BinRead, Clone)]
#[derive(Derivative)]
#[derivative(Debug)]
#[br(little)]
pub struct PsbHeader {
    pub version: u16,
    #[br(assert(header_encrypt == 0))]
    pub header_encrypt: u16,
    #[derivative(Debug = "ignore")]
    pub header_length: u32,
    #[derivative(Debug = "ignore")]
    pub offset_names: u32,

    #[derivative(Debug = "ignore")]
    pub offset_strings: u32,
    #[derivative(Debug = "ignore")]
    pub offset_strings_data: u32,

    #[derivative(Debug = "ignore")]
    pub offset_chunk_offsets: u32,
    #[derivative(Debug = "ignore")]
    pub offset_chunk_lengths: u32,
    #[derivative(Debug = "ignore")]
    pub offset_chunk_data: u32,

    #[derivative(Debug = "ignore")]
    pub offset_entries: u32,

    #[br(if (version > 2))]
    pub checksum: Option<u32>,

    #[br(if (version > 3))]
    pub extra_data: Option<PsbHeaderExtraData>,
}

#[derive(BinRead, Clone)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct PsbHeaderExtraData {
    pub offset_extra_chunk_offsets: u32,
    pub offset_extra_chunk_lengths: u32,
    pub offset_extra_chunk_data: u32,
}
