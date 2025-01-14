use clap::ValueEnum;
use flate2::read::{DeflateDecoder, GzDecoder};
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
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

#[derive(ValueEnum, Clone)]
pub enum ContentEncoding {
    Gzip,
    Deflate,
    Br,
    Zstd,
}

impl fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoding = match self {
            Self::Gzip => "gzip",
            Self::Deflate => "deflate",
            Self::Br => "br",
            Self::Zstd => "zstd",
        };
        write!(f, "{encoding}")
    }
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
            Self::Zstd => generic_decoder(&mut zstd::Decoder::new(data).or(Err(()))?),
        }
    }

    pub fn encode(&self, data: &str) -> Result<Vec<u8>, ()> {
        match self {
            Self::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data.as_bytes()).or(Err(()))?;
                encoder.finish().or(Err(()))
            }
            Self::Deflate => {
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data.as_bytes()).or(Err(()))?;
                encoder.finish().or(Err(()))
            }
            Self::Br => {
                let mut encoder = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
                encoder.write_all(data.as_bytes()).or(Err(()))?;
                encoder.flush().or(Err(()))?;
                Ok(encoder.into_inner())
            }
            Self::Zstd => {
                let mut encoder = zstd::Encoder::new(Vec::new(), 0).or(Err(()))?;
                encoder.write_all(data.as_bytes()).or(Err(()))?;
                encoder.finish().or(Err(()))
            }
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub status_code: u16,
    content: Option<Vec<u8>>,
    pub content_type: ContentType,
    pub content_encoding: Option<ContentEncoding>,
    pub headers: Vec<Header>,
    pub highlight_headers: Vec<String>,
}

impl Config {
    pub const fn new(
        status_code: u16,
        content: Option<Vec<u8>>,
        content_type: ContentType,
        content_encoding: Option<ContentEncoding>,
        headers: Vec<Header>,
        highlight_headers: Vec<String>,
    ) -> Self {
        Self {
            status_code,
            content,
            content_type,
            content_encoding,
            headers,
            highlight_headers,
        }
    }

    pub fn content(&self) -> Option<Vec<u8>> {
        self.content.clone()
    }
}
