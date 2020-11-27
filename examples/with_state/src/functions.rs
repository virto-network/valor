//! Simple macro example for functions

use valor::*;

#[vlugin]
fn hello_plugin(_req: Request) -> Response {
    "Hello Plugin!".into()
}

