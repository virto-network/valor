use http_types::{Body, Method, Request, Response, StatusCode};
use http_types::convert::{Deserialize, Serialize};

use valor::Handler;
use valor::proxy::ReverseProxyHandler;

#[async_std::test]
async fn test_proxy_handler() -> Result<(), Box<dyn std::error::Error>> {
    let handler = Handler::new(())
        .with_plugin("v1/foo", |req: Request| async move {
            req.url().path().into()
        })
        .with_plugin("v2/bar", |_req: Request| async move {
            let mut res = Response::new(StatusCode::Ok);
            res.set_body("baz");
            res
        })
        .with_plugin("qux", |mut req: Request| async move {
            let body: Person = req.body_json().await.unwrap();
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(Body::from_json(&body).unwrap());
            res
        });
    let reverse_proxy_handler = ReverseProxyHandler::new(handler.clone());
    let handler = handler
        .with_plugin("api", reverse_proxy_handler);

    let mut request = Request::new(Method::Get, "http://example.com/_api/_v1/foo/some/path");
    request.insert_header("x-request-id", "987");
    let mut res = handler.handle_request(request).await.unwrap();

    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(res.header("x-correlation-id").unwrap(), "987");
    assert_eq!(res.header("x-valor-plugin").unwrap(), "api");
    assert_eq!(res.header("x-valor-proxy").unwrap(), "v1/foo");
    assert_eq!(res.body_string().await.unwrap(), "/_v1/foo/some/path");

    request = Request::new(Method::Get, "http://example.com/_api/_v2/bar/another/path");
    request.insert_header("x-request-id", "123");
    res = handler.handle_request(request).await.unwrap();

    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(res.header("x-correlation-id").unwrap(), "123");
    assert_eq!(res.header("x-valor-plugin").unwrap(), "api");
    assert_eq!(res.header("x-valor-proxy").unwrap(), "v2/bar");
    assert_eq!(res.body_string().await.unwrap(), "baz");

    request = Request::new(Method::Get, "http://example.com/_api/_unknown/plugin");
    request.insert_header("x-request-id", "456");
    res = handler.handle_request(request).await.unwrap();

    assert_eq!(res.status(), StatusCode::NotFound);
    assert_eq!(res.header("x-correlation-id").unwrap(), "456");
    assert_eq!(res.header("x-valor-plugin").unwrap(), "api");
    assert!(res.header("x-valor-proxy").is_none());
    assert_eq!(res.body_string().await.unwrap(), "Plugin not supported: /_unknown/plugin");

    // Test Post
    #[derive(Debug, Serialize, Deserialize)]
    struct Person { name: String }
    let man = Person { name: String::from("John") };
    request = Request::new(Method::Post, "http://example.com/_api/_qux");
    request.set_body(Body::from_json(&man)?);
    request.insert_header("x-request-id", "222");
    res = handler.handle_request(request).await.unwrap();
    let person: Person = res.body_json().await?;
    assert_eq!(res.status(), StatusCode::Ok);
    assert_eq!(res.header("x-correlation-id").unwrap(), "222");
    assert_eq!(res.header("x-valor-plugin").unwrap(), "api");
    assert_eq!(res.header("x-valor-proxy").unwrap(), "qux");
    assert_eq!(person.name, "John");

    Ok(())
}
