mod registry;
mod vlugin_definition;

pub use vlugin_definition::{VluginDef, VluginType};

use crate::{async_trait, Answer, Context, Message, Vlugin};
use alloc::{borrow::ToOwned, boxed::Box, rc::Rc, string::String};
use core::{cell::RefCell, fmt, future::Future, pin::Pin};
use registry::PluginRegistry;

/// The runtime is a "Vlugin" itself that serves as the main entry point for
/// dispatching incoming messages to vlugins registered under a specific URL pattern.
///
/// ```
/// # use valor_core::*;
/// # use runtime::Runtime;
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
    pub async fn load_plugin(&self, mut plugin: VluginDef) -> Result<(), Error> {
        let factory = self
            .loader
            .load(&plugin)
            .await
            .map_err(|_| Error::LoadVlugin(plugin.name.clone()))?;
        let handler = factory(plugin.config.take())
            .await
            .map_err(|_| Error::InstantiateVlugin(plugin.name.clone()))?;
        self.register_plugin(plugin, handler)?;
        Ok(())
    }

    /// Expose the plugin registry as an endpoint on `_plugins` to add more plugins dynamically
    #[cfg(feature = "serde")]
    pub fn with_registry(self) -> Result<Self, Error> {
        self.register_plugin(
            ("registry", "_plugins"),
            PluginRegistry::get_handler(self.registry.clone(), self.loader.clone()),
        )?;
        Ok(self)
    }

    /// Include the built-in health plugin that returns _Ok_ on `_health`
    pub fn with_health(self) -> Result<Self, Error> {
        self.register_plugin("health", ())?;
        Ok(self)
    }

    /// Adds a plugin with its handler to the internal registry
    pub fn with_plugin<H>(self, plugin: impl Into<VluginDef>, handler: H) -> Result<Self, Error>
    where
        H: Vlugin + 'static,
    {
        self.register_plugin(plugin, handler)?;
        Ok(self)
    }

    fn register_plugin<H>(&self, plugin: impl Into<VluginDef>, handler: H) -> Result<(), Error>
    where
        H: Vlugin + 'static,
    {
        let handler: Box<dyn Vlugin> = Box::new(handler);
        let plugin = plugin.into();
        let name = plugin.name.clone();
        self.registry
            .borrow_mut()
            .register(plugin, handler)
            .map_err(|_| Error::RegisterVlugin(name))
    }
}

#[async_trait(?Send)]
impl<L> Vlugin for Runtime<L> {
    /// Handles an incoming request by answering form a plugin that matches the URL pattern
    ///
    /// It requires the request to specify a `x-request-id` header that is set back on
    /// the response as `x-correlation-id`(e.g. used by valor_web to match requests and responses)
    async fn on_msg(&self, msg: Message) -> Result<Answer, crate::Error> {
        use crate::http::{Error, StatusCode::*};
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
                res.append_header("x-correlation-id", req_id)
                    .expect("valid header");
                res.append_header("x-valor-plugin", plugin.name)
                    .expect("valid header");
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
pub enum Error {
    InstantiateVlugin(String),
    LoadVlugin(String),
    VluginNotSupported(VluginType),
    RegisterVlugin(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InstantiateVlugin(name) => write!(f, "Failed instantiating {}", name),
            Error::LoadVlugin(name) => write!(f, "Failed loading {}", name),
            Error::RegisterVlugin(name) => write!(f, "{} already registered", name),
            Error::VluginNotSupported(ty) => write!(f, "Loader doesn't support {:?}", ty),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// A Loader can fetch plugin handlers from various sources
/// such as the network or the file system
#[async_trait(?Send)]
pub trait Loader: 'static {
    /// Loads the given `plugin`
    async fn load(&self, plugin: &VluginDef) -> Result<VluginFactory, Error>;
}

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub type VluginFactory<'a> = Box<
    dyn Fn(Option<crate::VluginConfig>) -> BoxedFuture<'a, Result<Box<dyn Vlugin>, crate::Error>>,
>;

/// A dummy loader
#[async_trait(?Send)]
impl Loader for () {
    async fn load(&self, _plugin: &VluginDef) -> Result<VluginFactory, Error> {
        Ok(Box::new(|_cfg| {
            Box::pin(async { Ok(Box::new(()) as Box<dyn Vlugin>) })
        }))
    }
}
