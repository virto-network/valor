use crate::deps::{BTreeMap, String, Uuid};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Request<'a> {
    pub id: Uuid,
    pub path: &'a str,
    pub meta: Option<BTreeMap<String, String>>,
    pub body: Option<&'a [u8]>,
}

impl<'a> Request<'a> {
    pub fn new(
        path: &'a str,
        meta: Option<BTreeMap<String, String>>,
        body: Option<&'a [u8]>,
    ) -> Self {
        Request {
            id: Uuid::new_v4(),
            path,
            meta,
            body,
        }
    }
}
