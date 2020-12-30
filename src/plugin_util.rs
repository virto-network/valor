pub use valor_vlugin::vlugin;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web {
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    pub use wasm_bindgen;
    pub use wasm_bindgen_futures;
    pub use web_sys;

    pub struct JsRequest(pub web_sys::Request);
    pub struct JsResponse(pub Response);

    use http_types::{Method, Request, Response};

    impl From<JsRequest> for Request {
        fn from(_req: JsRequest) -> Self {
            // TODO
            Request::new(Method::Get, "")
        }
    }

    impl From<JsResponse> for web_sys::Response {
        fn from(_req: JsResponse) -> Self {
            // TODO
            web_sys::Response::new().unwrap()
        }
    }
}
