use fast_async_mutex::mutex::Mutex;
pub use http_types::{Error, Method, Request, Response, Result, StatusCode, Url};
use instant::Instant;
use kv_log_macro::{debug, info};
use registry::PluginRegistry;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;

mod registry;

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
    pub async fn new() -> Self {
        Handler(PluginRegistry::new().await)
    }

    /// Handle the incoming request and send back a response
    /// from the matched plugin to the caller.
    pub async fn handle_request(&self, request: impl Into<Request>) -> Result<Response> {
        let instant = Instant::now();
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
        let method = request.method();
        debug!("received request {} {}", method, path, { id: req_id.as_str() });

        let (plugin, handler) = {
            let registry = self.0.lock().await;
            registry
                .match_plugin_handler(&path)
                .ok_or(Error::from_str(StatusCode::NotFound, "no plugin matched"))?
        };

        let plugin = plugin.name();
        debug!("matched plugin \"{}\"", plugin);

        let mut response = handler.handle_request(request).await;
        let status: u16 = response.status().into();
        info!("[{}] {} {} {}", plugin, status, method, path, {
            req_id: req_id.as_str(), status: status, dur: instant.elapsed().as_nanos() as u64
        });
        response.insert_header("x-correlation-id", req_id);
        Ok(response)
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Plugin {
    BuiltIn { name: String },
    WebWorker { name: String, url: Url },
    Dummy,
}

impl Plugin {
    fn name(&self) -> String {
        match self {
            Self::Dummy => "dummy",
            Self::BuiltIn { name } => name,
            Self::WebWorker { name, .. } => name,
        }
        .into()
    }

    fn prefix(&self) -> String {
        match self {
            Self::BuiltIn { name } => ["_", name].join(""),
            Self::Dummy => "__dummy__".into(),
            Self::WebWorker { name, .. } => name.into(),
        }
    }
}
