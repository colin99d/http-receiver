use clap::{Parser, ValueEnum};
use std::fmt;

#[derive(ValueEnum, Debug, Clone)]
pub enum ContentType {
    /// JSON (`application/json`)
    Json,
    /// Text (`text/plain`)
    Text,
    /// HTML (`text/html`)
    Html, // Unknown(String),
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mime_type = match self {
            Self::Json => "application/json",
            Self::Text => "text/plain",
            Self::Html => "text/html",
            // Self::Unknown(ref custom) => custom,
        };
        write!(f, "{mime_type}")
    }
}

/// A simple HTTP server that prints receivec requests and returns a JSON response
#[derive(Parser, Debug)]
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

    /// The headers to highlight in the output
    /// Example usage: `--highlight-headers Content-Type,Authorization`
    #[arg(short = 'H', long, num_args = 0.., value_delimiter = ',')]
    highlight_headers: Vec<String>,
}

#[derive(Clone)]
pub struct Config {
    pub status_code: u16,
    pub content: Option<String>,
    pub content_type: ContentType,
    pub highlight_headers: Vec<String>,
}

impl Args {
    pub fn to_config(&self) -> Config {
        Config {
            status_code: self.status_code,
            content: self.content.clone(),
            content_type: self.content_type.clone(),
            highlight_headers: self.highlight_headers.clone(),
        }
    }

    pub const fn get_port(&self) -> u16 {
        self.port
    }
}
