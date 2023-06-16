use crate::{
    deps::{BTreeMap, Box, ToString},
    map,
};

use super::{Request, Response, ResponseError};

#[cfg(feature = "serialization")]
use ::serde::{Deserialize, Serialize};

pub trait Call = Fn(&Request) -> Result<Response, ResponseError>;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(bound(deserialize = "'de: 'a")))]
pub struct Method<'a> {
    pub name: &'a str,
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub call: Option<Box<dyn Call + Send + Sync>>,
    pub extensions: BTreeMap<&'a str, &'a str>,
}

impl<'a> Method<'a> {
    pub fn call(&self, request: Request<'a>) -> Result<Response, ResponseError> {
        if let Some(call) = &self.call {
            call(&request)
        } else {
            Err(ResponseError {
                meta: map! {},
                message: "Method not implemented".to_string(),
            })
        }
    }
}
