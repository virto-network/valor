use async_std::io::Read;
use http_types::{Response, Result};

pub async fn handle_request(_req: impl Read) -> Result<Response> {
    println!("Handling request");
    Ok(().into())
}
