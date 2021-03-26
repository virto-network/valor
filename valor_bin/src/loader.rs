use async_trait::async_trait;
use kv_log_macro::{debug, warn};
use libloading::{Library, Symbol};
use valor::{http, LoadError, LoadResult, Loader, Plugin, RequestHandler};

pub(crate) struct DynLoader;

#[async_trait(?Send)]
impl Loader for DynLoader {
    type Handler = PluginContainer;

    async fn load(&self, plugin: &Plugin) -> LoadResult<Self> {
        match plugin {
            Plugin::Native { name, path, .. } => {
                let path = path.as_ref().unwrap_or(name);
                debug!("loading native plugin {}", path);
                let lib = Library::new(path).map_err(|e| {
                    warn!("{}", e);
                    LoadError::NotFound
                })?;

                let get_request_handler: Symbol<'_, fn() -> _> =
                    unsafe { lib.get(b"get_request_handler") }.map_err(|_| LoadError::BadFormat)?;
                debug!("symbol {:?}", plugin);

                let handler = get_request_handler();

                Ok(PluginContainer { handler, _lib: lib })
            }
            _ => Err(LoadError::NotSupported),
        }
    }
}

pub(crate) struct PluginContainer {
    handler: Box<dyn RequestHandler>,
    _lib: Library,
}

#[async_trait(?Send)]
impl RequestHandler for PluginContainer {
    async fn on_request(&self, request: http::Request) -> http::Result<http::Response> {
        self.handler.on_request(request).await
    }
}
