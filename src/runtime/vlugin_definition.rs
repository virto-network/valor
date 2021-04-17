use crate::VluginConfig;
use alloc::{borrow::ToOwned, string::String};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The format used to define and configure plugins
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VluginDef {
    /// Name of the plugin
    pub name: String,
    /// Url prefix where the plugin is mounted, defaults to the name
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub prefix: Option<String>,
    /// What kind of plugin
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub r#type: VluginType,
    /// Environment configuration to pass down to the plugin instance
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub config: Option<VluginConfig>, // NOTE this makes the core dependent on serde
}

impl VluginDef {
    pub fn prefix_or_name(&self) -> &str {
        self.prefix
            .as_deref()
            .unwrap_or(&self.name)
            .trim_matches(&['/', ' '][..])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "type", rename_all = "snake_case")
)]
pub enum VluginType {
    /// Plugin that comes with the runtime
    Static,
    /// Natively compiled Rust plugin
    Native {
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        path: Option<String>,
    },
    /// Web script or WASM
    Web { url: String },
}

impl From<&str> for VluginDef {
    fn from(name: &str) -> Self {
        VluginDef {
            name: name.into(),
            prefix: Some("_".to_owned() + name),
            r#type: VluginType::Static,
            config: None,
        }
    }
}

impl From<(&str, &str)> for VluginDef {
    fn from((name, prefix): (&str, &str)) -> Self {
        VluginDef {
            name: name.into(),
            prefix: Some(prefix.into()),
            r#type: VluginType::Static,
            config: None,
        }
    }
}
