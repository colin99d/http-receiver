#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use clap::Parser;
use colored::Colorize;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use arguments::Args;
use requests::handle_request;

mod arguments;
mod pretty_request;
mod requests;
mod types;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let config = args.to_config();

    let addr = SocketAddr::from(([127, 0, 0, 1], args.get_port()));
    let listener = TcpListener::bind(addr).await?;

    println!("{} Listening on http://{}", "[Started]".green(), addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let config_clone = config.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|x| handle_request(x, &config_clone)))
                .await
            {
                eprintln!("Error serving connection: {err}");
            }
        });
    }
}
