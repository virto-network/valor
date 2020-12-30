//! Simple macro example for functions
use valor::*;

#[vlugin]
async fn hello_plugin(_req: Request) -> Response {
    "Hello Plugin!".into()
}
