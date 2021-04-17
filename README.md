# Valiu Open Runtime [![Docs](https://docs.rs/valor_core/badge.svg)](https://docs.rs/valor_core)

A plugin based system that allows combining several independently developed Javascript, WebAssembly or native modules under a single API(HTTP only initially). 
"Vlugins" are simple message handlers that can run natively or in the Web and are recommended to use web friendly dependencies, like following the Web Worker API, so that the same file can run unchanged in the server as well as in the browser(intercepting HTTP calls from a service worker).

## "Vlugin" system

Whether it's compiled as a native library, a WASM or JavaScript module, vlugins implement the `Vlugin` interface that allows receiving compatible messages like http requests and answer back with another message like an HTTP response. 

When following the Web Worker API you'll find it's powerful enough to get most tasks done, you can write native plugins that access all sort of operating system APIs or web only plugins that access the DOM for example but it is recommended that you only access functionality available to Workers to maximize portability.
There's plenty of powerful APIs like `fetch`, `IndexDB` or `WebSocket` available that suit most common needs and for more complex services you can consider to use databases that talk HTTP like [CouchDB](https://couchdb.apache.org) or consider persisting state in a [blockchain](https://github.com/valibre-org/vln-node) or a decentralized storage like [IPFS](https://ipfs.io). The preferred way would definitely be writing Rust plugins that use dependencies that can compile to WASM like [Surf](https://github.com/http-rs/surf) for example, an HTTP client that uses `fetch` when compiled to WASM and a performant backend when compiled natively.

Plugin types:

|        | Rust(WASM) | JS | Rust(Native) | WASI |
|--------|------------|----|--------------|------|
| Server | ⚠️ | ⚠️ | ✅ | ❓ |
| Browser| ⚠️ | ⚠️ | ✖️ | ❓ |

> ⚠️ **Caution** with native plugins, they use the `extern "Rust"` ABI which is unstable so your plugins
> and the runtime should be compiled with the same `rustc` version. Also load only native plugins you trust
> and don't make the plugin registry API public as it may be potentially unsafe.

### Writing plugins

#### Rust plugins

The recommended way to get the best performance and support would be to create a plugin in Rust. 
The `#[vlugin]` macro makes it convenient to define plugins with a simple top-level function, taking care of
generating a `Vlugin` trait implementation and exporting a factory function that instantiates your plugin whether 
is compiled to WASM or as a native library.

```rust
use valor::*;

#[vlugin]
pub async fn on_request(_req: http::Request) -> http::Response {
    "OK response from a plugin".into()
}
```

For slightly more complex needs check the example [with state](examples/with_state/src/lib.rs).

#### JS plugins

> ⚠️ This is a work in progress, there aren't helpers yet and there's no support in the server(will come powered by Deno).

### Running plugins

#### Native

Use `valor_bin` to run a server that can automatically register plugins defined in a [JSON file](examples/plugins.json) or enable the `/_plugins` endpoint to register plugins dynamically. 
E.g. `LD_LIBRARY_PATH=plugins/ cargo run -- -p plugins.json -w`. Native plugins will be searched in the system's library path that in this example is set to the path where the compiled plugins are.

