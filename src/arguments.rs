use crate::types::{Config, ContentEncoding, ContentType, Header};
use clap::Parser;
use colored::Colorize;
use std::net::IpAddr;

/// A simple HTTP server that prints received requests and returns a JSON response
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The port to listen on
    #[arg(short, long, default_value = "9000")]
    port: u16,

    /// The host address to bind to (e.g. "127.0.0.1" or "0.0.0.0")
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    host: IpAddr,

    /// The status code to return in the response
    #[arg(short, long, default_value = "200")]
    status_code: u16,

    /// The content to return
    #[arg(short, long)]
    content: Option<String>,

    /// The content type of the response
    #[arg(short = 't', long, default_value = "json")]
    content_type: ContentType,

    /// The content encoding of the response
    #[arg(short = 'e', long)]
    content_encoding: Option<ContentEncoding>,

    /// The headers to include in the response. Headers set here will override what
    /// might be sent as a result of a different argument being selected.
    /// i.e. content-type and content-encoding
    /// Example usage: `--header "Content-Type: application/json, Authorization: Bearer 6"`
    #[arg(short = 'H', long, value_parser, num_args = 0.., value_delimiter = ',')]
    headers: Vec<Header>,

    /// The headers to highlight in the output
    /// Example usage: `--highlight-headers Content-Type,Authorization`
    #[arg(long, num_args = 0.., value_delimiter = ',')]
    highlight_headers: Vec<String>,
}

pub fn get_content_bytes(
    content: Option<&str>,
    encoding: Option<&ContentEncoding>,
) -> Option<Vec<u8>> {
    let clean = content?;
    if let Some(path) = clean.strip_prefix('@') {
        if let Ok(data) = std::fs::read(path) {
            match encoding {
                None => {
                    if String::from_utf8(data.clone()).is_err() {
                        println!("{}",  "Failed to decode file content as UTF-8, try adding a content encoding.".red());
                    }
                }
                Some(inner_encoding) => {
                    if inner_encoding.decode(&data).is_err() {
                        println!(
                            "{}",
                            "Failed to decode file content, try a different encoding.".red()
                        );
                    }
                }
            }
            return Some(data);
        }
        let warning = format!("Failed to read file: {path}, sending the value '{clean}' instead.");
        println!("{}", warning.yellow());
    }
    match encoding {
        None => Some(clean.as_bytes().to_vec()),
        Some(encoding) => encoding.encode(clean).ok(),
    }
}

impl Args {
    pub fn to_config(&self) -> Config {
        let content = get_content_bytes(self.content.as_deref(), self.content_encoding.as_ref());
        Config::new(
            self.status_code,
            content,
            self.content_type.clone(),
            self.content_encoding.clone(),
            self.headers.clone(),
            self.highlight_headers.clone(),
        )
    }

    pub const fn get_port(&self) -> u16 {
        self.port
    }

    pub const fn get_host(&self) -> IpAddr {
        self.host
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle_request;
    use crate::requests::empty;
    use crate::types::{ContentEncoding, ContentType, Header};
    use http_body_util::BodyExt;
    use hyper::{Request, StatusCode};
    use std::io::Read;
    use std::net::Ipv4Addr;
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

    fn test_encoding(encoding: ContentEncoding) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let original_text = "Check test".to_string();
            let args = Args {
                port: 9000,
                host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                status_code: 200,
                content: Some(original_text.clone()),
                content_type: ContentType::Json,
                content_encoding: Some(encoding.clone()),
                headers: vec![],
                highlight_headers: vec![],
            };

            let config = args.to_config();

            let (status, body_bytes, headers) = run_handle_request(&config).await;

            assert_eq!(status, StatusCode::OK);

            let enc_header = headers.get("Content-Encoding").unwrap();
            assert_eq!(enc_header.to_str().unwrap(), encoding.to_string());

            println!("{:?}", body_bytes);
            let encoded = encoding.encode(&original_text).unwrap();
            assert_eq!(encoded, body_bytes);
        });
    }

    #[test]
    fn test_handle_request_gzip() {
        test_encoding(ContentEncoding::Gzip);
    }

    #[test]
    fn test_handle_request_deflate() {
        test_encoding(ContentEncoding::Deflate);
    }

    #[test]
    fn test_handle_request_br() {
        test_encoding(ContentEncoding::Br);
    }

    #[test]
    fn test_handle_request_zstd() {
        test_encoding(ContentEncoding::Zstd);
    }
}
