use crate::{Method, Plugin, Request, RequestHandler, Response, StatusCode};
use fast_async_mutex::mutex::Mutex;
use path_tree::PathTree;
use serde_json::to_string;
use std::collections::HashMap;
use std::iter::Iterator;
use std::sync::Arc;

/// Plugin to keep track of registered plugins
pub(crate) struct PluginRegistry {
    plugins: HashMap<String, (Plugin, Arc<dyn RequestHandler>)>,
    routes: PathTree<String>,
}

impl PluginRegistry {
    const NAME: &'static str = "plugins";

    pub(crate) async fn new() -> Arc<Mutex<Self>> {
        let registry = Arc::new(Mutex::new(PluginRegistry {
            plugins: HashMap::new(),
            routes: PathTree::new(),
        }));

        // plugin registry registers itself as a plugin
        let reg_clone = registry.clone();
        registry.clone().lock().await.register(
            Plugin::BuiltIn {
                name: Self::NAME.into(),
            },
            Arc::new(move |mut req: Request| {
                let registry = reg_clone.clone();
                async move {
                    match req.method() {
                        Method::Get => {
                            let plugins = registry.lock().await.plugin_list().collect::<Vec<_>>();
                            to_string(&plugins)
                                .map_or(Response::new(StatusCode::InternalServerError), |list| {
                                    list.into()
                                })
                        }
                        Method::Post => match req.body_json().await {
                            Ok(plugin) => {
                                let mut registry = registry.lock().await;
                                let handler = registry.get_handler(&plugin);
                                registry.register(plugin, handler);
                                StatusCode::Created.into()
                            }
                            Err(_) => Response::new(StatusCode::BadRequest),
                        },
                        _ => StatusCode::MethodNotAllowed.into(),
                    }
                }
            }),
        );
        registry
    }

    fn register(&mut self, plugin: Plugin, handler: Arc<dyn RequestHandler>) {
        self.routes.insert(&plugin.prefix(), plugin.name());
        self.plugins.insert(plugin.name().into(), (plugin, handler));
    }

    pub(crate) fn match_plugin_handler(
        &self,
        path: &str,
    ) -> Option<(Plugin, Arc<dyn RequestHandler>)> {
        let (name, _) = self.routes.find(path)?;
        let (plugin, handler) = self.plugins.get(name)?;
        Some((plugin.clone(), handler.clone()))
    }

    fn plugin_list(&self) -> impl Iterator<Item = Plugin> + '_ {
        self.plugins.values().map(|(p, _)| p.clone())
    }

    fn get_handler(&self, plugin: &Plugin) -> Arc<dyn RequestHandler> {
        match plugin {
            Plugin::BuiltIn { name } => match name.as_str() {
                "plugins" => unreachable!(),
                _ => todo!(),
            },
            Plugin::WebWorker { .. } => todo!(),
            Plugin::Dummy => {
                Arc::new(|_req: Request| async { "hello dummy".into() }) as Arc<dyn RequestHandler>
            }
        }
    }
}
