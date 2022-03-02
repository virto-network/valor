use glommio::{CpuSet, LocalExecutorPoolBuilder, PoolPlacement};
use hyper::{Body, Method, Request, Response, StatusCode};
use std::{collections::HashMap, convert::Infallible};

use deno_core::anyhow;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;
mod http;

struct Message {
    meta: HashMap<String, Vec<u8>>,
    body: Vec<u8>,
}

fn new_js_runtime() -> anyhow::Result<JsRuntime> {
    const BOOTSTRAP_CODE: &str = include_str!("bootstrap_runtime.js");
    struct Permissions {}

    impl deno_timers::TimersPermission for Permissions {
        fn allow_hrtime(&mut self) -> bool {
            true
        }

        fn check_unstable(&self, _state: &deno_core::OpState, _api_name: &'static str) {}
    }

    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![
            deno_webidl::init(),
            deno_console::init(),
            deno_url::init(),
            deno_web::init(deno_web::BlobStore::default(), None),
            // deno_fetch::init::<Permissions>(deno_fetch::Options {
            //     user_agent: "Valor".into(),
            //     file_fetch_handler: Rc::new(deno_fetch::FsFetchHandler),
            //     ..Default::default()
            // }),
            deno_timers::init::<Permissions>(),
        ],
        ..Default::default()
    });

    runtime.execute_script("<vlugin>", BOOTSTRAP_CODE)?;
    // runtime.run_event_loop(false).await
    Ok(runtime)
}

async fn hello_deno(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => Ok(Response::new(Body::from("world"))),
        (&Method::GET, "/world") => Ok(Response::new(Body::from("hello"))),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("notfound"))
            .unwrap()),
    }
}

fn main() {
    println!("Starting server on port 8000");

    LocalExecutorPoolBuilder::new(PoolPlacement::MaxSpread(
        num_cpus::get(),
        CpuSet::online().ok(),
    ))
    .on_all_shards(|| async move {
        let id = glommio::executor().id();
        println!("Starting executor {}", id);
        let _runtime = new_js_runtime().expect(&format!("runtime {}", id));
        http::hyper_compat::serve_http(([0, 0, 0, 0], 8000), hello_deno, 1024)
            .await
            .unwrap();
    })
    .unwrap()
    .join_all();
}
