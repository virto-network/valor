//! ## Valor
//!
//! A lightweight HTTP plugin system that runs in the server and the browser.
//!
//! - Use `valor_bin` to run your Rust and JS(soon!) plugins in the server.
//! - Use `valor_web` as a script imported from the main document or a worker
//! in your web application to have a local API powered by a service worker.

#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

#[cfg(feature = "proxy")]
mod proxy;
mod registry;
#[cfg(feature = "util")]
mod util;

use alloc::{borrow::ToOwned, boxed::Box, rc::Rc, string::String};
use core::marker::PhantomData;
use core::{cell::RefCell, fmt};
use registry::PluginRegistry;
#[cfg(feature = "_serde")]
use serde::{Deserialize, Serialize};

pub use async_trait::async_trait;
pub use http_types as http;
#[cfg(feature = "util")]
pub use util::*;

/// The main entry point for dispatching incoming requests
/// to plugins registered under a specific URL prefix.
///
/// ```
/// # use valor_core::*;
/// # #[async_std::main] async fn main() { test().await.expect("Runtime handles messages") }
/// # async fn test() -> Result<(), Error> {
/// let handler = Runtime::new(())
///     .with_plugin("foo", h(|req: http::Request| async move {
///         let res: http::Response = req.url().path().into();
///         Ok(res)
///     }));
///
/// let mut request = http::Request::new(http::Method::Get, "http://example.com/_foo/bar/baz");
/// request.insert_header("x-request-id", "123");
/// let mut res: http::Response = handler.on_msg(request.into()).await?.into();
///
/// assert_eq!(res.status(), http::StatusCode::Ok);
/// assert_eq!(res.header("x-correlation-id").unwrap(), "123");
/// assert_eq!(res.header("x-valor-plugin").unwrap(), "foo");
/// assert_eq!(res.body_string().await.unwrap(), "/bar/baz");
/// # Ok(()) }
/// ```
pub struct Runtime<L> {
    registry: Rc<RefCell<PluginRegistry>>,
    loader: Rc<L>,
}

impl<L: Loader> Runtime<L> {
    /// Creates a new `Handler` instance
    pub fn new(loader: impl Into<Rc<L>>) -> Self {
        Runtime {
            registry: Rc::new(RefCell::new(PluginRegistry::new())),
            loader: loader.into(),
        }
    }

    /// Uses the configured loader to load and register the provided plugin
    pub async fn load_plugin(&self, plugin: Plugin) -> Result<(), LoadError>
where {
        let factory = self.loader.load(&plugin).await?;
        let handler = factory();
        self.register_plugin(plugin, handler);
        Ok(())
    }

    /// Expose the plugin registry as an endpoint on `_plugins` to add more plugins dynamically
    #[cfg(feature = "_serde")]
    pub fn with_registry(self) -> Self {
        self.register_plugin(
            Plugin::Static {
                name: "registry".into(),
                prefix: Some("_plugins".into()),
            },
            PluginRegistry::get_handler(self.registry.clone(), self.loader.clone()),
        );
        self
    }

    /// Include the built-in health plugin that returns _Ok_ on `_health`
    pub fn with_health(self) -> Self {
        self.register_plugin("health", ());
        self
    }

    /// Adds a plugin with its handler to the internal registry
    pub fn with_plugin<H>(self, plugin: impl Into<Plugin>, handler: H) -> Self
    where
        H: Handler + 'static,
    {
        self.register_plugin(plugin, handler);
        self
    }

    fn register_plugin<H>(&self, plugin: impl Into<Plugin>, handler: H)
    where
        H: Handler + 'static,
    {
        let handler: Box<dyn Handler> = Box::new(handler);
        self.registry.borrow_mut().register(plugin.into(), handler);
    }
}

#[async_trait(?Send)]
impl<L> Handler for Runtime<L> {
    /// Handles an incoming request by answering form a plugin that matches the URL pattern
    ///
    /// It requires the request to specify a `x-request-id` header that is set back on
    /// the response as `x-correlation-id`(e.g. used by valor_web to match requests and responses)
    async fn on_msg(&self, msg: Message) -> Result<Output, Error> {
        use http::{Error, StatusCode::*};
        let Message::Http(mut request) = msg;
        let req_id = request
            .header("x-request-id")
            .ok_or_else(|| Error::from_str(BadRequest, "Missing request ID"))?
            .as_str()
            .to_owned();

        let (plugin, handler) = self
            .registry
            .borrow()
            .match_plugin_handler(request.url().path())
            .ok_or_else(|| Error::from_str(NotFound, "No plugin matched"))?;

        let without_prefix = request
            .url()
            .path()
            .trim_start_matches('/')
            .strip_prefix(plugin.prefix())
            .expect("prefix")
            .to_owned();
        request.url_mut().set_path(&without_prefix);

        handler.on_msg(request.into()).await.map(|out| match out {
            Output::Http(mut res) => {
                res.append_header("x-correlation-id", req_id);
                res.append_header("x-valor-plugin", plugin.name());
                res.into()
            }
            _ => Output::None,
        })
    }
}

