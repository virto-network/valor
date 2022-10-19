use async_trait::async_trait;
use kv_log_macro::{debug, warn};
use libloading::{library_filename, Library, Symbol};
use std::{cell::RefCell, collections::HashMap, pin::Pin, rc::Rc};
use valor::{runtime, Vlugin, VluginConfig};

#[derive(Default)]
pub(crate) struct Loader {
    plugins: RefCell<HashMap<String, Rc<Library>>>,
}

#[async_trait(?Send)]
impl runtime::Loader for Loader {
    async fn load(
        &self,
        plugin: &runtime::VluginDef,
    ) -> Result<runtime::VluginFactory, runtime::Error> {
        match &plugin.r#type {
            runtime::VluginType::Native { path } => {
                let name = &plugin.name;
                if let Some(factory) = self.get_factory(name) {
                    return Ok(factory);
                }

                let path = path
                    .as_ref()
                    .map(Into::into)
                    .unwrap_or_else(|| library_filename(name));
                debug!("loading native plugin {}({})", name, path.to_string_lossy());
                let lib = unsafe { Library::new(path) }.map_err(|e| {
                    warn!("{}", e);
                    runtime::Error::LoadVlugin(name.to_owned())
                })?;

                {
                    self.plugins.borrow_mut().insert(name.into(), Rc::new(lib));
                }

                self.get_factory(name)
                    .ok_or(runtime::Error::LoadVlugin(name.to_owned()))
            }
            ty => Err(runtime::Error::VluginNotSupported(ty.to_owned())),
        }
    }
}

type Factory<'a> = fn(
    Option<VluginConfig>,
) -> Pin<
    Box<dyn core::future::Future<Output = Result<Box<dyn Vlugin>, valor::Error>> + 'a>,
>;

impl Loader {
    fn get_factory(&self, name: &str) -> Option<runtime::VluginFactory> {
        let lib = self.plugins.borrow().get(name)?.clone();
        let name = name.to_owned();

        Some(Box::new(move |cfg| {
            let lib = lib.clone();
            let name = name.clone();
            Box::pin(async move {
                let lib = lib.clone();
                let factory: Symbol<'_, Factory> =
                    unsafe { lib.get(b"instantiate_vlugin") }.expect("Plugin interface");
                debug!("{} {:?}", name, factory);
                factory(cfg).await
            })
        }))
    }
}
