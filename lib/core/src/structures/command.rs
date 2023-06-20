use super::{Module, Request, Response, ResponseError};
use crate::deps::Arc;

#[cfg(feature = "serialization")]
use crate::deps::BTreeMap;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serialization", serde(bound(deserialize = "'de: 'a")))]
pub enum Command<'a> {
    ExportModule,
    MakeCall(Request<'a>),
}

pub enum CommandOutput {
    ModuleInfo(Arc<Module>),
    CallResult(Result<Response, ResponseError>),
}

#[cfg(feature = "serialization")]
impl<'s> Serialize for CommandOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::{Error, SerializeStruct};

        let mut struct_serializer = serializer.serialize_struct("CommandOutput", 1)?;
        match self {
            CommandOutput::ModuleInfo(module) => {
                let module = module.clone();

                let value = serde_json::to_value(&*module).map_err(Error::custom)?;
                struct_serializer.serialize_field("ModuleInfo", &value)?;
            }
            CommandOutput::CallResult(result) => {
                let value = serde_json::to_value(result).map_err(Error::custom)?;
                struct_serializer.serialize_field("CallResult", &value)?;
            }
        }

        struct_serializer.end()
    }
}

#[cfg(feature = "serialization")]
impl<'d> Deserialize<'d> for CommandOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        use ::{serde::de::Error, serde_json::Value};

        let map = BTreeMap::<String, Value>::deserialize(deserializer)?;

        if let Some(value) = map.get("ModuleInfo") {
            let module = serde_json::from_value(value.clone())
                .map_err(|_| Error::custom("Error deserializing Module"))?;
            return Ok(CommandOutput::ModuleInfo(Arc::new(module)));
        }

        if let Some(value) = map.get("CallResult") {
            let result: Result<Response, ResponseError> = serde_json::from_value(value.clone())
                .map_err(|_| Error::custom("Error deserializing Result"))?;
            return Ok(CommandOutput::CallResult(result));
        }

        Err(Error::custom("Unexpected variant"))
    }
}
