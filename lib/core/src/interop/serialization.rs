use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string, Result};

pub fn deserialize<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T> {
    from_str(s)
}

pub fn serialize(s: &impl Serialize) -> Result<String> {
    to_string(s)
}