impl<L> Clone for Runtime<L> {
    fn clone(&self) -> Self {
        Runtime {
            registry: self.registry.clone(),
            loader: self.loader.clone(),
        }
    }
}

/// Type of message supported by a handler
pub enum Message {
    Http(http::Request),
}

impl From<http::Request> for Message {
    fn from(req: http::Request) -> Self {
        Message::Http(req)
    }
}

impl From<Message> for http::Request {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Http(req) => req,
        }
    }
}

/// Type of valid outputs that a handler can return
pub enum Output {
    Http(http::Response),
    None,
}

impl From<Output> for http::Response {
    fn from(out: Output) -> Self {
        match out {
            Output::Http(res) => res,
            _ => unreachable!(),
        }
    }
}

impl From<http::Body> for Output {
    fn from(body: http::Body) -> Self {
        let res: http::Response = body.into();
        res.into()
    }
}

impl From<http::Response> for Output {
    fn from(res: http::Response) -> Self {
        Output::Http(res)
    }
}

impl From<()> for Output {
    fn from(_: ()) -> Self {
        Output::None
    }
}

#[derive(Debug)]
pub enum Error {
    Http(http::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Http(err) => write!(f, "{}", err),
        }
    }
}

impl From<Error> for http::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Http(err) => err,
        }
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Self {
        Error::Http(err)
    }
}

/// Something that can handle messages
#[async_trait(?Send)]
pub trait Handler {
    async fn on_msg(&self, msg: Message) -> Result<Output, Error>;
}

#[async_trait(?Send)]
impl<T> Handler for Box<T>
where
    T: Handler + ?Sized,
{
    async fn on_msg(&self, msg: Message) -> Result<Output, Error> {
        (&**self).on_msg(msg).await
    }
}

/// Shorthand for handlers created from a function closure
pub fn h<M, O, F, Fut>(handler_fn: F) -> FnHandler<M, O, F, Fut>
where
    F: Fn(M) -> Fut,
    M: From<Message>,
    O: Into<Output>,
    Fut: core::future::Future<Output = Result<O, Error>>,
{
    FnHandler(handler_fn, PhantomData)
}

pub struct FnHandler<M, O, F, Fut>(F, PhantomData<(M, O, Fut)>)
where
    F: Fn(M) -> Fut,
    M: From<Message>,
    O: Into<Output>,
    Fut: core::future::Future<Output = Result<O, Error>>;

#[async_trait(?Send)]
impl<M, O, F, Fut> Handler for FnHandler<M, O, F, Fut>
where
    F: Fn(M) -> Fut,
    M: From<Message>,
    O: Into<Output>,
    Fut: core::future::Future<Output = Result<O, Error>>,
{
    async fn on_msg(&self, msg: Message) -> Result<Output, Error> {
        Ok(self.0(M::from(msg)).await?.into())
    }
}

// Dummy handler mostly for test purposes
#[async_trait(?Send)]
impl Handler for () {
    async fn on_msg(&self, _msg: Message) -> Result<Output, Error> {
        Ok(Output::None)
    }
}

/// A Loader can fetch plugin handlers from various sources
/// such as the network or the file system
#[async_trait(?Send)]
pub trait Loader: 'static {
    /// Loads the given `plugin`
    async fn load(&self, plugin: &Plugin) -> Result<VluginFactory, LoadError>;
}

pub type VluginFactory<'a> = Box<dyn Fn() -> Box<dyn Handler> + 'a>;

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
    async fn load(&self, _plugin: &Plugin) -> Result<VluginFactory, LoadError> {
        Ok(Box::new(|| Box::new(())))
    }
}

/// Plugin information
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "_serde",
    derive(Serialize, Deserialize),
    serde(tag = "type", rename_all = "snake_case")
)]
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
        #[cfg_attr(feature = "_serde", serde(skip_serializing_if = "Option::is_none"))]
        path: Option<String>,
        /// Url prefix where the plugin is mounted, defaults to the name
        #[cfg_attr(feature = "_serde", serde(skip_serializing_if = "Option::is_none"))]
        prefix: Option<String>,
    },
    /// Web script or WASM
    Web {
        /// Name
        name: String,
        /// Url of the JS script
        url: String,
        /// Url prefix where the plugin is mounted, defaults to the name
        #[cfg_attr(feature = "_serde", serde(skip_serializing_if = "Option::is_none"))]
        prefix: Option<String>,
    },
}

impl Plugin {
    #[inline]
    fn name(&self) -> &str {
        &match self {
            Self::Static { name, .. } => name,
            Self::Native { name, .. } => name,
            Self::Web { name, .. } => name,
        }
    }

    #[inline]
    fn prefix(&self) -> &str {
        match self {
            Self::Static { prefix, .. } => prefix,
            Self::Native { prefix, .. } => prefix,
            Self::Web { prefix, .. } => prefix,
        }
        .as_ref()
        .map(|p| p.as_str())
        .unwrap_or_else(|| self.name())
        .trim_matches(&['/', ' '][..])
    }
}

impl From<&str> for Plugin {
    fn from(name: &str) -> Self {
        Plugin::Static {
            name: name.into(),
            prefix: Some("_".to_owned() + name),
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
