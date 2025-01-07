#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use clap::Parser;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{service::service_fn, Request, Response, Result};
use hyper_util::rt::TokioIo;
use pretty_request::PrettyRequest;
use std::net::SocketAddr;
use std::str;
use tokio::net::TcpListener;

mod pretty_request;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

/// A simple HTTP server that prints receivec requests and returns a JSON response
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The status code to return in the response
    #[arg(short, long, default_value = "200")]
    status_code: u16,

    /// The JSON response to return
    #[arg(short, long, default_value = "{}")]
    json: String,

    /// The port to listen on
    #[arg(short, long, default_value = "9000")]
    port: u16,

    /// The headers to highlight in the output
    #[arg(short = 'H', long, num_args = 0.., value_delimiter = ',')]
    highlight_headers: Vec<String>,
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    response_code: u16,
    json_response: &str,
    highlight_headers: &[String],
) -> Result<Response<BoxBody>> {
    let pretty_req = PrettyRequest::from_hyper_request(req, highlight_headers).await;
    println!("{pretty_req}");

    let response = Response::builder()
        .status(response_code)
        .header("Content-Type", "application/json")
        .body(full(json_response.to_string()))
        .unwrap();
    Ok(response)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let json_clone = args.json.clone();
        let headers_clone = args.highlight_headers.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|x| {
                        handle_request(x, args.status_code, &json_clone, &headers_clone)
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {err}");
            }
        });
    }
}
