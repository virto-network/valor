use crate::{
    map,
    structures::{Command, CommandOutput, Module, Request, Response, ResponseError},
};

pub fn handle_command<'a, 's>(
    command: Command<'s>,
    module: &'a Module<'s>,
) -> CommandOutput<'a, 's> {
    match command {
        Command::ExportModule => CommandOutput::ModuleInfo(module),
        Command::MakeCall(request) => {
            CommandOutput::CallResult(make_call(&module, request.path, request))
        }
    }
}

pub fn make_call<'a, 'b>(
    module: &'b Module,
    method_name: &'a str,
    request: Request<'a>,
) -> Result<Response, ResponseError> {
    let method = module
        .methods
        .iter()
        .find(|m| m.name == method_name)
        .ok_or_else(|| ResponseError {
            message: "Method not found".to_string(),
            meta: map! {},
        })?;

    Ok(method.call(request)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structures::{Method, Request, Response};

    #[test]
    fn test_managed_handle_command() {
        let request = Request {
            path: "test",
            meta: Some(map! {}),
            body: Some(b"Hello world!"),
        };

        let method = Method {
            name: "test",
            call: Some(Box::new(|request: &Request| {
                Ok(Response {
                    meta: map! {},
                    body: request.body.unwrap().to_vec(),
                })
            })),
            extensions: map! {},
        };

        let module = Module {
            name: "test",
            methods: vec![method],
            extensions: map! {},
        };

        let command = Command::MakeCall(request.clone());

        let result = handle_command(command, &module);
        match result {
            CommandOutput::CallResult(response) => assert_eq!(request.id, response.id),
            _ => panic!("Unexpected command output"),
        }
    }
}
