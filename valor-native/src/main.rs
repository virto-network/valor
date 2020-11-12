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

#[async_trait::async_trait]
impl tide::Endpoint<()> for Handler {
    async fn call(&self, mut req: tide::Request<()>) -> tide::Result {
        if req.header("x-request-id").is_none() {
            req.insert_header("x-request-id", Uuid::new_v4().to_string());
        }

        self.0.handle_request(req).await.map(tide::Response::from)
    }
}
