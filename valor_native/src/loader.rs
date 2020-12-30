use async_trait::async_trait;
use kv_log_macro::{debug, warn};
use libloading::{Library, Symbol};
use valor::{Loader, Plugin, Request, RequestHandler, Response};

pub(crate) struct DynLoader;

#[async_trait(?Send)]
impl Loader for DynLoader {
    async fn load(&self, plugin: &Plugin) -> Result<Box<dyn RequestHandler>, ()> {
        match plugin {
            Plugin::Native { name, path } => {
                let path = path.as_ref().unwrap_or(name);
                debug!("loading native plugin {}", path);
                let lib = Library::new(path).map_err(|e| {
                    warn!("{}", e);
                })?;

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

#[async_trait(?Send)]
impl RequestHandler for PluginContainer {
    async fn handle_request(&self, request: Request) -> Response {
        self.handler.handle_request(request).await
    }
}
