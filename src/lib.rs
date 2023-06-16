#![feature(error_in_core)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use valor_core::{map, structures};
pub use valor_proc::{extensions, method, module};

#[cfg(target_arch = "wasm32")]
pub use valor_core::interop;
