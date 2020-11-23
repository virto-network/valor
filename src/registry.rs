use crate::{res, HandlerResponse, Loader, Method, Plugin, Request, RequestHandler, StatusCode};
use path_tree::PathTree;
use serde_json as json;
use std::collections::HashMap;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};

/// Plugin to keep track of registered plugins
pub(crate) struct PluginRegistry {
    plugins: Mutex<HashMap<String, (Plugin, Arc<dyn RequestHandler>)>>,
    routes: Mutex<PathTree<String>>,
}

impl PluginRegistry {
    const NAME: &'static str = "plugins";

    pub fn new() -> Arc<Self> {
        Arc::new(PluginRegistry {
            plugins: Mutex::new(HashMap::new()),
            routes: Mutex::new(PathTree::new()),
        })
    }

    pub fn match_plugin_handler(&self, path: &str) -> Option<(Plugin, Arc<dyn RequestHandler>)> {
        let routes = self.routes.lock().unwrap();
        let plugins = self.plugins.lock().unwrap();
        let (name, _) = routes.find(path)?;
        let (plugin, handler) = plugins.get(name)?;
        Some((plugin.clone(), handler.clone()))
    }

    pub fn register(&self, plugin: Plugin, handler: Box<dyn RequestHandler>) {
        let mut routes = self.routes.lock().unwrap();
        let mut plugins = self.plugins.lock().unwrap();
        routes.insert(&plugin.prefix(), plugin.name());
        plugins.insert(plugin.name().into(), (plugin, handler.into()));
    }

    fn plugin_list(&self) -> Vec<Plugin> {
        self.plugins
            .lock()
            .unwrap()
            .values()
            .map(|(p, _)| p.clone())
            .collect()
    }

    pub fn as_handler(
        self: Arc<Self>,
        loader: Arc<impl Loader>,
    ) -> (Plugin, Box<dyn RequestHandler>) {
        (
            Plugin::BuiltIn {
                name: Self::NAME.into(),
            },
            Box::new(move |mut req: Request| {
                let registry = self.clone();
                let loader = loader.clone();
                Box::pin(async move {
                    match req.method() {
                        Method::Get => {
                            let plugins = registry.plugin_list();
                            json::to_vec(&plugins)
                                .map_or(res(StatusCode::InternalServerError, ""), |list| {
                                    list.into()
                                })
                        }
                        Method::Post => match req.body_json().await {
                            Ok(plugin) => {
                                if let Ok(handler) = loader.load(&plugin) {
                                    registry.register(plugin, handler);
                                    res(StatusCode::Created, "")
                                } else {
                                    res(StatusCode::UnprocessableEntity, "")
                                }
                            }
                            Err(_) => res(StatusCode::BadRequest, ""),
                        },
                        _ => res(StatusCode::MethodNotAllowed, ""),
                    }
                }) as HandlerResponse
            }),
        )
    }
}
