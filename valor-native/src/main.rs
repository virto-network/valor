use kv_log_macro::{debug, info};
use std::time::Instant;
use uuid::Uuid;

#[async_std::main]
pub async fn main() -> tide::Result<()> {
    femme::with_level(femme::LevelFilter::Debug);

    let handler = Handler(valor::Handler::new().await);
    let mut app = tide::new();
    app.at("/*").all(handler);

    app.listen(("localhost", 8080)).await?;
    Ok(())
}

struct Handler(valor::Handler);

const REQ_ID_HEADER: &'static str = "x-request-id";

#[async_trait::async_trait]
impl tide::Endpoint<()> for Handler {
    async fn call(&self, mut req: tide::Request<()>) -> tide::Result {
        let instant = Instant::now();

        if req.header(REQ_ID_HEADER).is_none() {
            req.insert_header(REQ_ID_HEADER, Uuid::new_v4().to_string());
        }
        let id = req.header(REQ_ID_HEADER).unwrap().as_str();
        let path = req.url().path().to_string();
        let method = req.method();
        debug!("received request {} {}", method, path, { id: id });

        let response = self.0.handle_request(req).await.map(tide::Response::from)?;

        let status: u16 = response.status().into();
        let plugin = response.header("x-valor-plugin").unwrap().as_str();
        let id = response.header("x-correlation-id").unwrap().as_str();
        info!("[{}] {} {} {}", plugin, status, method, path, {
            id: id, status: status, nanos: instant.elapsed().as_nanos() as u64
        });

        Ok(response)
    }
}
