use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::{Request, Response, Result};

use crate::arguments::Config;
use crate::pretty_request::PrettyRequest;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty() -> BoxBody {
    Empty::new().map_err(|never| match never {}).boxed()
}

pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    config: &Config,
) -> Result<Response<BoxBody>> {
    let pretty_req = PrettyRequest::from_hyper_request(req, &config.highlight_headers).await;
    println!("{pretty_req}");

    let body = config
        .content
        .as_ref()
        .map_or_else(empty, |content| full(content.clone()));

    let response = Response::builder()
        .status(config.status_code)
        .header("Content-Type", config.content_type.to_string())
        .body(body)
        .unwrap();
    Ok(response)
}
