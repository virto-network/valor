//! Simple macro example for structures

use valor::*;

#[vlugin]
#[derive(Default)]
struct MyHandler;

#[async_trait(?Send)]
impl RequestHandler for MyHandler {
    async fn handle_request(&self, _: Request) -> Response {
        unimplemented!()
    }
}

