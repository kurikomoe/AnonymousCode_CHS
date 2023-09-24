use derivative::Derivative;

#[derive(Clone)]
#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct Context<'a> {
    pub key: &'a str,

    pub mdf_key: Option<String>,

    #[derivative(Default(value = "0x83"))]
    pub mdf_key_length: usize,

    pub is_psb_zlib_fast_compress: Option<bool>,
}
