use kv_log_macro::debug;
use libloading::{Library, Symbol};
use valor::{Loader, Plugin, RequestHandler};

pub(crate) struct DynLoader;

impl Loader for DynLoader {
    fn load(&self, plugin: &Plugin) -> Result<Box<dyn RequestHandler>, ()> {
        match plugin {
            Plugin::Native { name, path } => {
                let path = path.as_ref().unwrap_or(name);
                debug!("loading native plugin {}", path);

                let lib = Library::new(path).map_err(|_| ())?;
                let plugin: Symbol<fn() -> _> =
                    unsafe { lib.get(b"_request_handler") }.map_err(|_| ())?;

                Ok(plugin())
            }
            _ => Err(()),
        }
    }
}
