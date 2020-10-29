struct Handler(valor::Handler);

#[async_std::main]
pub async fn main() -> tide::Result<()> {
    let handler = Handler(valor::Handler::new());
    let mut app = tide::new();
    app.at("/*").all(handler);

    app.listen(("localhost", 8080)).await?;
    Ok(())
}

#[async_trait::async_trait]
impl tide::Endpoint<()> for Handler {
    async fn call(&self, req: tide::Request<()>) -> tide::Result {
        self.0
            .handle_request(req.into())
            .await
            .map(tide::Response::from)
    }
}
