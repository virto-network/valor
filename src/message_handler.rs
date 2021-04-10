use crate::{async_trait, http};
use alloc::boxed::Box;
use core::{
    any::{Any, TypeId},
    fmt,
    marker::PhantomData,
};
use hashbrown::HashMap;

/// Context allows plugins to pass state to the message handler
/// and eventually to easily communicate with other plugins.
#[derive(Debug, Default)]
pub struct Context {
    data: HashMap<TypeId, Box<dyn Any>>,
}

impl Context {
    pub fn set(&mut self, data: impl Any) {
        self.data.insert(data.type_id(), Box::new(data));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|d| d.downcast_ref::<T>())
            .flatten()
    }
}

/// The Vlugin trait defines plugins that can handle any supported message
/// format. It also allows the plugin to initialize an internal state with the
/// help of the `Context` type.
///
/// ```
/// # #[async_std::main] async fn main() { test().await.unwrap() }
/// # use valor_core::*;
/// #[derive(Default)]
/// struct SomeVlugin(Context);
///
/// #[async_trait(?Send)]
/// impl Vlugin for SomeVlugin {
///     async fn on_create(&mut self) -> Result<(), Error> {
///         self.0.set("some data");
///         Ok(())
///     }
///
///     async fn on_msg(&self, msg: Message) -> Result<Answer, Error> {
///         let _data = self.context().get::<&str>();
///         Ok(().into())
///     }
///
///     fn context(&self) -> &Context {
///         &self.0
///     }
/// }
///
/// # async fn test() -> Result<(), Error> {
/// let v = SomeVlugin::create().await?;
/// match v.on_msg(().into()).await? {
///     Answer::Pong => {},
///     _ => panic!("Wrong answer!"),
/// };
/// # Ok(()) }
/// ```
///
#[async_trait(?Send)]
pub trait Vlugin {
    async fn create() -> Result<Self, Error>
    where
        Self: Sized + Default,
    {
        let mut h = Self::default();
        h.on_create().await?;
        Ok(h)
    }

    fn context(&self) -> &Context;

    /// Hook to do some plugin initialization like setting some shared state
    async fn on_create(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn on_msg(&self, msg: Message) -> Result<Answer, Error>;
}

#[async_trait(?Send)]
impl<T> Vlugin for Box<T>
where
    T: Vlugin + ?Sized,
{
    async fn on_create(&mut self) -> Result<(), Error> {
        (&mut **self).on_create().await
    }

    async fn on_msg(&self, msg: Message) -> Result<Answer, Error> {
        (&**self).on_msg(msg).await
    }

    fn context(&self) -> &Context {
        (&**self).context()
    }
}

/// Shorthand for handlers created from a closure
pub fn h<M, O, F, Fut>(handler_fn: F) -> FnHandler<M, O, F, Fut>
where
    F: Fn(M, &Context) -> Fut,
    M: From<Message>,
    O: Into<Answer>,
    Fut: core::future::Future<Output = Result<O, Error>>,
{
    FnHandler(handler_fn, Context::default(), PhantomData)
}

pub struct FnHandler<M, O, F, Fut>(F, Context, PhantomData<(M, O, Fut)>)
where
    F: Fn(M, &Context) -> Fut,
    M: From<Message>,
    O: Into<Answer>,
    Fut: core::future::Future<Output = Result<O, Error>>;

#[async_trait(?Send)]
impl<M, O, F, Fut> Vlugin for FnHandler<M, O, F, Fut>
where
    F: Fn(M, &Context) -> Fut,
    M: From<Message>,
    O: Into<Answer>,
    Fut: core::future::Future<Output = Result<O, Error>>,
{
    async fn on_msg(&self, msg: Message) -> Result<Answer, Error> {
        Ok(self.0(M::from(msg), self.context()).await?.into())
    }

    fn context(&self) -> &Context {
        &self.1
    }
}

// Dummy handler mostly for test purposes
#[async_trait(?Send)]
impl Vlugin for () {
    async fn on_msg(&self, _msg: Message) -> Result<Answer, Error> {
        Ok(Answer::Pong)
    }

    fn context(&self) -> &Context {
        unreachable!()
    }
}

/// Type of message supported by a handler
#[derive(Debug)]
pub enum Message {
    Http(http::Request),
    Ping,
}

impl From<http::Request> for Message {
    fn from(req: http::Request) -> Self {
        Message::Http(req)
    }
}

impl From<()> for Message {
    fn from(_: ()) -> Self {
        Message::Ping
    }
}

impl From<Message> for http::Request {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Http(req) => req,
            _ => unimplemented!(),
        }
    }
}

/// Type of valid outputs that a handler can return
#[derive(Debug)]
pub enum Answer {
    Http(http::Response),
    Pong,
}

impl From<Answer> for http::Response {
    fn from(out: Answer) -> Self {
        match out {
            Answer::Http(res) => res,
            Answer::Pong => http::StatusCode::Ok.into(),
        }
    }
}

impl From<http::Body> for Answer {
    fn from(body: http::Body) -> Self {
        let res: http::Response = body.into();
        res.into()
    }
}

impl From<http::Response> for Answer {
    fn from(res: http::Response) -> Self {
        Answer::Http(res)
    }
}

impl From<()> for Answer {
    fn from(_: ()) -> Self {
        Answer::Pong
    }
}

#[derive(Debug)]
pub enum Error {
    Http(http::Error),
    NotSupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Http(err) => write!(f, "{}", err),
            Error::NotSupported => write!(f, "Not supported"),
        }
    }
}

impl From<Error> for http::Error {
    fn from(err: Error) -> Self {
        match err {
            Error::Http(err) => err,
            _ => http::Error::from_str(http::StatusCode::InternalServerError, ""),
        }
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Self {
        Error::Http(err)
    }
}
