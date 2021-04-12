//! Simple example usage of an HTTP handler plugin.
use valor::*;

#[vlugin]
pub async fn on_request(_req: http::Request) -> http::Response {
    "Hello Plugin!".into()
}
