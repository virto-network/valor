use crate::deps::{BTreeMap, String, Vec};

use super::Method;

#[cfg(feature = "serialization")]
use ::serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Module {
    pub name: String,
    pub methods: Vec<Method>,
    pub extensions: BTreeMap<String, String>,
}
