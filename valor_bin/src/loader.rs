use async_trait::async_trait;
use kv_log_macro::{debug, warn};
use libloading::{Library, Symbol};
use std::{cell::RefCell, collections::HashMap, pin::Pin, rc::Rc};
use valor::{Error, LoadError, Vlugin, VluginFactory};

#[derive(Default)]
pub(crate) struct Loader {
    plugins: RefCell<HashMap<String, Rc<Library>>>,
}

#[async_trait(?Send)]
impl valor::Loader for Loader {
    async fn load(&self, plugin: &valor::Plugin) -> Result<VluginFactory, valor::LoadError> {
        match plugin {
            valor::Plugin::Native { name, path, .. } => {
                if let Some(factory) = self.get_factory(name) {
                    return Ok(factory);
                }

                let path = path.as_ref().unwrap_or(name);
                debug!("loading native plugin {}", path);
                let lib = Library::new(path).map_err(|e| {
                    warn!("{}", e);
                    LoadError::NotFound
                })?;

                {
                    self.plugins.borrow_mut().insert(name.into(), Rc::new(lib));
                }

                self.get_factory(name).ok_or(LoadError::NotFound)
            }
            _ => Err(LoadError::NotSupported),
        }
    }
}

type Factory<'a> =
    fn() -> Pin<Box<dyn core::future::Future<Output = Result<Box<dyn Vlugin>, Error>> + 'a>>;

impl Loader {
    fn get_factory(&self, name: &str) -> Option<VluginFactory> {
        let lib = self.plugins.borrow().get(name)?.clone();
        let name = name.to_owned();

        Some(Box::new(move || {
            let lib = lib.clone();
            let name = name.clone();
            Box::pin(async move {
                let lib = lib.clone();
                let factory: Symbol<'_, Factory> =
                    unsafe { lib.get(b"instantiate_vlugin") }.expect("Plugin interface");
                debug!("{} {:?}", name, factory);
                factory().await
            })
        }))
    }
}
