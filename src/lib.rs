//! Valor

pub use async_trait::async_trait;
pub use http_types::{Method, StatusCode, Url};
use registry::PluginRegistry;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::sync::Arc;
#[cfg(feature = "util")]
pub use util::*;
#[cfg(feature = "util")]
mod util;

// short-hand for creating or modifiying simple responses
macro_rules! res {
    () => { res!(http_types::StatusCode::Ok) };
    ($res:expr) => { res!($res, "") };
    ($res:expr, { $($h:ident : $v:expr),* $(,)? }) => { res!($res, "", { $($h : $v),* }) };
    ($res:expr, $b:expr) => { res!($res, $b, {}) };
    ($res:expr, $b:expr, { $($h:ident : $v:expr),* $(,)? }) => {{
        let mut res: http_types::Response = $res.into();
        let body: http_types::Body = $b.into();
        if body.len().is_some() && !body.is_empty().unwrap() {
            res.set_body($b);
        }
        $(
            res.insert_header(stringify!($h).replace("_", "-").as_str(), $v);
        )*
        res
    }};
}

mod registry;

pub type Request = http_types::Request;
pub type Response = http_types::Response;
type Result = core::result::Result<Response, Response>;

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
    /// Creates a new `Handler` instance
    pub fn new(loader: Arc<impl Loader>) -> Self {
        let registry = PluginRegistry::new();
        let handler = registry.clone().as_handler(loader);
        registry.register(BuiltInPlugin::Registry.into(), handler);
        registry.register(
            BuiltInPlugin::Health.into(),
            Box::new(|_| async { res!() }) as Box<dyn RequestHandler>,
        );
        Handler(registry)
    }

    /// Handle the incoming request and send back a response
    /// from the matched plugin to the caller.
    pub async fn handle_request(&self, request: impl Into<Request>) -> Result {
        let request = request.into();
        let req_id = request
            .header("x-request-id")
            .ok_or_else(|| res!(StatusCode::BadRequest, "Missing request ID"))?
            .as_str()
            .to_owned();

        let (plugin, handler) = self
            .0
            .match_plugin_handler(request.url().path())
            .ok_or_else(|| res!(StatusCode::NotFound, { x_correlation_id: &req_id }))?;

        Ok(res!(handler.handle_request(request).await, {
            x_correlation_id: req_id,
            x_vlugin: plugin.name()
        }))
    }
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        Handler(self.0.clone())
    }
}

impl fmt::Debug for Handler
where
    for<'a> dyn RequestHandler + 'a: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Handler").field(&self.0).finish()
    }
}

/// Loader
#[async_trait(?Send)]
pub trait Loader: 'static {
    /// Loads the given `plugin`
    async fn load(&self, plugin: &Plugin) -> LoadResult;
}

pub type LoadResult = std::result::Result<Box<dyn RequestHandler>, LoadError>;

pub enum LoadError {
    NotSupported,
    NotFound,
    BadFormat,
}

impl From<LoadError> for LoadResult {
    fn from(err: LoadError) -> Self {
        Err(err)
    }
}

/// Request handler
#[async_trait(?Send)]
pub trait RequestHandler {
    /// Handles the request
    async fn handle_request(&self, request: Request) -> Response;
}

#[async_trait(?Send)]
impl<F, R> RequestHandler for F
where
    F: Fn(Request) -> R,
    R: Future<Output = Response> + 'static,
{
    async fn handle_request(&self, request: Request) -> Response {
        self(request).await
    }
}

/// Plugin information
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Plugin {
    /// Built in
    BuiltIn(BuiltInPlugin),
    /// Native
    Native {
        /// Name
        name: String,
        /// Path
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// Web script or WASM
    Web {
        /// Name
        name: String,
        /// Url of the JS script
        url: Url,
    },
}

impl Plugin {
    fn name(&self) -> &str {
        match self {
            Self::BuiltIn(p) => p.name(),
            Self::Native { name, .. } => name,
            Self::Web { name, .. } => name,
        }
        .into()
    }

    fn prefix(&self) -> &str {
        match self {
            Self::BuiltIn(p) => p.prefix(),
            Self::Native { name, .. } => name,
            Self::Web { name, .. } => name,
        }
    }
}

impl From<BuiltInPlugin> for Plugin {
    fn from(p: BuiltInPlugin) -> Self {
        Self::BuiltIn(p)
    }
}

/// Plugins included with the runtime
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum BuiltInPlugin {
    Registry,
    Health,
}

impl BuiltInPlugin {
    fn name(&self) -> &str {
        match self {
            Self::Registry => "plugin_registry",
            Self::Health => "health",
        }
    }

    fn prefix(&self) -> &str {
        match self {
            Self::Registry => "_plugins",
            Self::Health => "_health",
        }
    }
}
