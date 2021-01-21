pub use valor_plugin::vlugin;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web {
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    use crate::{Request, Response};
    use core::pin::Pin;
    use js_sys::{Array, Uint8Array};
    pub use wasm_bindgen;
    use wasm_bindgen::JsCast;
    pub use wasm_bindgen_futures;
    use wasm_bindgen_futures::JsFuture;
    pub use web_sys;
    use web_sys::{
        Headers, Request as JsRequest, RequestInit, Response as JsResponse, ResponseInit,
    };

    // NOTE we might be able to remove this some day
    // https://github.com/http-rs/http-types/issues/317
    pub async fn into_js_request(mut req: Request) -> (JsRequest, Pin<alloc::vec::Vec<u8>>) {
        let mut init = RequestInit::new();
        init.method(req.method().as_ref());

        let body = req.take_body().into_bytes().await.unwrap();
        let body = Pin::new(body);
        if body.len() > 0 {
            let uint_8_array = unsafe { Uint8Array::view(&body) };
            init.body(Some(&uint_8_array));
        }

        let headers = Headers::new().unwrap();
        for (name, value) in req.iter() {
            headers
                .set(name.as_str(), value.as_str())
                .expect("valid header name");
        }
        init.headers(&headers);

        (
            JsRequest::new_with_str_and_init(req.url().as_str(), &init).expect("valid url"),
            body,
        )
    }

    pub async fn into_request(req: JsRequest) -> Request {
        let method = req.method().parse().expect("valid method");
        let buffer = req.array_buffer().unwrap();
        let buffer = JsFuture::from(buffer).await.unwrap();
        let body = Uint8Array::new(&buffer).to_vec();

        let mut request = Request::new(method, req.url().as_str());
        request.set_body(body);

        for header in js_sys::try_iter(&req.headers()).unwrap().unwrap() {
            let header = header.unwrap();
            let header = header.unchecked_ref::<Array>();
            let name = header.get(0).as_string().unwrap();
            let value = header.get(1).as_string().unwrap();

            request.insert_header(name.as_str(), value.as_str());
        }

        request
    }

    pub async fn into_js_response(mut res: Response) -> JsResponse {
        let mut init = ResponseInit::new();
        init.status(res.status() as u16);
        let mut body = res.body_bytes().await.unwrap();

        let headers = Headers::new().unwrap();
        for (name, value) in res.iter() {
            headers
                .set(name.as_str(), value.as_str())
                .expect("valid header name");
        }
        init.headers(&headers);

        JsResponse::new_with_opt_u8_array_and_init(Some(body.as_mut()), &init).unwrap()
    }

    pub async fn into_response(res: JsResponse) -> Response {
        let body = JsFuture::from(res.array_buffer().unwrap()).await.unwrap();
        let body = Uint8Array::new(&body).to_vec();

        let mut response = Response::new(res.status());
        response.set_body(body);

        for header in js_sys::try_iter(&res.headers()).unwrap().unwrap() {
            let header = header.unwrap();
            let header = header.unchecked_ref::<Array>();
            let name = header.get(0).as_string().unwrap();
            let value = header.get(1).as_string().unwrap();

            response.insert_header(name.as_str(), value.as_str());
        }

        response
    }

    #[cfg(test)]
    pub mod tests {
        wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

        use super::*;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        use wasm_bindgen_test::*;

        const REQ_BODY: &str = "Hello mundo!";
        const REQ_URL: &str = "foo://hello-mundo";

        #[wasm_bindgen_test]
        async fn convert_into_js_request() {
            let (js_req, _) = into_js_request(test_request()).await;

            let body = JsFuture::from(js_req.text().unwrap()).await.unwrap();
            let headers = js_req.headers();

            assert_eq!(js_req.url(), REQ_URL);
            assert_eq!(js_req.method(), "POST");
            assert_eq!(body, JsValue::from(REQ_BODY));
            assert_eq!(headers.get("x-foo").unwrap(), Some("foo132".to_string()));
        }

        #[wasm_bindgen_test]
        async fn convert_into_request() {
            let mut req = into_request(test_js_request()).await;

            assert_eq!(req.url().as_str(), REQ_URL);
            assert_eq!(req.method().as_ref(), "POST");
            assert_eq!(req.body_string().await.unwrap(), REQ_BODY);
            let header = req.header("x-foo").unwrap().get(0).unwrap().as_str();
            assert_eq!(header, "foo132");
        }

        #[wasm_bindgen_test]
        async fn convert_into_response() {
            let mut init = ResponseInit::new();
            init.status(400);
            let headers = Headers::new().unwrap();
            headers.set("x-foo", "foo132").unwrap();
            init.headers(&headers);
            let js_res = JsResponse::new_with_opt_str_and_init(Some(REQ_BODY), &init).unwrap();
            let mut res = into_response(js_res).await;

            assert_eq!(res.status() as u16, 400);
            assert_eq!(res.body_string().await.unwrap(), REQ_BODY);
            assert_eq!(
                res.header("x-foo").unwrap().get(0).unwrap().as_str(),
                "foo132"
            );
        }

        #[wasm_bindgen_test]
        async fn convert_into_js_response() {
            let mut res = Response::new(400);
            res.set_body(REQ_BODY);
            res.insert_header("x-foo", "foo132");

            let res = into_js_response(res).await;

            assert_eq!(res.status(), 400);
            let body = JsFuture::from(res.text().unwrap()).await.unwrap();
            assert_eq!(body, REQ_BODY);
            let headers = res.headers();
            assert_eq!(headers.get("x-foo").unwrap(), Some("foo132".to_string()));
        }

        fn test_request() -> Request {
            let mut req = Request::new(crate::Method::Post, REQ_URL);
            req.set_body(REQ_BODY);
            req.insert_header("x-foo", "foo132");
            req
        }

        fn test_js_request() -> web_sys::Request {
            let mut init = RequestInit::new();
            init.method("POST");
            init.body(Some(&JsValue::from(REQ_BODY)));
            let headers = Headers::new().unwrap();
            headers.set("x-foo", "foo132").unwrap();
            init.headers(&headers);

            JsRequest::new_with_str_and_init(REQ_URL, &init).unwrap()
        }
    }
}
