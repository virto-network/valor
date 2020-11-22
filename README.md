# Valibre Open Runtime

A plug-in based system that allows running Javascript, WebAssembly or native modules that handle HTTP API requests. Plug-ins are expected to follow the WebWorker API so that the same file can run unchanged in a server environment as well as in a web browser(intercepting HTTP calls from a service worker).

## Plug-in system

Since the Web Worker API is quite powerful and simple to use, i.e. we only receive asynchronous messsages with the `onmessage` callback and answer back the host with the `postMessage()` function, is quite suitable to use as the baseline for our plugins. We can for example have long running blocking tasks because of the worker's nature of being executed in its own thread. Also comes with a variety of high level APIs like `fetch`, `IndexDB` and `WebSocket` among others which suit most common needs. That's why even for native Rust plugins is recommended to only use dependencies that can compile to WASM and run in the context of a worker.

Plugin types:

|        | Rust(WASM Worker) | JS(Worker) | Rust(Native) | WASI |
|--------|-------------------|------------|--------------|------|
| Server | ⚠️ | ⚠️ | ⚠️ | ❓ |
| Browser| ⚠️ | ⚠️ | ✖️ | ❓ |

### Handling HTTP requests

#### Rust plugins

The recommended way to get the best performance and support would be to create a plugin in Rust. Simply include the helper macro and types and declare your handler.

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

As it runs in a Worker, you would listen to the `message` event receiving request messages that look like:

```js
const request = {
  url: 'some/url',
  method: 'GET',
  headers: [['x-request-id', '123abc']], // list of headers
  body: null, // or an ArrayBuffer
}
```
The response would then look like
```js
postMessage({
  status: 200,
  headers: [['x-correlation-id', '123abc']],
  body: null, // or an ArrayBuffer
})
```

