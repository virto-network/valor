//! ValorBin it's the native runtime that is able to load vlugins
//! from a JSON configuration file and serve incoming HTTP requests.
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
use std::net::SocketAddr;

use kv_log_macro as log;
// use loader::Loader;
use serde::Deserialize;
use std::{fs::File, path::PathBuf, time::Instant};
use structopt::StructOpt;
// use uuid::Uuid;
use valor::runtime;
// use valor::{http, Vlugin};
use deno_core::anyhow::Result;

// mod loader;

// type Runtime = runtime::Runtime<Loader>;

#[derive(StructOpt, Debug)]
#[structopt(name = "valor")]
struct Opt {
    /// Enables the plugin registry endpoint
    #[structopt(short)]
    with_registry: bool,

    /// Json file with the list of plugins to load at startup
    #[structopt(short)]
    plugin_file: Option<PathBuf>,

    /// Server address to listen for new connections
    #[structopt(short, default_value = "0.0.0.0:8080")]
    address: SocketAddr,
}

#[derive(Deserialize)]
struct ConfigFile {
    plugins: Vec<runtime::VluginDef>,
}

#[tokio::main]
async fn main() -> Result<()> {
    femme::with_level(femme::LevelFilter::Debug);
    let opt = Opt::from_args();

    if opt.plugin_file.is_some() {
        let file = File::open(opt.plugin_file.unwrap())?;
        let config: ConfigFile = serde_json::from_reader(file)?;
        for p in config.plugins {
            log::debug!("got plugin {}", p.name);
            // runtime
            //     .load_plugin(p)
            //     .await
            //     .unwrap_or_else(|err| warn!("{}", err));
        }
    }

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    log::debug!("Server listening at {}", &opt.address);
    let server = Server::bind(&opt.address).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        log::error!("server error: {}", e);
    }
    Ok(())
}

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}

// #[tokio::main]
// async fn main() {
//     femme::with_level(femme::LevelFilter::Debug);
//     run(Opt::from_args())
//         //.catch_unwind()
//         .await
//         .unwrap_or_else(|e| error!("{}", e));
// }

// async fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
//     let listener = TcpListener::bind(("0.0.0.0", 8080))
//         .await
//         .expect("Bind address");
//     let addr = format!("http://{}", listener.local_addr()?);
//     info!("listening on {}", addr);

//     let mut runtime = Runtime::new(Loader::default()).with_health()?;
//     if opt.with_registry {
//         runtime = runtime.with_registry()?;
//     }

//     if opt.plugin_file.is_some() {
//         let file = File::open(opt.plugin_file.unwrap())?;
//         let config: ConfigFile = serde_json::from_reader(file)?;
//         for p in config.plugins {
//             runtime
//                 .load_plugin(p)
//                 .await
//                 .unwrap_or_else(|err| warn!("{}", err));
//         }
//     }

//     let mut incoming = listener.incoming();
//     while let Some(Ok(stream)) = incoming.next().await {
//         let runtime = runtime.clone();
//         task::spawn_local(async move {
//             if let Err(err) = accept(stream, runtime).await {
//                 error!("{}", err);
//             }
//         });
//     }
//     Err("Stream closed".into())
// }

// const REQ_ID_HEADER: &str = "x-request-id";

// async fn accept(stream: TcpStream, runtime: Runtime) -> Result<(), valor::Error> {
//     async_h1::accept(stream.clone(), |mut req| async {
//         let instant = Instant::now();
//         if req.header(REQ_ID_HEADER).is_none() {
//             let id = Uuid::new_v4().to_string();
//             req.insert_header(REQ_ID_HEADER, id);
//         }

//         let method = req.method();
//         let path = req.url().path().to_string();

//         let res: http::Response = match runtime.on_msg(req.into()).await {
//             Ok(res) => res.into(),
//             Err(err) => match err {
//                 valor::Error::Http(err) => err.status().into(),
//                 err => return Err(err.into()),
//             },
//         };

//         let id = res
//             .header("x-correlation-id")
//             .map(|h| h.as_str())
//             .unwrap_or("-");
//         let plugin = res
//             .header("x-valor-plugin")
//             .map(|h| h.as_str())
//             .unwrap_or("unkown");
//         let status: u16 = res.status().into();

//         if !path.starts_with("/_health") {
//             if res.status().is_server_error() {
//                 warn!("[{}] {} {} {}", plugin, status, method, path, {
//                     id: id, status: status, dur: instant.elapsed().as_millis() as u64
//                 });
//             } else {
//                 info!("[{}] {} {} {}", plugin, status, method, path, {
//                     id: id, status: status, dur: instant.elapsed().as_millis() as u64
//                 });
//             }
//         }

//         Ok(res)
//     })
//     .await?;
//     Ok(())
// }
