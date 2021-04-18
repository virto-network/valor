use valor::*;

#[vlugin]
pub async fn on_create(cx: &mut Context) {
    let someone = cx.config::<String>().unwrap();
    cx.set(someone + " says hello");
}

pub async fn on_request(cx: &Context, req: http::Request) -> http::Result<http::Response> {
    let greeting = cx.get::<String>();
    let who = req
        .url()
        .query_pairs()
        .find(|(q, _)| q == "who")
        .ok_or(http::Error::from_str(
            http::StatusCode::BadRequest,
            "Missing buddy to greet",
        ))?
        .1;
    Ok(format!("{} {}!", greeting, who).into())
}
