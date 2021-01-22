//! Valor vlupin

use async_std::task;
use async_std::{
    net::{TcpListener, TcpStream},
    stream::StreamExt,
};
use kv_log_macro::{error, info, warn};
use loader::DynLoader;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;
use uuid::Uuid;

mod loader;

type Handler = valor::Handler<DynLoader>;

#[derive(StructOpt, Debug)]
#[structopt(name = "valor")]
struct Opt {
    /// Enables the plugin registry endpoint
    #[structopt(short)]
    with_registry: bool,

    /// Json file with the list of plugins to load at startup
    #[structopt(short)]
    plugin_file: Option<PathBuf>,
}

#[async_std::main]
async fn main() -> http_types::Result<()> {
    femme::with_level(femme::LevelFilter::Debug);
    let opt = Opt::from_args();

    let listener = TcpListener::bind(("0.0.0.0", 8080)).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    info!("listening on {}", addr);

    let mut handler = Handler::new(DynLoader).with_health();
    if opt.with_registry {
        handler = handler.with_registry()
    };

    if opt.plugin_file.is_some() {
        let file = File::open(opt.plugin_file.unwrap()).expect("Plugin file");
        let plugins: Vec<valor::Plugin> = serde_json::from_reader(file).expect("Plugin list");
        for p in plugins {
            handler
                .load_plugin(p)
                .await
                .unwrap_or_else(|err| warn!("{:?}", err));
        }
    }

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
            .header("x-valor-plugin")
            .map(|h| h.as_str())
            .unwrap_or("unkown");
        let status: u16 = res.status().into();

        if !path.starts_with("/_health") {
            info!("[{}] {} {} {}", plugin, status, method, path, {
                id: id, status: status, dur: instant.elapsed().as_millis() as u64
            });
        }

        Ok(res)
    })
    .await?;
    Ok(())
}
