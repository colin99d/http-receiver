use crate::types::{Config, ContentEncoding, ContentType, Header};
use clap::Parser;
use colored::Colorize;

/// A simple HTTP server that prints received requests and returns a JSON response
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The port to listen on
    #[arg(short, long, default_value = "9000")]
    port: u16,

    /// The host address to bind to (e.g. "127.0.0.1" or "0.0.0.0")
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    host: String,

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
    let clean_content = content?;
    if let Some(path) = clean_content.strip_prefix('@') {
        if let Ok(content) = std::fs::read(path) {
            return Some(content);
        }
        let warning =
            format!("Failed to read file: {path}, will send the value '{clean_content}' instead.");
        println!("{}", warning.yellow());
    }
    match encoding {
        None => Some(clean_content.as_bytes().to_vec()),
        Some(encoding) => encoding.encode(clean_content).ok(),
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

    pub fn get_host(&self) -> &str {
        &self.host
    }
}
