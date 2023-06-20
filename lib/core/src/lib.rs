#![feature(error_in_core)]
#![feature(trait_alias)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(no_std)]
extern crate alloc;

#[cfg(not(no_std))]
extern crate std;

pub(crate) mod deps;
pub mod structures;
pub mod util;
