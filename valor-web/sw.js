const files = [
  "/",
  "/favicon.ico",
  "/lib/valor.js",
  "/lib/valor_bg.wasm",
].map((f) => `${new URL(f, self.location)}`);

self.addEventListener("install", (e) => e.waitUntil(self.skipWaiting()));
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));
self.addEventListener("fetch", (e) => e.respondWith(handleRequest(e.request)));

const reqToTransferable = async (req) => ({
  url: req.url,
  method: req.method,
  headers: [...req.headers],
  body: await req.arrayBuffer(),
});

async function handleRequest(request) {
  const req = await reqToTransferable(request);
  return files.includes(req.url)
    ? fetch(request)
    : broadcastAndWaitResponse(req);
}

async function broadcastAndWaitResponse(req) {
  const rid = uuidv4();
  req.headers.push(["x-request-id", rid]);

  reqChan.postMessage(req);

  let resolve;
  const response = new Promise((r) => {
    resolve = r;
  });
  const id = await hash(rid);
  pendingRequests.set(id, resolve);

  return Promise.race([timeoutResponse(id), response]);
}

const reqChan = new BroadcastChannel("requests-channel");
const resChan = new BroadcastChannel("responses-channel");
const pendingRequests = new Map();

resChan.onmessage = ({ data = {} }) => {
  let { status, headers, body } = {
    ...{ status: 200, headers: {}, body: null },
    ...data,
  };

  headers = new Headers(headers);
  const id = headers.get("x-correlation-id");
  if (!id) return;

  const resolve = pendingRequests.get(id);
  if (!resolve) return;

  resolve(new Response(body, { status, headers }));
};

const TIMEOUT = 3000;
const timeout = (ms) => new Promise((res) => setTimeout(res, ms));
const timeoutResponse = (id) =>
  timeout(TIMEOUT).then(() => {
    pendingRequests.delete(id);
    return new Response("Timeout!", { status: 504 });
  });

function uuidv4() {
  return ([1e7] + -1e3 + -4e3 + -8e3 + -1e11).replace(
    /[018]/g,
    (c) =>
      (c ^ crypto.getRandomValues(new Uint8Array(1))[0] & 15 >> c / 4).toString(
        16,
      ),
  );
}

const bufferToB64 = (b) => btoa(String.fromCharCode(...new Uint8Array(b)));

async function hash(message) {
  message = new TextEncoder().encode(message);
  let digest = await crypto.subtle.digest("SHA-256", message);
  return bufferToB64(digest);
}
