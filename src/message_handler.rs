use crate::{async_trait, http};
use alloc::{boxed::Box, rc::Rc};
use core::{
    any::{Any, TypeId},
    fmt,
    marker::PhantomData,
};
use hashbrown::HashMap;

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

/// Context allows plugins to pass some state to the message handler
/// and eventually to easily communicate with other plugins.
pub struct Context {
    data: HashMap<TypeId, Rc<dyn Any>>,
}

impl Context {
    pub fn set(&mut self, data: impl Any) {
        self.data.insert(data.type_id(), Rc::new(data));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|d| d.downcast_ref::<T>())
            .flatten()
    }
}

/// Something that can handle messages
#[async_trait(?Send)]
pub trait Handler {
    /// Hook to do some plugin initialization like setting some shared state
    async fn on_create(&self, _cx: &mut Context) -> Result<(), ()> {
        Ok(())
    }

    async fn on_msg(&self, msg: Message) -> Result<Output, Error>;
}

#[async_trait(?Send)]
impl<T> Handler for Box<T>
where
    T: Handler + ?Sized,
{
    async fn on_create(&self, cx: &mut Context) -> Result<(), ()> {
        (&**self).on_create(cx).await
    }

    async fn on_msg(&self, msg: Message) -> Result<Output, Error> {
        (&**self).on_msg(msg).await
    }
}

/// Shorthand for handlers created from a closure
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
