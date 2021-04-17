//! ## Valor
//!
//! A lightweight HTTP plugin system that runs in the server and the browser.
//!
//! - Use `valor_bin` to run your Rust and JS(soon!) plugins in the server.
//! - Use `valor_web` as a script imported from the main document or a worker
//! in your web application to have a local API powered by a service worker.
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

extern crate alloc;
extern crate core;

#[cfg(feature = "proxy")]
mod proxy;
pub mod runtime;
#[cfg(feature = "util")]
mod util;
mod vlugin;

use core::fmt;

pub use async_trait::async_trait;
pub use http_types as http;
#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};
#[cfg(feature = "util")]
pub use util::*;
pub use vlugin::*;

pub type VluginConfig = serde_json::Value;

#[derive(Debug)]
pub enum Error {
    Http(http::Error),
    Runtime(runtime::Error),
    NotSupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Http(err) => write!(f, "{}", err),
            Error::NotSupported => write!(f, "Not supported"),
            Error::Runtime(_) => write!(f, "Runtime error"),
        }
    }
}

impl From<Error> for http::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Http(err) => err,
            _ => http::Error::from_str(http::StatusCode::InternalServerError, ""),
        }
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Self {
        Error::Http(err)
    }
}

impl From<runtime::Error> for Error {
    fn from(err: runtime::Error) -> Self {
        Error::Runtime(err)
    }
}
