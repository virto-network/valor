use super::VluginDef;
use crate::Vlugin;
use alloc::{borrow::ToOwned, rc::Rc, string::String};
use hashbrown::HashMap;
use path_tree::PathTree;

type PluginHandler = (VluginDef, Rc<dyn Vlugin>);

/// Plugin to keep track of registered plugins
pub(crate) struct PluginRegistry {
    pub(self) plugins: HashMap<String, PluginHandler>,
    routes: PathTree<String>,
}

#[derive(Debug)]
pub(crate) struct RegistrationError;

impl PluginRegistry {
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
            routes: PathTree::new(),
        }
    }

    pub fn match_vlugin(&self, path: &str) -> Option<PluginHandler> {
        let (name, _) = self.routes.find(path)?;
        let (plugin, handler) = self.plugins.get(name)?;
        Some((plugin.clone(), handler.clone()))
    }

    pub fn register<H: Vlugin + 'static>(
        &mut self,
        plugin: VluginDef,
        handler: H,
    ) -> Result<(), RegistrationError> {
        if self.plugins.contains_key(&plugin.name) {
            return Err(RegistrationError);
        }
        let prefix = "/".to_owned() + plugin.prefix_or_name();

        self.routes.insert(&prefix, plugin.name.clone());
        self.routes.insert(&(prefix + "/*"), plugin.name.clone());
        self.plugins
            .insert(plugin.name.clone(), (plugin, Rc::new(handler)));
        Ok(())
    }

    #[cfg(feature = "serde")]
    pub fn get_handler<L: super::Loader>(
        registry: Rc<core::cell::RefCell<Self>>,
        loader: Rc<L>,
    ) -> impl crate::Vlugin {
        RegistryHandler { registry, loader }
    }
}

#[cfg(feature = "serde")]
use alloc::boxed::Box;
#[cfg(feature = "serde")]
struct RegistryHandler<L> {
    registry: Rc<core::cell::RefCell<PluginRegistry>>,
    loader: Rc<L>,
}

#[cfg(feature = "serde")]
#[async_trait::async_trait(?Send)]
impl<L> crate::Vlugin for RegistryHandler<L>
where
    L: super::Loader,
{
    async fn on_msg(&self, msg: crate::Message) -> Result<crate::Answer, crate::Error> {
        use crate::{
            http::{headers, mime, Error, Method::*, Response, StatusCode},
            Message,
        };
        use alloc::vec::Vec;
        use core::result::Result::Ok;

        let mut request = match msg {
            Message::Http(req) => req,
            Message::Ping => return Err(crate::Error::NotSupported),
        };

        match request.method() {
            Get => {
                let reg = self.registry.borrow();
                let plugins = reg.plugins.values().map(|(p, _)| p).collect::<Vec<_>>();
                serde_json::to_vec(&plugins)
                    .map(|list| {
                        let mut res: Response = list.into();
                        res.append_header(headers::CONTENT_TYPE, mime::JSON);
                        res.into()
                    })
                    .map_err(|e| Error::new(StatusCode::InternalServerError, e).into())
            }
            Post => {
                let mut plugin: VluginDef = request.body_json().await?;
                let name = plugin.name.clone();
                let factory = self.loader.load(&plugin).await?;
                let handler = factory(plugin.config.take()).await?;
                self.registry
                    .borrow_mut()
                    .register(plugin, handler)
                    .map_err(|_| Error::from_str(StatusCode::Conflict, name + " already exists"))?;
                let res: Response = StatusCode::Created.into();
                Ok(res.into())
            }
            _ => {
                let res: Response = StatusCode::MethodNotAllowed.into();
                Ok(res.into())
            }
        }
    }

    fn context(&self) -> &crate::Context {
        unimplemented!()
    }
    fn context_mut(&mut self) -> &mut crate::Context {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_multiple_times_gives_an_error() {
        let mut registry = PluginRegistry::new();
        registry.register("foo".into(), ()).unwrap();
        let res = registry.register("foo".into(), ());
        assert!(res.is_err());
        assert_eq!(registry.plugins.len(), 1);
    }

    #[test]
    fn match_with_leading_slash() {
        let mut registry = PluginRegistry::new();
        registry.register("foo".into(), ()).unwrap();
        let handler = registry.match_vlugin("/_foo/");
        assert!(handler.is_some());
    }

    #[test]
    fn match_without_leading_slash() {
        let mut registry = PluginRegistry::new();
        registry.register("foo".into(), ()).unwrap();
        let handler = registry.match_vlugin("/_foo");
        assert!(handler.is_some());
    }

    #[test]
    fn match_all_after_prefix() {
        let mut registry = PluginRegistry::new();
        registry.register("foo".into(), ()).unwrap();
        let handler = registry.match_vlugin("/_foo/bar");
        assert!(handler.is_some());
        let handler = registry.match_vlugin("/_foo/bar/");
        assert!(handler.is_some());
        let handler = registry.match_vlugin("/_foo/bar/baz");
        assert!(handler.is_some());
    }
}
