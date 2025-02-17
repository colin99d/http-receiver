use crate::types::ContentEncoding;
use colored::Colorize;
use http::header::HeaderMap;
use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::Request;
use std::fmt;
use std::str;

pub struct PrettyRequest {
    method: String,
    path: String,
    headers_str: String,
    body_str: String,
}

/// The list was gathered from the mozilla docs
/// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#syntax>
impl PrettyRequest {
    pub async fn from_hyper_request<B>(req: Request<B>, highlight_headers: &[String]) -> Self
    where
        B: HttpBody<Data = Bytes> + Send + 'static,
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        let method = req.method().to_string();
        let path = req
            .uri()
            .path_and_query()
            .map_or_else(|| String::from("/"), |p| p.as_str().to_owned());

        let headers_str = Self::format_headers(req.headers(), highlight_headers);
        let encoding = Self::get_encrpytion(req.headers());

        let body_str = (req.collect().await).map_or_else(
            |_| String::from("(error reading body)"),
            |body| Self::format_message(&body.to_bytes(), encoding.as_ref()),
        );

        Self {
            method,
            path,
            headers_str,
            body_str,
        }
    }

    fn format_message(body_bytes: &Bytes, encryption: Option<&ContentEncoding>) -> String {
        if body_bytes.is_empty() {
            return String::from("(no body)");
        }
        match encryption {
            None => str::from_utf8(body_bytes)
                .map_or("(non-UTF8 data)", |body_str| body_str)
                .to_string(),
            Some(value) => value
                .decode(body_bytes)
                .map_or("(error decoding)".to_string(), |body_str| body_str),
        }
    }

    fn get_encrpytion(headers: &HeaderMap) -> Option<ContentEncoding> {
        headers
            .get("content-encoding")
            .and_then(|encoding| encoding.to_str().ok())
            .and_then(ContentEncoding::from_str)
    }

    fn format_headers(headers: &HeaderMap, highlight_headers: &[String]) -> String {
        let mut output = String::new();
        for (name, value) in headers {
            let header_name = name.as_str().to_lowercase();
            let should_highlight = highlight_headers
                .iter()
                .any(|h| h.to_lowercase() == header_name);
            if should_highlight {
                output.push_str(&format!("  {}: {:?}\n", name.to_string().red(), value));
            } else {
                output.push_str(&format!("  {name}: {value:?}\n"));
            }
        }
        output
    }
}

impl fmt::Display for PrettyRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Method: {}", self.method)?;
        writeln!(f, "Path: {}", self.path)?;
        writeln!(f, "Headers:\n{}", self.headers_str)?;
        writeln!(f, "Body: {}", self.body_str)?;
        writeln!(f, "{}", "=".repeat(50))?;
        Ok(())
    }
}
