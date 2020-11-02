use fast_async_mutex::mutex::Mutex;
pub use http_types::{Error, Method, Request, Response, Result, StatusCode, Url};
use log::{debug, info};
use path_tree::PathTree;
use std::sync::Arc;

pub struct Handler {
    routes: Mutex<PathTree<&'static str>>,
}

impl Handler {
    pub fn new() -> Arc<Self> {
        let mut r = PathTree::new();
        r.insert("/_plugins", "plugins");
        debug!("Handler initialized");
        Arc::new(Handler {
            routes: Mutex::new(r),
        })
    }

    pub async fn handle_request(self: Arc<Self>, request: impl Into<Request>) -> Result<Response> {
        let request = request.into();
        let req_id = request.header("x-request-id").ok_or(Error::from_str(
            StatusCode::BadRequest,
            "missing request ID",
        ))?;
        let routes = self.routes.lock().await;
        debug!("Handling request {}", req_id);
        match routes.find(request.url().path()) {
            Some((plugin, _)) => info!("Matched plugin '{}'", plugin),
            None => info!("No plugin matched"),
        }
        let mut response = Response::new(200);
        response.insert_header("x-correlation-id", req_id);
        Ok(response)
    }
}
