use alloc::prelude::v1::Box;
use alloc::string::String;

use async_trait::async_trait;
use http_types::{Request, Response, StatusCode};

use crate::{Handler, Loader, RequestHandler};

/// ReverseProxyHandler designed to forward requests to other known
/// plugins or return a 404 Not Found error.
///
/// ```
/// # use valor_core::*;
/// # use valor_core::proxy::ReverseProxyHandler;
/// # #[async_std::main] async fn main() { test().await }
/// # async fn test() {
///
/// let handler = Handler::new(())
///     .with_plugin("foo", |req: Request| async move { req.url().path().into() });
///
/// let reverse_proxy_handler = ReverseProxyHandler::new(handler.clone());
/// let handler = handler
///     .with_plugin("api", reverse_proxy_handler);
///
/// let mut request = Request::new(Method::Get, "http://example.com/_api/_foo/test/path");
/// request.insert_header("x-request-id", "123");
/// let mut res = handler.handle_request(request).await.unwrap();
///
/// assert_eq!(res.status(), StatusCode::Ok);
/// assert_eq!(res.header("x-correlation-id").unwrap(), "123");
/// assert_eq!(res.header("x-valor-plugin").unwrap(), "api");
/// assert_eq!(res.header("x-valor-proxy").unwrap(), "foo");
/// assert_eq!(res.body_string().await.unwrap(), "/_foo/test/path");
///
/// request = Request::new(Method::Get, "http://example.com/_api/_unknown/plugin/path");
/// request.insert_header("x-request-id", "987");
/// res = handler.handle_request(request).await.unwrap();
///
/// assert_eq!(res.status(), StatusCode::NotFound);
/// assert_eq!(res.header("x-correlation-id").unwrap(), "987");
/// assert_eq!(res.header("x-valor-plugin").unwrap(), "api");
/// assert!(res.header("x-valor-proxy").is_none());
/// assert_eq!(res.body_string().await.unwrap(), "Plugin not supported: /_unknown/plugin/path");
///
/// # }
/// ```
pub struct ReverseProxyHandler<L> {
    handler: Handler<L>,
}

impl<L: Loader + 'static> ReverseProxyHandler<L> {
    pub fn new(handler: Handler<L>) -> Self {
        ReverseProxyHandler {
            handler,
        }
    }
}

#[cfg(feature = "_serde")]
#[async_trait(?Send)]
impl<L: Loader + 'static> RequestHandler for ReverseProxyHandler<L> {
    async fn handle_request(&self, request: Request) -> Response {
        let path: &str = request.url().path();
        let maybe_plugin = self.handler.registry.borrow_mut().match_plugin_handler(path);
        if let Some((plugin, handler)) = maybe_plugin {
            let mut r = handler.handle_request(request).await;
            r.insert_header("x-valor-proxy", plugin.name());
            return r;
        }
        let mut body = String::from("Plugin not supported: ");
        body.push_str(path);
        let mut res = Response::new(StatusCode::NotFound);
        res.set_body(&body[..]);
        res
    }
}