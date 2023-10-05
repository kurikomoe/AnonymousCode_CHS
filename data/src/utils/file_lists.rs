use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use serde::de::Error;
use serde_repr::Deserialize_repr;

#[derive(PartialEq, Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListType {
    None,

    All,

    #[serde(untagged)]
    List(Vec<String>),
}

fn mapping_to_all<'de, D>(de: D) -> Result<ListType, D::Error>
where
    D: Deserializer<'de>
{
    let buf: String = Deserialize::deserialize(de)?;
    if buf == "*" {
        Ok(ListType::All)
    } else {
        Err(Error::custom("Invalid string"))
    }
}

pub type FileLists = HashMap<String, ListType>;