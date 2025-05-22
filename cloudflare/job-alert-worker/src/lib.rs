use worker::*;

#[event(email)]
async fn main(
    _message: EmailMessage,
    _env: Env,
    _ctx: Context,
) -> Result<HttpResponse> {
    console_error_panic_hook::set_once();

    // Ok(http::Response::builder()
    //     .status(http::StatusCode::OK)
    //     .body(Body::empty())?)

}

