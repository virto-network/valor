self.addEventListener("install", (e) => e.waitUntil(self.skipWaiting()));
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));
self.addEventListener("fetch", (e) => e.respondWith(handleRequest(e.request)));

const hasWhitelistedExt = (url) =>
  [".html", ".css", ".ico", ".js", ".wasm"].some((ext) => url.endsWith(ext));

const toObjLiteral = (iter) =>
  [...iter].reduce((o, [k, v]) => {
    o[k] = v;
    return o;
  }, {});

const reqToTransferable = async (req) => {
  const body = await req.arrayBuffer();
  return {
    url: req.url,
    init: {
      method: req.method,
      headers: toObjLiteral(req.headers),
      body,
    },
  };
};

const handleRequest = (req) =>
  hasWhitelistedExt(req.url) ? fetch(req) : broadcastAndWaitResponse(req);

const TIMEOUT = 3000;
const timeout = (ms) => new Promise((res) => setTimeout(res, ms));
const timeoutResponse = (id) =>
  timeout(TIMEOUT).then(() => {
    pendingRequests.delete(id);
    return new Response("Timeout!", { status: 504 });
  });

async function broadcastAndWaitResponse(req) {
  req = await reqToTransferable(req);
  const rid = uuidv4();
  req.headers["x-request-id"] = rid;

  reqChan.postMessage(req, [req.body]);

  let resolve;
  const response = new Promise((r) => {
    resolve = r;
  });
  pendingRequests.set(rid, resolve);

  return Promise.race([timeoutResponse(rid), response]);
}

const reqChan = new BroadcastChannel("req_channel");
const resChan = new BroadcastChannel("res_channel");
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

function uuidv4() {
  return ([1e7] + -1e3 + -4e3 + -8e3 + -1e11).replace(
    /[018]/g,
    (c) =>
      (c ^ crypto.getRandomValues(new Uint8Array(1))[0] & 15 >> c / 4).toString(
        16,
      ),
  );
}
