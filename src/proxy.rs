use crate::{async_trait, http, Context, Error, Handler, Message, Output};
use alloc::boxed::Box;
use alloc::string::String;
use core::convert::TryFrom;
#[cfg(target_arch = "wasm32")]
use http_client::{h1::wasm::WasmClient as Client, HttpClient};
#[cfg(not(target_arch = "wasm32"))]
use http_client::{h1::H1Client as Client, HttpClient};

struct Proxy {
    client: Client,
    server: http::Url,
}

impl TryFrom<String> for Proxy {
    type Error = http::Error;

    fn try_from(url: String) -> Result<Self, Self::Error> {
        Ok(Proxy {
            client: Client::new(),
            server: url.parse()?,
        })
    }
}

#[async_trait(?Send)]
impl Handler for Proxy {
    fn context(&self) -> &Context {
        unreachable!()
    }

    async fn on_msg(&self, msg: Message) -> Result<Output, Error> {
        let mut req = match msg {
            Message::Http(req) => req,
            Message::Ping => return Err(Error::NotSupported),
        };

        let url = req.url();
        let mut upstream_url = self
            .server
            .join(url.path())
            .map_err(|_| http::Error::from_str(http::StatusCode::InternalServerError, ""))?;
        upstream_url.set_query(url.query());
        upstream_url.set_fragment(url.fragment());

        let mut proxied_req = http::Request::new(req.method(), upstream_url);
        // copy headers
        proxied_req.as_mut().clone_from(req.as_ref());

        proxied_req.set_body(req.take_body());

        Ok(self.client.send(proxied_req).await?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::test;
    use core::convert::TryInto;
    use http_types::Method;

    #[test]
    async fn foward_request() -> Result<(), Error> {
        let body = r#"{"hello": "world"}"#;
        let mock = mockito::mock("POST", "/foo?f=fff")
            .with_status(202)
            .match_header("content-type", "application/json")
            .match_body(body)
            .create();

        let p: Proxy = mockito::server_url().try_into()?;

        let mut req = http::Request::new(Method::Post, "foo:/foo?f=fff");
        req.append_header(http_types::headers::CONTENT_TYPE, http_types::mime::JSON);
        req.set_body(body);
        let res: http::Response = p.on_msg(req.into()).await?.into();

        assert_eq!(res.status(), http::StatusCode::Accepted);
        mock.assert();
        Ok(())
    }
}
