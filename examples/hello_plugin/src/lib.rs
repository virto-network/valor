//! Simple macro example for functions
use valor::*;

#[vlugin]
async fn on_request(_req: Request) -> Response {
    "Hello Plugin!".into()
}
