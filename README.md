# Valibre Open Runtime

A plug-in based system that allows running Javascript or WebAssembly modules that register HTTP API handlers. Plug-ins follow the WebWorker API so that the same system can run unchanged in a server environment(with the help of deno-core) as well as in a web browser(intercepting HTTP calls from a service worker).

## Plug-in system - WebWorkers all the way

The Web Worker API is simple yet quite powerful, allows for asynchronous receiving of messsages using the `onmessage` handler and sending messages back to the host with the `postMessage()` function, long running blocking tasks are not a problem since a worker is executed in its own thread and it comes with a variety of high level APIs like `fetch`, `IndexDB` and `WebSocket` among others.

### Handling HTTP requests

**First register the plug-in** dynamically with a `POST` to the `/_plugin` endpoint.  
JSON Body:

```json
{
  "name": "<plug-in name>",
  "url": "<plug-in URL>", 
  "prefix": "<URL prefix>"
}
```
Incoming requests are matched with the prefix and sent to the registered worker. The worker receives a JSON message with details about the request and its body.

#### Request message example
```json
{
  "rid": 123,
  "uri": "/path/without/prefix",
  "method": "GET",
  "type": "json",
  "body": null
}
```
> **TODO** Define how multipart form/data should be received.

To reply you can `posetMessage()` a reply containing the request ID(`rid`) so the host can reply to the waiting client.

#### Response example
```json
{
  "rid": 123,
  "status": 200,
  "body": {}
}
```

### Inter plug-in communication

> **TODO**
