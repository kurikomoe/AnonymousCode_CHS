use binrw::{binrw, BinResult, BinWriterExt};


#[binrw]
#[derive(Clone, Debug)]
pub struct KBuf {
    #[bw(calc = data.len() as u32)]
    pub size: u32,

    #[br(args(size))]
    #[br(parse_with = KBuf::parser)]
    #[bw(write_with = KBuf::writer)]
    pub data: Vec<u8>,
}

impl From<Vec<u8>> for KBuf {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value,
        }
    }
}

impl KBuf {
    #[binrw::parser(reader, endian)]
    fn parser(sz: u32) -> BinResult<Vec<u8>> {
        let mut buf = vec![0u8; sz as usize];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[binrw::writer(writer, endian)]
    fn writer(buf: &Vec<u8>) -> BinResult<()> {
        writer.write_le(&buf)?;
        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone)]
pub struct KString {
    #[bw(calc = data.len() as u32)]
    pub size: u32,

    #[br(args(size))]
    #[br(parse_with = KString::parse_string)]
    #[bw(write_with = KString::write_string)]
    pub data: String,
}

impl From<String> for KString {
    fn from(value: String) -> Self {
        Self {
            data: value,
        }
    }
}

impl KString {
    #[binrw::parser(reader, endian)]
    fn parse_string(sz: u32) -> BinResult<String> {
        let mut buf = vec![0u8; sz as usize];
        reader.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).expect("Invalid String"))
    }

    #[binrw::writer(writer, endian)]
    fn write_string(s: &String) -> BinResult<()> {
        writer.write_le(&s.as_bytes())?;
        Ok(())
    }
}