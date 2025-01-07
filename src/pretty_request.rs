use colored::Colorize;
use http::header::HeaderMap;
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

impl PrettyRequest {
    pub async fn from_hyper_request(
        req: Request<hyper::body::Incoming>,
        highlight_headers: &[String],
    ) -> Self {
        let method = req.method().to_string();
        let path = req
            .uri()
            .path_and_query()
            .map_or_else(|| String::from("/"), |p| p.as_str().to_owned());

        let headers_str = Self::format_headers(req.headers(), highlight_headers);

        let body_bytes = req.collect().await.unwrap().to_bytes();
        let body_str = Self::format_message(&body_bytes);

        Self {
            method,
            path,
            headers_str,
            body_str: body_str.to_string(),
        }
    }

    fn format_message(body_bytes: &Bytes) -> &str {
        if body_bytes.is_empty() {
            return "(no body)";
        }
        str::from_utf8(body_bytes).map_or("(non-UTF8 data)", |body_str| body_str)
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
        Ok(())
    }
}
