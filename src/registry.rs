use crate::{Plugin, RequestHandler};
use alloc::{borrow::ToOwned, rc::Rc, string::String};
use hashbrown::HashMap;
use path_tree::PathTree;

type PluginHandler = (Plugin, Rc<dyn RequestHandler>);

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

    pub fn register<H>(&mut self, plugin: impl Into<Plugin>, handler: H)
    where
        H: RequestHandler + 'static,
    {
        let plugin = plugin.into();
        let prefix = "/".to_owned() + plugin.prefix();
        let name = plugin.name().to_owned();

        self.routes.insert(&prefix, name.clone());
        self.routes.insert(&(prefix + "/*"), name.clone());
        self.plugins.insert(name, (plugin, Rc::new(handler)));
    }

    #[cfg(feature = "_serde")]
    pub fn get_handler<L: crate::Loader>(
        registry: Rc<core::cell::RefCell<Self>>,
        loader: Rc<L>,
    ) -> impl RequestHandler {
        RegistryHandler { registry, loader }
    }
}

#[cfg(feature = "_serde")]
struct RegistryHandler<L> {
    registry: Rc<core::cell::RefCell<PluginRegistry>>,
    loader: Rc<L>,
}

#[cfg(feature = "_serde")]
use alloc::boxed::Box;
#[cfg(feature = "_serde")]
#[async_trait::async_trait(?Send)]
impl<L: crate::Loader> RequestHandler for RegistryHandler<L> {
    async fn on_request(
        &self,
        mut request: crate::Request,
    ) -> crate::http::Result<crate::Response> {
        use crate::{http, Method::*, StatusCode};
        use alloc::vec::Vec;
        use core::result::Result::Ok;

        match request.method() {
            Get => {
                let reg = self.registry.borrow();
                let plugins = reg.plugins.values().map(|(p, _)| p).collect::<Vec<_>>();
                serde_json::to_vec(&plugins)
                    .map(|list| {
                        let mut res: http::Response = list.into();
                        res.append_header(http::headers::CONTENT_TYPE, http::mime::JSON);
                        res
                    })
                    .map_err(Into::into)
            }
            Post => {
                let plugin = request.body_json().await?;
                self.loader.load(&plugin).await?;
                Ok(StatusCode::Created.into())
            }
            _ => Ok(StatusCode::MethodNotAllowed.into()),
        }
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
