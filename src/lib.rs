//! ## Valor
//!
//! A lightweight HTTP plugin system that runs in the server and the browser.
//!
//! - Use `valor_bin` to run your Rust and JS(soon!) plugins in the server.
//! - Use `valor_web` as a script imported from the main document or a worker
//! in your web application to have a local API powered by a service worker.

#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

extern crate alloc;
extern crate core;

#[cfg(feature = "proxy")]
mod proxy;
mod registry;
#[cfg(feature = "util")]
mod util;
mod vlugin;

use alloc::{borrow::ToOwned, boxed::Box, rc::Rc, string::String};
use core::future::Future;
use core::pin::Pin;
use core::{cell::RefCell, fmt};
use registry::PluginRegistry;

pub use async_trait::async_trait;
pub use http_types as http;
#[cfg(feature = "serde")]
pub use serde::{Deserialize, Serialize};
#[cfg(feature = "util")]
pub use util::*;
pub use vlugin::*;

/// The runtime is a "Vlugin" itself that serves as the main entry point for
/// dispatching incoming messages to vlugins registered under a specific URL pattern.
///
/// ```
/// # use valor_core::*;
/// # #[async_std::main] async fn main() { test().await.expect("Runtime handles messages") }
/// # async fn test() -> Result<(), Error> {
/// let runtime = Runtime::new(())
///     .with_plugin("foo", h(|req: http::Request, _| async move {
///         let res: http::Response = req.url().path().into();
///         Ok(res)
///     }))?;
///
/// let mut request = http::Request::new(http::Method::Get, "http://example.com/_foo/bar/baz");
/// request.insert_header("x-request-id", "123");
/// let mut res: http::Response = runtime.on_msg(request.into()).await?.into();
///
/// assert_eq!(res.status(), http::StatusCode::Ok);
/// assert_eq!(res.header("x-correlation-id").unwrap(), "123");
/// assert_eq!(res.header("x-valor-plugin").unwrap(), "foo");
/// assert_eq!(res.body_string().await.unwrap(), "/bar/baz");
/// # Ok(()) }
/// ```
pub struct Runtime<L> {
    cx: Context,
    registry: Rc<RefCell<PluginRegistry>>,
    loader: Rc<L>,
}

impl<L: Loader> Runtime<L> {
    /// Creates a new `Handler` instance
    pub fn new(loader: impl Into<Rc<L>>) -> Self {
        Runtime {
            cx: Context::default(),
            registry: Rc::new(RefCell::new(PluginRegistry::new())),
            loader: loader.into(),
        }
    }

    /// Uses the configured loader to load and register the provided plugin
    pub async fn load_plugin(&self, mut plugin: VluginInfo) -> Result<(), RuntimeError> {
        let factory = self
            .loader
            .load(&plugin)
            .await
            .map_err(|_| RuntimeError::LoadPlugin(plugin.name.clone()))?;
        let handler = factory(plugin.config.take())
            .await
            .map_err(|_| RuntimeError::InstantiateVlugin(plugin.name.clone()))?;
        self.register_plugin(plugin, handler)?;
        Ok(())
    }

    /// Expose the plugin registry as an endpoint on `_plugins` to add more plugins dynamically
    #[cfg(feature = "serde")]
    pub fn with_registry(self) -> Result<Self, RuntimeError> {
        self.register_plugin(
            ("registry", "_plugins"),
            PluginRegistry::get_handler(self.registry.clone(), self.loader.clone()),
        )?;
        Ok(self)
    }

    /// Include the built-in health plugin that returns _Ok_ on `_health`
    pub fn with_health(self) -> Result<Self, RuntimeError> {
        self.register_plugin("health", ())?;
        Ok(self)
    }

    /// Adds a plugin with its handler to the internal registry
    pub fn with_plugin<H>(
        self,
        plugin: impl Into<VluginInfo>,
        handler: H,
    ) -> Result<Self, RuntimeError>
    where
        H: Vlugin + 'static,
    {
        self.register_plugin(plugin, handler)?;
        Ok(self)
    }

    fn register_plugin<H>(
        &self,
        plugin: impl Into<VluginInfo>,
        handler: H,
    ) -> Result<(), RuntimeError>
    where
        H: Vlugin + 'static,
    {
        let handler: Box<dyn Vlugin> = Box::new(handler);
        let plugin = plugin.into();
        let name = plugin.name.clone();
        self.registry
            .borrow_mut()
            .register(plugin, handler)
            .map_err(|_| RuntimeError::RegisterPlugin(name))
    }
}

