use kv_log_macro::debug;
use libloading::{Library, Symbol};
use valor::{HandlerResponse, Loader, Plugin, Request, RequestHandler};

pub(crate) struct DynLoader;

impl Loader for DynLoader {
    fn load(&self, plugin: &Plugin) -> Result<Box<dyn RequestHandler>, ()> {
        match plugin {
            Plugin::Native { name, path } => {
                let path = path.as_ref().unwrap_or(name);
                let lib = Library::new(path).map_err(|_| ())?;

                debug!("loading native plugin {}", path);
                let get_request_handler: Symbol<'_, fn() -> _> =
                    unsafe { lib.get(b"get_request_handler") }.map_err(|_| ())?;
                debug!("symbol {:?}", plugin);

                let handler = get_request_handler();

                Ok(Box::new(PluginContainer { handler, _lib: lib }))
            }
            _ => Err(()),
        }
    }
}

struct PluginContainer {
    handler: Box<dyn RequestHandler>,
    _lib: Library,
}

impl RequestHandler for PluginContainer {
    fn handle_request(&self, request: Request) -> HandlerResponse {
        self.handler.handle_request(request)
    }
}
