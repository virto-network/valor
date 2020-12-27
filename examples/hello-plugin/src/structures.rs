//! Simple macro example for structures

use valor::*;

#[vlugin]
#[derive(Default)]
struct MyHandler;

impl RequestHandler for MyHandler {
    fn handle_request(&self, _: Request) -> HandlerResponse {
        unimplemented!()
    }
}
