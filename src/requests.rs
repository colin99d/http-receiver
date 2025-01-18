use http_body::Body as HttpBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::{Request, Response};

use crate::pretty_request::PrettyRequest;
use crate::types::Config;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub fn empty() -> BoxBody {
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

pub async fn handle_request<B>(req: Request<B>, config: &Config) -> hyper::Result<Response<BoxBody>>
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    let pretty_req = PrettyRequest::from_hyper_request(req, &config.highlight_headers).await;
    println!("{pretty_req}");
    let response = generate_response(config).unwrap();
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentEncoding, ContentType, Header};
    use hyper::{Request, StatusCode};
    use std::io::Read;
    use tokio::runtime::Runtime;

    async fn run_handle_request(config: &Config) -> (StatusCode, Vec<u8>, http::HeaderMap) {
        let req = Request::builder()
            .uri("http://localhost/test")
            .body(empty())
            .unwrap();

        let response = handle_request(req, config).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();

        let body_bytes = response.collect().await.unwrap().to_bytes().to_vec();
        (status, body_bytes, headers)
    }

    #[test]
    fn test_generate_response_no_encoding() {
        let config = Config::new(
            200,
            Some(b"Hello World!".to_vec()),
            ContentType::Text,
            None,
            vec![],
            vec![],
        );

        let response = generate_response(&config).unwrap();
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/plain"
        );
        assert!(response.headers().get("Content-Encoding").is_none());
    }

    #[test]
    fn test_handle_request_no_encoding_body() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let config = Config::new(
                200,
                Some(b"Test data".to_vec()),
                ContentType::Text,
                None,
                vec![],
                vec![],
            );

            let (status, body_bytes, headers) = run_handle_request(&config).await;

            assert_eq!(status, StatusCode::OK);
            assert!(headers.get("Content-Encoding").is_none());
            assert_eq!(String::from_utf8(body_bytes).unwrap(), "Test data");
        });
    }

    #[test]
    fn test_generate_response_with_additional_headers() {
        let headers = vec![
            Header {
                key: "X-Custom-Header".to_string(),
                value: "CustomValue".to_string(),
            },
            Header {
                key: "Another-Header".to_string(),
                value: "AnotherValue".to_string(),
            },
        ];

        let config = Config::new(
            201,
            Some(b"Test content".to_vec()),
            ContentType::Text,
            None,
            headers,
            vec![],
        );

        let response = generate_response(&config).unwrap();
        assert_eq!(response.status(), 201);
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/plain"
        );

        assert_eq!(
            response.headers().get("X-Custom-Header").unwrap(),
            "CustomValue"
        );
        assert_eq!(
            response.headers().get("Another-Header").unwrap(),
            "AnotherValue"
        );
    }
}
