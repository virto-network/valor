#[async_std::main]
pub async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/").all(valor::handle_request);
    app.listen(("localhost", 8080)).await?;
    Ok(())
}
