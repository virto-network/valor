use crate::{
    deps::Arc,
    map,
    structures::{Command, CommandOutput, Module, Request, Response, ResponseError},
};

pub fn handle_command<'a>(command: Command<'a>, module: Arc<Module>) -> CommandOutput {
    match command {
        Command::ExportModule => CommandOutput::ModuleInfo(module),
        Command::MakeCall(request) => {
            CommandOutput::CallResult(make_call(module.clone(), request.path, request))
        }
    }
}

pub fn make_call<'a>(
    module: Arc<Module>,
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
        let request = Request::new("test", Some(map! {}), Some(b"Hello world!"));

        let method = Method {
            name: "test".to_owned(),
            call: Some(Box::new(|request: &Request| {
                Ok(Response {
                    id: request.id,
                    meta: map! {},
                    body: request.body.unwrap().to_vec(),
                })
            })),
            extensions: map! {},
        };

        let module = Arc::new(Module {
            name: "test".to_owned(),
            methods: vec![method],
            extensions: map! {},
        });

        let command = Command::MakeCall(request.clone());

        let result = handle_command(command, module.clone());
        match result {
            CommandOutput::CallResult(response) => match response {
                Ok(res) => assert_eq!(request.id, res.id),
                Err(err) => panic!("error processing call: {}", err),
            },
            _ => panic!("Unexpected command output"),
        }
    }
}
