//! Simple macro example for functions
use valor::*;

#[vlugin]
async fn on_request(_req: http::Request) -> http::Response {
    "Hello Plugin!".into()
}
