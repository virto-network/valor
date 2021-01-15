//! Valor

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

    pub fn with_plugin<H>(&self, plugin: impl Into<Plugin>, handler: H)
    where
        H: RequestHandler + 'static,
    {
        self.registry
            .borrow_mut()
            .register(plugin.into(), Box::new(handler));
    }

    pub async fn load_plugin(&self, plugin: Plugin) -> core::result::Result<(), LoadError> {
        let handler = self.loader.load(&plugin).await?;
        self.with_plugin(plugin, handler);
        Ok(())
    }

    pub fn with_registry(self) -> Self {
        self.with_plugin(
            BuiltInPlugin::Registry,
            PluginRegistry::as_handler(self.registry.clone(), self.loader.clone()),
        );
        self
    }

    pub fn with_health(self) -> Self {
        self.with_plugin(BuiltInPlugin::Health, |_| async { res!() });
        self
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
            .registry
            .borrow()
            .match_plugin_handler(request.url().path())
            .ok_or_else(|| res!(StatusCode::NotFound, { x_correlation_id: &req_id }))?;

        Ok(res!(handler.handle_request(request).await, {
            x_correlation_id: req_id,
            x_vlugin: plugin.name()
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

/// Loader
#[async_trait(?Send)]
pub trait Loader: 'static {
    type Handler: RequestHandler;
    /// Loads the given `plugin`
    async fn load(&self, plugin: &Plugin) -> LoadResult<Self>;
}

pub type LoadResult<L> = core::result::Result<<L as Loader>::Handler, LoadError>;

pub enum LoadError {
    NotSupported,
    NotFound,
    BadFormat,
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
            Self::Native { name, prefix, .. } => prefix.as_ref().unwrap_or(name),
            Self::Web { name, prefix, .. } => prefix.as_ref().unwrap_or(name),
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
