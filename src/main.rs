#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::option_if_let_else)]
use clap::Parser;
use colored::Colorize;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use http::Uri;
use std::net::SocketAddr;

use arguments::Args;
use requests::{handle_request, empty};
use types::Config;

mod arguments;
mod pretty_request;
mod requests;
mod types;

async fn echo_server(config: Config, address: &SocketAddr) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;

    println!("{} Listening on http://{}", "[Started]".green(), address);

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

async fn gateway_server(in_addr: &SocketAddr, out_addr: SocketAddr, highlight: &[String]) -> std::io::Result<()> {

    let listener = TcpListener::bind(in_addr).await?;

    let out_arc = Arc::new(out_addr);

    println!("Listening on http://{}", in_addr);
    println!("Proxying on {}", out_addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let out_clone = Arc::clone(&out_arc);

        let service = service_fn(move |mut req| {
            let new_uri: Uri = req.uri().path_and_query().unwrap().as_str().parse().unwrap();
            *req.uri_mut() = new_uri;


            let target_addr = Arc::clone(&out_clone);
            async move {
                // This unwrap means that if the forward URL doesnt work we get panics
                println!("Forwarding to {}", out_addr);
                let client_stream = TcpStream::connect(*target_addr).await.unwrap();
                let io = TokioIo::new(client_stream);

                let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        println!("Connection failed: {:?}", err);
                    }
                });

                sender.send_request(req).await
            }
        });

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve the connection: {:?}", err);
            }
        });
    }
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let config = args.to_config();
    let address = args.get_address();
    if let Some(socket) = args.get_socket() {
        gateway_server(&address, socket, &args.get_highlight_headers()).await
    } else {
        echo_server(config, &address).await
    }
}
