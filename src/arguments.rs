use crate::types::{ContentType, Header};
use clap::Parser;

/// A simple HTTP server that prints receivec requests and returns a JSON response
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The port to listen on
    #[arg(short, long, default_value = "9000")]
    port: u16,

    /// The status code to return in the response
    #[arg(short, long, default_value = "200")]
    status_code: u16,

    /// The content to return
    #[arg(short, long)]
    content: Option<String>,

    /// The content type of the response
    #[arg(short = 't', long, default_value = "json")]
    content_type: ContentType,

    /// The headers to include in the response. Content-Type used here will override the
    /// `content_type` argument.
    /// Example usage: `--header "Content-Type: application/json, Authorization: Bearer 6"`
    #[arg(short = 'H', long, value_parser, num_args = 0.., value_delimiter = ',')]
    headers: Vec<Header>,

    /// The headers to highlight in the output
    /// Example usage: `--highlight-headers Content-Type,Authorization`
    #[arg(long, num_args = 0.., value_delimiter = ',')]
    highlight_headers: Vec<String>,
}

#[derive(Clone)]
pub struct Config {
    pub status_code: u16,
    pub content: Option<String>,
    pub content_type: ContentType,
    pub headers: Vec<Header>,
    pub highlight_headers: Vec<String>,
}

impl Args {
    pub fn to_config(&self) -> Config {
        Config {
            status_code: self.status_code,
            content: self.content.clone(),
            content_type: self.content_type.clone(),
            headers: self.headers.clone(),
            highlight_headers: self.highlight_headers.clone(),
        }
    }

    pub const fn get_port(&self) -> u16 {
        self.port
    }
}
