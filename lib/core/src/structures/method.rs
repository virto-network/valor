use crate::{
    deps::{BTreeMap, Box, String, ToString},
    map,
};

use super::{Request, Response, ResponseError};

#[cfg(feature = "serialization")]
use ::serde::{Deserialize, Serialize};

pub trait Call = Fn(&Request) -> Result<Response, ResponseError>;

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Method {
    pub name: String,
    #[cfg_attr(feature = "serialization", serde(skip))]
    pub call: Option<Box<dyn Call + Send + Sync>>,
    pub extensions: BTreeMap<String, String>,
}

impl Method {
    pub fn call<'a>(&self, request: Request<'a>) -> Result<Response, ResponseError> {
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
