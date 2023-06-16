use crate::deps::{BTreeMap, Display, String, Vec};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Response {
    pub meta: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub struct ResponseError {
    pub meta: BTreeMap<String, String>,
    pub message: String,
}

impl core::error::Error for ResponseError {}

impl Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}
Meta = {:?}",
            self.message, self.meta
        )
    }
}
