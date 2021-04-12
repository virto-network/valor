use valor::*;

#[vlugin]
pub async fn on_create(cx: &mut Context) {
    cx.set("some state");
}

pub async fn on_request(cx: &Context, _req: http::Request) -> http::Result<http::Response> {
    cx.get::<&str>()
        .ok_or(http::Error::from_str(
            http::StatusCode::InternalServerError,
            "Not possible",
        ))
        .map(|s| s.to_string().into())
}
