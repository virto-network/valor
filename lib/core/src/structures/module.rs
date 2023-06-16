use crate::{
    deps::{BTreeMap, Box, ToString, Vec},
    map,
};

#[cfg(feature = "serialization")]
use ::serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(bound(deserialize = "'de: 'a")))]
pub struct Module<'a> {
    pub name: &'a str,
    pub methods: Vec<Method<'a>>,
    pub extensions: BTreeMap<&'a str, &'a str>,
}
