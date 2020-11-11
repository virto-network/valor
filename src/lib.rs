//use fast_async_mutex::mutex::Mutex;
pub use http_types::{Error, Method, Request, Response, Result, StatusCode, Url};
use log::{debug, info};
use path_tree::PathTree;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::collections::HashMap;
use std::future::Future;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};

/// Handler is the main entry point for dispatching incoming requests
/// to registered plugins under a specific URL pattern.
///
/// ```
/// # use http_types::{StatusCode, Request};
/// let handler = Handler::new();
/// let request = Request::new();
/// let res = handler.handle_request(request).await?;
/// assert_eq(res, StatusCode::Ok);
/// ```
pub struct Handler(Arc<Mutex<PluginRegistry>>);

impl Handler {
    pub fn new() -> Self {
        Handler(PluginRegistry::new())
    }

    /// Handle the incoming request and send back a response
    /// from the matched plugin to the caller.
    pub async fn handle_request(&self, request: impl Into<Request>) -> Result<Response> {
        let request = request.into();
        let req_id = request
            .header("x-request-id")
            .ok_or(Error::from_str(
                StatusCode::BadRequest,
                "missing request ID",
            ))?
            .as_str()
            .to_owned();
        let path = request.url().path().to_owned();
        let reg = self.0.lock().expect("accuire lock");
        match reg.get(&path) {
            Some((plugin, handler)) => {
                let method = request.method();
                let mut response = handler.handle_request(request).await;
                response.insert_header("x-correlation-id", req_id);
                info!(
                    "[{}]:{} {}:{}",
                    plugin.name(),
                    method,
                    path,
                    response.status()
                );
                Ok(response)
            }
            None => {
                debug!("No plugin matched for {} {}", request.method(), path);
                Err(Error::from_str(StatusCode::NotFound, "no plugin matched").into())
            }
        }
    }
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        Handler(self.0.clone())
    }
}

#[async_trait::async_trait]
pub trait RequestHandler: Send + Sync + 'static {
    async fn handle_request(&self, request: Request) -> Response;
}

#[async_trait::async_trait]
impl<F, U> RequestHandler for F
where
    F: Fn(Request) -> U + Send + Sync + 'static,
    U: Future<Output = Response> + Send + 'static,
{
    async fn handle_request(&self, request: Request) -> Response {
        self(request).await
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Plugin {
    BuiltIn(String),
    WebWorker { name: String, url: Url },
    Dummy,
}

impl Plugin {
    fn name(&self) -> String {
        match self {
            Self::Dummy => "dummy",
            Self::BuiltIn(name) => name,
            Self::WebWorker { name, .. } => name,
        }
        .into()
    }

    fn prefix(&self) -> String {
        match self {
            Self::BuiltIn(name) => format!("/_{}", name),
            _ => "".into(),
        }
    }
}

/// Plugin to keep track of registered plugins
struct PluginRegistry {
    plugins: HashMap<String, (Plugin, Box<dyn RequestHandler>)>,
    routes: PathTree<String>,
}

impl PluginRegistry {
    const NAME: &'static str = "plugins";

    fn new() -> Arc<Mutex<Self>> {
        let registry = Arc::new(Mutex::new(PluginRegistry {
            plugins: HashMap::new(),
            routes: PathTree::new(),
        }));

        // plugin registry registers itself as a plugin
        let reg_clone = registry.clone();
        registry.clone().lock().unwrap().register(
            Plugin::BuiltIn(Self::NAME.into()),
            Box::new(move |mut req: Request| {
                let registry = reg_clone.clone();
                async move {
                    match req.method() {
                        Method::Get => {
                            let registry = registry.lock().unwrap();
                            let plugins = registry.plugin_list().collect::<Vec<_>>();
                            to_string(&plugins)
                                .map_or(Response::new(StatusCode::BadRequest), |list| list.into())
                        }
                        Method::Post => match req.body_json().await {
                            Ok(plugin) => {
                                let mut registry = registry.lock().unwrap();
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

    fn register(&mut self, plugin: Plugin, handler: Box<dyn RequestHandler>) {
        self.routes.insert(&plugin.prefix(), plugin.name());
        self.plugins.insert(plugin.name().into(), (plugin, handler));
    }

    fn get(&self, path: &str) -> Option<(&Plugin, &dyn RequestHandler)> {
        use std::borrow::Borrow;

        let (name, _) = self.routes.find(path)?;
        let (plugin, handler) = self.plugins.get(name)?;
        Some((plugin, handler.borrow()))
    }

    fn plugin_list(&self) -> impl Iterator<Item = Plugin> + '_ {
        self.plugins.values().map(|(p, _)| p.clone())
    }

    fn get_handler(&self, plugin: &Plugin) -> Box<dyn RequestHandler> {
        match plugin {
            Plugin::BuiltIn(name) => match name.as_str() {
                "plugins" => unreachable!(),
                _ => todo!(),
            },
            Plugin::WebWorker { .. } => todo!(),
            Plugin::Dummy => {
                Box::new(|_req: Request| async { "hello dummy".into() }) as Box<dyn RequestHandler>
            }
        }
    }
}
