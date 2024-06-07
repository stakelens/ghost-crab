// use crate::db::establish_connection;
// use diesel::PgConnection;
use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::rt::TokioIo;
use hyper_util::rt::TokioTimer;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use tokio::net::TcpListener;

pub struct RpcWithCache {
    // connection: PgConnection,
    rpc_url: String,
    pub url: String,
}

impl RpcWithCache {
    pub fn new(/*database_url: String, */ rpc_url: String) -> Self {
        // let connection = establish_connection(database_url);
        let url = "127.0.0.1:3000";

        Self {
            rpc_url,
            url: url.to_string(),
        }
    }

    pub async fn run(&self) {
        let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let rpc_url = self.rpc_url.clone();
            let (tcp, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(tcp);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .timer(TokioTimer::new())
                    .serve_connection(
                        io,
                        service_fn(move |request| handler(request, rpc_url.clone())),
                    )
                    .await
                {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }
}

async fn handler(
    request: Request<hyper::body::Incoming>,
    rpc_url: String,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let https = HttpsConnector::new();
    let client = Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);
    let request_received = request.collect().await.unwrap().to_bytes();

    println!("RPC request: {:?}", request_received);

    let rpc_quest = hyper::Request::builder()
        .method("POST")
        .uri(rpc_url)
        .header("Content-Type", "application/json")
        .body(Full::new(request_received.clone()))
        .unwrap();

    println!("Foward request: {:?}", rpc_quest);

    let rpc_response = client
        .request(rpc_quest)
        .await
        .unwrap()
        .collect()
        .await
        .unwrap()
        .to_bytes();

    println!("RPC response: {:?}", rpc_response);

    Ok(Response::new(Full::new(rpc_response)))
}
