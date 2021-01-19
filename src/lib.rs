//! ## Valor
//!
//! A lightweight HTTP plugin system that runs in the server and the browser.
//!
//! - Use `valor_bin` to run your Rust and JS(soon!) plugins in the server.
//! - Use `valor_web` as a script imported from the main document or a worker
//! in your web application to have a local API powered by a service worker.

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
#[cfg(feature = "util")]
mod util;

pub use async_trait::async_trait;
pub use http_types::{Method, StatusCode, Url};
use registry::PluginRegistry;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::{cell::RefCell, rc::Rc};
#[cfg(feature = "util")]
pub use util::*;

pub type Request = http_types::Request;
pub type Response = http_types::Response;
type Result = core::result::Result<Response, Response>;

/// The main entry point for dispatching incoming requests
/// to plugins registered under a specific URL prefix.
///
/// ```
/// # use valor_core::*;
/// # #[async_std::main] async fn main() { test().await }
/// # async fn test() {
/// let handler = Handler::new(()).with_plugin("foo", ());
///
/// let request = Request::new(Method::Get, "http://example.com/foo/123");
/// let res = handler.handle_request(request).await;
///
/// assert_eq!(res.status(), StatusCode::Ok);
/// # }
/// ```
pub struct Handler<L> {
    registry: Rc<RefCell<PluginRegistry>>,
    loader: Rc<L>,
}

impl<L: Loader + 'static> Handler<L> {
    /// Creates a new `Handler` instance
    pub fn new(loader: L) -> Self {
        Handler {
            registry: Rc::new(RefCell::new(PluginRegistry::new())),
            loader: Rc::new(loader),
        }
    }

    /// Adds a plugin with its handler to the internal registry
    pub fn with_plugin<H>(&self, plugin: impl Into<Plugin>, handler: H)
    where
        H: RequestHandler + 'static,
    {
        self.registry
            .borrow_mut()
            .register(plugin.into(), Box::new(handler));
    }

    /// Using the configured loader, loads a plugin to
    pub async fn load_plugin(&self, plugin: Plugin) -> core::result::Result<(), LoadError> {
        let handler = self.loader.load(&plugin).await?;
        self.with_plugin(plugin, handler);
        Ok(())
    }

    /// Expose the plugin registry as an endpoint on `_plugins` to add more plugins dynamically
    pub fn with_registry(self) -> Self {
        self.with_plugin(
            Plugin::Static {
                name: "registry".into(),
                prefix: Some("_plugins".into()),
            },
            PluginRegistry::as_handler(self.registry.clone(), self.loader.clone()),
        );
        self
    }

    /// Include the built-in health plugin that returns _Ok_ on `_health`
    pub fn with_health(self) -> Self {
        self.with_plugin("health", ());
        self
    }

    /// Handles an incoming request by answering form a plugin that matches the URL pattern
    ///
    /// It requires the request to specify a `x-request-id` header that is set back on
    /// the response as `x-correlation-id`(e.g. used by valor_web to match requests and responses)
    pub async fn handle_request(&self, request: impl Into<Request>) -> Result {
        let request = request.into();
        let req_id = request
            .header("x-request-id")
            .ok_or_else(|| res!(StatusCode::BadRequest, "Missing request ID"))?
            .as_str()
            .to_owned();

        let (plugin, handler) = self
            .registry
            .borrow()
            .match_plugin_handler(request.url().path())
            .ok_or_else(|| res!(StatusCode::NotFound, { x_correlation_id: &req_id }))?;

        Ok(res!(handler.handle_request(request).await, {
            x_correlation_id: req_id,
            x_valor_plugin: plugin.name()
        }))
    }
}

impl<L> Clone for Handler<L> {
    fn clone(&self) -> Self {
        Handler {
            registry: self.registry.clone(),
            loader: self.loader.clone(),
        }
    }
}

/// A Loader can fetch plugin handlers from various sources
/// such as the network or the file system
#[async_trait(?Send)]
pub trait Loader: 'static {
    type Handler: RequestHandler;

    /// Loads the given `plugin`
    async fn load(&self, plugin: &Plugin) -> LoadResult<Self>;
}

pub type LoadResult<L> = core::result::Result<<L as Loader>::Handler, LoadError>;

#[derive(Debug)]
pub enum LoadError {
    NotSupported,
    NotFound,
    BadFormat,
}

/// A dummy loader
#[async_trait(?Send)]
impl Loader for () {
    type Handler = ();
    async fn load(&self, _plugin: &Plugin) -> LoadResult<Self> {
        Ok(())
    }
}

/// Request handlers only job is to respond to http requests
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

/// A dummy handler
#[async_trait(?Send)]
impl RequestHandler for () {
    /// Handles the request
    async fn handle_request(&self, _request: Request) -> Response {
        StatusCode::Ok.into()
    }
}

/// Plugin information
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Plugin {
    /// Plugin that comes with the runtime
    Static {
        name: String,
        prefix: Option<String>,
    },
    /// Natively compiled Rust plugin
    Native {
        /// Name
        name: String,
        /// Path
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        /// Url prefix where the plugin is mounted, defaults to the name
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
    },
    /// Web script or WASM
    Web {
        /// Name
        name: String,
        /// Url of the JS script
        url: Url,
        /// Url prefix where the plugin is mounted, defaults to the name
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
    },
}

impl Plugin {
    fn name(&self) -> &str {
        &match self {
            Self::Static { name, .. } => name,
            Self::Native { name, .. } => name,
            Self::Web { name, .. } => name,
        }
    }

    fn prefix(&self) -> &str {
        match self {
            Self::Static { prefix, .. } => prefix,
            Self::Native { prefix, .. } => prefix,
            Self::Web { prefix, .. } => prefix,
        }
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or(self.name())
        .trim_matches(&['/', ' '][..])
    }
}

impl From<&str> for Plugin {
    fn from(name: &str) -> Self {
        Plugin::Static {
            name: name.into(),
            prefix: Some(format!("_{}", name)),
        }
    }
}

impl From<(&str, &str)> for Plugin {
    fn from((name, prefix): (&str, &str)) -> Self {
        Plugin::Static {
            name: name.into(),
            prefix: Some(prefix.into()),
        }
    }
}
