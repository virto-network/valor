use crate::{Loader, Method, Plugin, Request, RequestHandler, Response, StatusCode};
use path_tree::PathTree;
use serde_json as json;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

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

    pub fn register(&mut self, plugin: Plugin, handler: Box<dyn RequestHandler>) {
        self.routes.insert(&plugin.prefix(), plugin.name().into());
        self.plugins
            .insert(plugin.name().into(), (plugin, handler.into()));
    }

    fn plugin_list(&self) -> Vec<Plugin> {
        self.plugins.values().map(|(p, _)| p.clone()).collect()
    }

    pub fn as_handler<L: Loader>(
        registry: Rc<RefCell<Self>>,
        loader: Rc<L>,
    ) -> impl RequestHandler {
        RegistryHandler { registry, loader }
    }
}

struct RegistryHandler<L> {
    registry: Rc<RefCell<PluginRegistry>>,
    loader: Rc<L>,
}

#[async_trait::async_trait(?Send)]
impl<L: Loader> RequestHandler for RegistryHandler<L> {
    async fn handle_request(&self, mut request: Request) -> Response {
        match request.method() {
            Method::Get => {
                let plugins = self.registry.borrow().plugin_list();
                json::to_vec(&plugins).map_or(res!(StatusCode::InternalServerError), |list| {
                    res!(list, {
                        content_type: "application/json",
                    })
                })
            }
            Method::Post => match request.body_json().await {
                Ok(plugin) => match self.loader.load(&plugin).await {
                    Ok(handler) => {
                        self.registry
                            .borrow_mut()
                            .register(plugin, Box::new(handler));
                        res!(StatusCode::Created)
                    }
                    Err(_) => {
                        res!(StatusCode::UnprocessableEntity, "Can't load plugin")
                    }
                },
                Err(e) => res!(StatusCode::BadRequest, e.to_string()),
            },
            _ => res!(StatusCode::MethodNotAllowed),
        }
    }
}
