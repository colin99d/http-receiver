use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::{Request, Response, Result};

use crate::pretty_request::PrettyRequest;
use crate::types::Config;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

fn empty() -> BoxBody {
    Empty::new().map_err(|never| match never {}).boxed()
}

fn generate_response(config: &Config) -> http::Result<Response<BoxBody>> {
    let final_body = config.content().map_or_else(empty, full);

    let mut response_builder = Response::builder()
        .status(config.status_code)
        .header("Content-Type", config.content_type.to_string());
    if let Some(encoding) = &config.content_encoding {
        response_builder = response_builder.header("Content-Encoding", encoding.to_string());
    }

    for header in &config.headers {
        response_builder = response_builder.header(&header.key, &header.value);
    }

    response_builder.body(final_body)
}

pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    config: &Config,
) -> Result<Response<BoxBody>> {
    let pretty_req = PrettyRequest::from_hyper_request(req, &config.highlight_headers).await;
    println!("{pretty_req}");
    let response = generate_response(config).unwrap();
    Ok(response)
}
