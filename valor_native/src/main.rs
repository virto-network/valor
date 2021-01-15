//! Valor vlupin

use async_std::task;
use async_std::{
    net::{TcpListener, TcpStream},
    stream::StreamExt,
};
use kv_log_macro::{error, info};
use loader::DynLoader;
use std::time::Instant;
use uuid::Uuid;

mod loader;

type Handler = valor::Handler<DynLoader>;

#[async_std::main]
async fn main() -> http_types::Result<()> {
    femme::with_level(femme::LevelFilter::Debug);

    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    info!("listening on {}", addr);

    let handler = Handler::new(DynLoader).with_registry().with_health();

    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let handler = handler.clone();
        task::spawn_local(async move {
            if let Err(err) = accept(stream, handler).await {
                error!("{}", err);
            }
        });
    }
    Ok(())
}

const REQ_ID_HEADER: &str = "x-request-id";

async fn accept(stream: TcpStream, handler: Handler) -> http_types::Result<()> {
    async_h1::accept(stream.clone(), |mut req| async {
        //let handler = handler.clone();
        let instant = Instant::now();
        if req.header(REQ_ID_HEADER).is_none() {
            let id = Uuid::new_v4().to_string();
            req.insert_header(REQ_ID_HEADER, id);
        }

        let method = req.method();
        let path = req.url().path().to_string();

        let res = handler.handle_request(req).await.unwrap_or_else(|err| err);

        let id = res
            .header("x-correlation-id")
            .map(|h| h.as_str())
            .unwrap_or("-");
        let plugin = res
            .header("x-vlugin")
            .map(|h| h.as_str())
            .unwrap_or("unkown");
        let status: u16 = res.status().into();

        info!("[{}] {} {} {}", plugin, status, method, path, {
            id: id, status: status, nanos: instant.elapsed().as_nanos() as u64
        });

        Ok(res)
    })
    .await?;
    Ok(())
}
