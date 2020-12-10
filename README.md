# Valiu Open Runtime

A plug-in based system that allows combining several independently developed Javascript, WebAssembly or native modules under a single HTTP API. Plug-ins are simple request handlers that can run natively or in the Web and are recommended to follow the Web Worker API so that the same file can run unchanged in the server as well as in the browser(intercepting HTTP calls from a service worker).

## Plug-in system

Whether it's compiled as a native library, a WASM or JavaScript module, plug-ins implement a `RequestHandler` interface that allows them to receive requests and are expected to return a valid response. Since the Web Worker API is quite powerful, it is quite suitable to use as the baseline for our plugins, you can write native plugins that access all sort of operating system APIs or web only plugins that access the DOM for example but it is recommended that you only access functionality available to Workers to maximize portability. There's plenty of powerful APIs like `fetch`, `IndexDB` or `WebSocket` available that suit most common needs and for more complex services you can consider to use databases that talk HTTP like [CouchDB](https://couchdb.apache.org) or consider persisting state in a [blockchain](https://github.com/valibre-org/node) or decentralized storage like [IPFS](https://ipfs.io) for example. The prefered way would defenetily be writing Rust plugins that use dependencies that can compile to WASM like [Surf](https://github.com/http-rs/surf) for example, an HTTP client that uses `fetch` when compiled to WASM and a performant backend when compiled natively.

Plugin types:

|        | Rust(WASM) | JS | Rust(Native) | WASI |
|--------|------------|----|--------------|------|
| Server | ⚠️ | ⚠️ | ✅ | ❓ |
| Browser| ⚠️ | ⚠️ | ✖️ | ❓ |

### Handling HTTP requests

#### Rust plugins

The recommended way to get the best performance and support would be to create a plugin in Rust. Simply include the helper macro and types, then declare request your handler.

```rust
use valor::*;

#[vlugin]
async fn my_handler(req: Request) -> Response {
    "OK response from a plugin".into()
}
```

Depending on the compilation target it will adapt to use the browser's built-in WASM compiler or run natively server.

#### JS plugins

> ⚠️ This is a work in progress, there aren't helpers yet and there's no support in the server(will come powered by Deno).
