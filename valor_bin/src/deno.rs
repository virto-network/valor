use std::rc::Rc;

use deno_core::anyhow::Result;
use deno_core::JsRuntime;
use deno_core::RuntimeOptions;

const BOOTSTRAP_CODE: &str = r#"
"use strict";

((window) => {
    const {
        ObjectDefineProperties,
    } = window.__bootstrap.primordials;
    const core = Deno.core;
    const timers = window.__bootstrap.timers;
    const Console = window.__bootstrap.console.Console;
    const fetch = window.__bootstrap.fetch;

    const util = {
        writable(value) {
            return {
                value,
                writable: true,
                enumerable: true,
                configurable: true,
            }
        },
        nonEnumerable(value) {
            return {
                value,
                writable: true,
                enumerable: false,
                configurable: true,
            }
        },
    }

    const globalScope = {
        console: util.nonEnumerable(
            new Console((msg, level) => core.print(msg, level > 1))
        ),
        fetch: util.writable(fetch.fetch),
        setInterval: util.writable(timers.setInterval),
        setTimeout: util.writable(timers.setTimeout),
        clearInterval: util.writable(timers.clearInterval),
        clearTimeout: util.writable(timers.clearTimeout),
        Request: util.nonEnumerable(fetch.Request),
        Response: util.nonEnumerable(fetch.Response),
    };
    ObjectDefineProperties(globalThis, globalScope);

    const consoleFromV8 = window.console;
    const wrapConsole = window.__bootstrap.console.wrapConsole;
    const consoleFromDeno = globalThis.console;
    wrapConsole(consoleFromDeno, consoleFromV8);

    delete globalThis.__bootstrap;
    delete globalThis.bootstrap

    // runtime start
    core.setMacrotaskCallback(timers.handleTimerMacrotask);
    core.setWasmStreamingCallback(fetch.handleWasmStreaming);
})(this);

(async function() {
    console.log('fetching...');
    let code = await fetch('./test_plugin/test_plugin_bg.wasm');
    console.log('code> ', code);
    //code = await code.arrayBuffer();
    //const wasm = (await WebAssembly.instantiate(code, {})).instance;
    //console.log(wasm.on_create());
})();
"#;

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
