#[cfg(no_std)]
pub use ::{
    alloc::{
        boxed::Box, collections::BTreeMap, string::String, string::ToString, sync::Arc, vec::Vec,
    },
    uuid::Uuid,
};

#[cfg(not(no_std))]
pub use ::{
    std::{
        boxed::Box, collections::BTreeMap, fmt::Display, string::String, string::ToString,
        sync::Arc, vec::Vec,
    },
    uuid::Uuid,
};