#[async_trait(?Send)]
impl<L> Vlugin for Runtime<L> {
    /// Handles an incoming request by answering form a plugin that matches the URL pattern
    ///
    /// It requires the request to specify a `x-request-id` header that is set back on
    /// the response as `x-correlation-id`(e.g. used by valor_web to match requests and responses)
    async fn on_msg(&self, msg: Message) -> Result<Answer, Error> {
        use http::{Error, StatusCode::*};
        let mut request = match msg {
            Message::Http(req) => req,
            _ => return Err(crate::Error::NotSupported),
        };

        let req_id = request
            .header("x-request-id")
            .ok_or_else(|| Error::from_str(BadRequest, "Missing request ID"))?
            .as_str()
            .to_owned();

        let (plugin, handler) = self
            .registry
            .borrow()
            .match_vlugin(request.url().path())
            .ok_or_else(|| Error::from_str(NotFound, "No plugin matched"))?;

        let without_prefix = request
            .url()
            .path()
            .trim_start_matches('/')
            .strip_prefix(plugin.prefix_or_name())
            .expect("prefix")
            .to_owned();
        request.url_mut().set_path(&without_prefix);

        handler.on_msg(request.into()).await.map(|out| match out {
            Answer::Http(mut res) => {
                res.append_header("x-correlation-id", req_id);
                res.append_header("x-valor-plugin", plugin.name);
                res.into()
            }
            _ => Answer::Pong,
        })
    }

    fn context(&self) -> &Context {
        &self.cx
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.cx
    }
}

impl<L> Clone for Runtime<L> {
    fn clone(&self) -> Self {
        Runtime {
            cx: Context::default(),
            registry: self.registry.clone(),
            loader: self.loader.clone(),
        }
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    InstantiateVlugin(String),
    LoadPlugin(String),
    RegisterPlugin(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::InstantiateVlugin(name) => write!(f, "Failed instantiating {}", name),
            RuntimeError::LoadPlugin(name) => write!(f, "Failed loading {}", name),
            RuntimeError::RegisterPlugin(name) => write!(f, "{} already registered", name),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RuntimeError {}

/// A Loader can fetch plugin handlers from various sources
/// such as the network or the file system
#[async_trait(?Send)]
pub trait Loader: 'static {
    /// Loads the given `plugin`
    async fn load(&self, plugin: &VluginInfo) -> Result<VluginFactory, LoadError>;
}

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
pub type VluginFactory<'a> =
    Box<dyn Fn(Option<VluginConfig>) -> BoxedFuture<'a, Result<Box<dyn Vlugin>, Error>>>;

/// Errors loading a plugin
#[derive(Debug)]
pub enum LoadError {
    NotSupported,
    NotFound,
}

impl From<LoadError> for Error {
    fn from(e: LoadError) -> Self {
        use http::{Error, StatusCode::*};
        match e {
            LoadError::NotSupported => {
                Error::from_str(BadRequest, "Plugin type not supported by loader").into()
            }
            LoadError::NotFound => Error::from_str(NotFound, "Couldn't find plugin").into(),
        }
    }
}

/// A dummy loader
#[async_trait(?Send)]
impl Loader for () {
    async fn load(&self, _plugin: &VluginInfo) -> Result<VluginFactory, LoadError> {
        Ok(Box::new(|_cfg| {
            Box::pin(async { Ok(Box::new(()) as Box<dyn Vlugin>) })
        }))
    }
}

/// Plugin info
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct VluginInfo {
    /// Name of the plugin
    pub name: String,
    /// Url prefix where the plugin is mounted, defaults to the name
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub prefix: Option<String>,
    /// What kind of plugin
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub r#type: VluginType,
    /// Environment configuration to pass down to the plugin instance
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub config: Option<VluginConfig>, // NOTE this makes the core dependent on serde
}

pub type VluginConfig = serde_json::Value;

impl VluginInfo {
    fn prefix_or_name(&self) -> &str {
        self.prefix
            .as_deref()
            .unwrap_or(&self.name)
            .trim_matches(&['/', ' '][..])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "type", rename_all = "snake_case")
)]
pub enum VluginType {
    /// Plugin that comes with the runtime
    Static,
    /// Natively compiled Rust plugin
    Native {
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        path: Option<String>,
    },
    /// Web script or WASM
    Web { url: String },
}

impl From<&str> for VluginInfo {
    fn from(name: &str) -> Self {
        VluginInfo {
            name: name.into(),
            prefix: Some("_".to_owned() + name),
            r#type: VluginType::Static,
            config: None,
        }
    }
}

impl From<(&str, &str)> for VluginInfo {
    fn from((name, prefix): (&str, &str)) -> Self {
        VluginInfo {
            name: name.into(),
            prefix: Some(prefix.into()),
            r#type: VluginType::Static,
            config: None,
        }
    }
}
