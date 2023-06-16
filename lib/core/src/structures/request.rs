use crate::deps::{BTreeMap, String};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Request<'a> {
    pub path: &'a str,
    pub meta: Option<BTreeMap<String, String>>,
    pub body: Option<&'a [u8]>,
}
