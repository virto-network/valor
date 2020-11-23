use http_types::Body;
pub use http_types::{Method, Request, Response, StatusCode, Url};
use registry::PluginRegistry;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
pub use vlugin::vlugin;

mod registry;

type Result = std::result::Result<Response, Response>;

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
pub struct Handler(Arc<PluginRegistry>);

impl Handler {
    pub fn new(loader: Arc<impl Loader>) -> Self {
        let registry = PluginRegistry::new();
        let (plugin, handler) = registry.clone().as_handler(loader);
        registry.register(plugin, handler);
        Handler(registry)
    }

    /// Handle the incoming request and send back a response
    /// from the matched plugin to the caller.
    pub async fn handle_request(&self, request: impl Into<Request>) -> Result {
        let request = request.into();
        let req_id = request
            .header("x-request-id")
            .ok_or(res(StatusCode::BadRequest, "Missing request ID"))?
            .as_str()
            .to_owned();

        let (plugin, handler) = self
            .0
            .match_plugin_handler(request.url().path())
            .ok_or(res(StatusCode::NotFound, ""))?;

        let mut response = handler.handle_request(request).await;
        response.insert_header("x-correlation-id", req_id);
        response.insert_header("x-valor-plugin", plugin.name());

        Ok(response)
    }
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        Handler(self.0.clone())
    }
}

pub trait Loader: Send + Sync + 'static {
    fn load(&self, plugin: &Plugin) -> std::result::Result<Box<dyn RequestHandler>, ()>;
}

#[inline]
pub(crate) fn res(status: StatusCode, msg: impl Into<Body>) -> Response {
    let mut res = Response::new(status);
    res.set_body(msg);
    res
}

pub trait RequestHandler: Send + Sync {
    fn handle_request(&self, request: Request) -> Pin<Box<dyn Future<Output = Response> + Send>>;
}

impl<F> RequestHandler for F
where
    F: Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync,
{
    fn handle_request(&self, request: Request) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        self(request)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Plugin {
    BuiltIn { name: String },
    Dummy,
    Native { name: String, path: Option<String> },
    WebWorker { name: String, url: Url },
}

impl Plugin {
    fn name(&self) -> String {
        match self {
            Self::Dummy => "dummy",
            Self::BuiltIn { name } => name,
            Self::Native { name, .. } => name,
            Self::WebWorker { name, .. } => name,
        }
        .into()
    }

    fn prefix(&self) -> String {
        match self {
            Self::BuiltIn { name } => ["_", name].join(""),
            Self::Dummy => "__dummy__".into(),
            Self::Native { name, .. } => name.into(),
            Self::WebWorker { name, .. } => name.into(),
        }
    }
}
