use std::rc::Rc;

use deno_core::anyhow::Result;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;

// stripped down version of https://github.com/denoland/deno/blob/main/runtime/js/99_main.js
const BOOTSTRAP_CODE: &str = include_str!("bootstrap_runtime.js");

#[tokio::main]
async fn main() -> Result<()> {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![
            deno_webidl::init(),
            deno_console::init(),
            deno_url::init(),
            deno_web::init(deno_web::BlobStore::default(), None),
            deno_fetch::init::<Permissions>(deno_fetch::Options {
                user_agent: "Valor".into(),
                file_fetch_handler: Rc::new(deno_fetch::FsFetchHandler),
                ..Default::default()
            }),
            deno_timers::init::<Permissions>(),
        ],
        ..Default::default()
    });

    runtime.execute_script("<vlugin>", BOOTSTRAP_CODE)?;
    runtime.run_event_loop(false).await
}

struct Permissions {}

impl deno_timers::TimersPermission for Permissions {
    fn allow_hrtime(&mut self) -> bool {
        true
    }

    fn check_unstable(&self, _state: &deno_core::OpState, _api_name: &'static str) {}
}

impl deno_fetch::FetchPermissions for Permissions {
    fn check_net_url(&mut self, _url: &deno_core::url::Url) -> Result<()> {
        Ok(())
    }

    fn check_read(&mut self, _p: &std::path::Path) -> Result<()> {
        Ok(())
    }
}
