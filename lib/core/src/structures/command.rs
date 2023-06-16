use super::{Module, Request, Response, ResponseError};

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize, Serializer};

#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub enum Command<'a> {
    ExportModule,
    MakeCall(Request<'a>),
}

#[cfg_attr(feature = "serialization", derive(Deserialize))]
pub enum CommandOutput<'a, 's> {
    ModuleInfo(&'a Module<'s>),
    CallResult(Result<Response, ResponseError>),
}

#[cfg(feature = "serialization")]
impl<'a, 's> Serialize for CommandOutput<'a, 's> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::{Error, SerializeStruct};

        match self {
            CommandOutput::ModuleInfo(module) => {
                let module = module.clone();
                let value = serde_json::to_value(&module).map_err(Error::custom)?;
                let mut struct_serializer = serializer.serialize_struct("CommandOutput", 1)?;
                struct_serializer.serialize_field("ModuleInfo", &value)?;
                struct_serializer.end()
            }
            CommandOutput::CallResult(result) => {
                let value = serde_json::to_value(result).map_err(Error::custom)?;
                let mut struct_serializer = serializer.serialize_struct("CommandOutput", 1)?;
                struct_serializer.serialize_field("CallResult", &value)?;
                struct_serializer.end()
            }
        }
    }
}
