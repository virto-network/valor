use crate::{Vlugin, VluginInfo};
use alloc::{borrow::ToOwned, rc::Rc, string::String};
use hashbrown::HashMap;
use path_tree::PathTree;

type PluginHandler = (VluginInfo, Rc<dyn Vlugin>);

/// Plugin to keep track of registered plugins
pub(crate) struct PluginRegistry {
    plugins: HashMap<String, PluginHandler>,
    routes: PathTree<String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        PluginRegistry {
            plugins: HashMap::new(),
            routes: PathTree::new(),
        }
    }

    pub fn match_plugin_handler(&self, path: &str) -> Option<PluginHandler> {
        let (name, _) = self.routes.find(path)?;
        let (plugin, handler) = self.plugins.get(name)?;
        Some((plugin.clone(), handler.clone()))
    }

    pub fn register<H: Vlugin + 'static>(&mut self, plugin: impl Into<VluginInfo>, handler: H) {
        let plugin = plugin.into();
        let prefix = "/".to_owned() + plugin.prefix_or_name();

        self.routes.insert(&prefix, plugin.name.clone());
        self.routes.insert(&(prefix + "/*"), plugin.name.clone());
        self.plugins
            .insert(plugin.name.clone(), (plugin, Rc::new(handler)));
    }

    #[cfg(feature = "_serde_")]
    pub fn get_handler<L: crate::Loader>(
        registry: Rc<core::cell::RefCell<Self>>,
        loader: Rc<L>,
    ) -> impl crate::Vlugin {
        RegistryHandler { registry, loader }
    }
}

#[cfg(feature = "_serde_")]
use alloc::boxed::Box;
#[cfg(feature = "_serde_")]
struct RegistryHandler<L> {
    registry: Rc<core::cell::RefCell<PluginRegistry>>,
    loader: Rc<L>,
}

#[cfg(feature = "_serde_")]
#[async_trait::async_trait(?Send)]
impl<L> crate::Vlugin for RegistryHandler<L>
where
    L: crate::Loader,
{
    async fn on_msg(&self, msg: crate::Message) -> Result<crate::Answer, crate::Error> {
        use crate::{
            http::{headers, mime, Method::*, Response, StatusCode},
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
                    .map_err(|e| crate::Error::Http(e.into()))
            }
            Post => {
                let plugin: VluginInfo = request.body_json().await?;
                let factory = self.loader.load(&plugin).await?;
                let handler = factory().await?;
                self.registry.borrow_mut().register(plugin, handler);
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
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_with_leading_slash() {
        let mut registry = PluginRegistry::new();
        registry.register("foo", ());
        let handler = registry.match_plugin_handler("/_foo/");
        assert!(handler.is_some());
    }

    #[test]
    fn match_without_leading_slash() {
        let mut registry = PluginRegistry::new();
        registry.register("foo", ());
        let handler = registry.match_plugin_handler("/_foo");
        assert!(handler.is_some());
    }

    #[test]
    fn match_all_after_prefix() {
        let mut registry = PluginRegistry::new();
        registry.register("foo", ());
        let handler = registry.match_plugin_handler("/_foo/bar");
        assert!(handler.is_some());
        let handler = registry.match_plugin_handler("/_foo/bar/");
        assert!(handler.is_some());
        let handler = registry.match_plugin_handler("/_foo/bar/baz");
        assert!(handler.is_some());
    }
}
