use clap::Parser;
use serde_json::Value;
use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{
    service::service_fn,
    Request, Response, Result,
};
use tokio::net::TcpListener;
use std::str;
use std::net::SocketAddr;
use hyper::server::conn::http1;
use std::sync::Arc;
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "200")]
    status_code: u16,

    #[arg(short, long, default_value = "{}")]
    json: String,

    #[arg(short, long, default_value = "9000")]
    port: u16,
}


fn get_body_message(body_bytes: &hyper::body::Bytes) -> String {
    if body_bytes.is_empty() {
        return String::from("Body: (no body)")
    }
    match str::from_utf8(&body_bytes) {
        Ok(body_str) => return format!("Body: {}", body_str),
        Err(_) => return String::from("Body: (non-UTF8 data)")
    };
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}


async fn handle_request(req: Request<hyper::body::Incoming>, response_code: u16, json_response: Arc<Value>) -> Result<Response<BoxBody>> {
    let method = &req.method();
    println!("Method: {}", method);
    println!("Path: {}", req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/"));

    println!("Headers:");
    for (name, value) in req.headers().iter() {
        println!("  {}: {:?}", name, value);
    }

    let body_bytes = req.collect().await.unwrap().to_bytes();
    let body_str = get_body_message(&body_bytes);
    println!("{}", body_str);

    println!("{}", "-".repeat(50));


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
    let json_body: Value = serde_json::from_str(&args.json).unwrap();
    let json_item = Arc::new(json_body);

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let json_instance = Arc::clone(&json_item);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|x| handle_request(x, args.status_code, json_instance.clone())))
                .await
            {
                eprintln!("Error serving connection: {}", err);
            }
        });
    }
}
