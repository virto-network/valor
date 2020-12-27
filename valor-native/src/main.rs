//! Valor vlugin

use kv_log_macro::{error, info};
use loader::DynLoader;
use std::sync::Arc;
use std::time::Instant;
use tide::Request;
use uuid::Uuid;

mod loader;

#[async_std::main]
async fn main() {
    femme::with_level(femme::LevelFilter::Debug);

    let loader = Arc::new(DynLoader);
    let mut app = tide::new();
    app.at("*").all(handler(valor::Handler::new(loader)));

    if let Err(err) = app.listen(("localhost", 8080)).await {
        error!("{}", err);
    }
}

const REQ_ID_HEADER: &str = "x-request-id";

fn handler(handler: valor::Handler) -> impl tide::Endpoint<()> {
    move |mut req: Request<()>| {
        let plugins = handler.clone();
        async move {
            let plugins = plugins.clone();
            let instant = Instant::now();
            if req.header(REQ_ID_HEADER).is_none() {
                let id = Uuid::new_v4().to_string();
                req.insert_header(REQ_ID_HEADER, id);
            }

            let method = req.method();
            let path = req.url().path().to_string();

            let res = plugins.handle_request(req).await.unwrap_or_else(|err| err);

            let id = match res.header("x-correlation-id") {
                Some(hv) => hv.as_str(),
                None => "No header: x-correlation-id",
            };
            let plugin = match res.header("x-valor-plugin") {
                Some(hv) => hv.as_str(),
                None => "No header: x-valor-plugin",
            };
            let status: u16 = res.status().into();

            info!("[{}] {} {} {}", plugin, status, method, path, {
                id: id, status: status, nanos: instant.elapsed().as_nanos() as u64
            });

            Ok(res)
        }
    }
}
