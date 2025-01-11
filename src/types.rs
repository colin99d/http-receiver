use clap::ValueEnum;
use flate2::read::{DeflateDecoder, GzDecoder};
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::str::FromStr;

#[derive(ValueEnum, Clone)]
pub enum ContentType {
    /// JSON (`application/json`)
    Json,
    /// Text (`text/plain`)
    Text,
    /// HTML (`text/html`)
    Html,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mime_type = match self {
            Self::Json => "application/json",
            Self::Text => "text/plain",
            Self::Html => "text/html",
        };
        write!(f, "{mime_type}")
    }
}

#[derive(Clone)]
pub struct Header {
    pub key: String,
    pub value: String,
}

impl FromStr for Header {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(From::from(format!("Improperly formatted header: {s}")));
        }
        Ok(Self {
            key: parts[0].trim().to_string(),
            value: parts[1].trim().to_string(),
        })
    }
}

fn generic_decoder(decoder: &mut dyn Read) -> Result<String, ()> {
    let mut result = String::new();
    decoder.read_to_string(&mut result).or(Err(()))?;
    Ok(result)
}

pub enum ContentEncoding {
    Gzip,
    Deflate,
    Br,
    Zstd,
}

impl ContentEncoding {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gzip" => Some(Self::Gzip),
            "deflate" => Some(Self::Deflate),
            "br" => Some(Self::Br),
            "zstd" => Some(Self::Zstd),
            _ => None,
        }
    }

    pub fn decode(&self, data: &[u8]) -> Result<String, ()> {
        match self {
            Self::Gzip => generic_decoder(&mut GzDecoder::new(data)),
            Self::Deflate => generic_decoder(&mut DeflateDecoder::new(data)),
            Self::Br => generic_decoder(&mut brotli::Decompressor::new(data, 4096)),
            Self::Zstd => generic_decoder(&mut zstd::Decoder::new(data).unwrap()),
        }
    }
}
