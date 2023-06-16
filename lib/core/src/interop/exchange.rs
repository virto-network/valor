use super::{
    managed,
    serialization::{deserialize, serialize},
};
use crate::{
    deps::{Arc, String},
    structures::{Command, Module},
};

fn output_message(input: String) -> (*const u8, usize) {
    let bytes: &[u8] = &input.as_bytes();
    let bytes_len = bytes.len();

    (bytes.as_ptr(), bytes_len)
}

pub fn export_module<'a>(module: Arc<Module>) -> (*const u8, usize) {
    let serialized = serialize(&*module).expect("Could not serialize module");
    let output = output_message(serialized);

    let (ptr, len) = output;
    unsafe {
        let slice = std::slice::from_raw_parts(ptr, len);
        dbg!(&slice);
    }

    output
}

pub fn make_call<'a>(
    module: Arc<Module>,
    method_name: &'a str,
    request_input: &'a str,
) -> (*const u8, usize) {
    let request = deserialize(request_input).expect("Could not deserialize the request");

    let output = match managed::make_call(module, method_name, request) {
        Ok(response) => serialize(&response).expect("Could not serialize response"),
        Err(error) => serialize(&error).expect("Could not serialize error"),
    };

    output_message(output)
}

pub fn handle_command<'a>(module: Arc<Module>) {
    use std::io::Read;

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    let command: Command = deserialize(&input).unwrap();

    let output = managed::handle_command(command, module);

    let serde_error = serde_json::json!({
        "error": "Could not serialize"
    });

    println!("{}", &serialize(&output).unwrap_or(serde_error.to_string()));
}
