use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use blake3;
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
use rocksdb::DB;
use tokio::net::TcpListener;

pub struct RpcWithCache {
    rpc_url: Arc<String>,
    cache: Arc<DB>,
    port: u16,
}

impl RpcWithCache {
    pub fn new(network: String, rpc_url: String, port: u16) -> Self {
        let current_dir = std::env::current_dir().unwrap();
        let cache = Arc::new(DB::open_default(current_dir.join("cache").join(network)).unwrap());

        Self {
            rpc_url: Arc::new(rpc_url),
            cache,
            port,
        }
    }

    pub async fn run(&self) {
        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (tcp, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(tcp);

            let db = Arc::clone(&self.cache);
            let rpc_url = Arc::clone(&self.rpc_url);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .timer(TokioTimer::new())
                    .serve_connection(
                        io,
                        service_fn(|request| {
                            handler(request, Arc::clone(&rpc_url), Arc::clone(&db))
                        }),
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
    rpc_url: Arc<String>,
    db: Arc<DB>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let https = HttpsConnector::new();
    let client = Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);
    let request_received = request.collect().await.unwrap().to_bytes();

    let request_hash = blake3::hash(&request_received).to_string();

    if let Some(data) = db.get(&request_hash).unwrap() {
        return Ok(Response::new(Full::new(Bytes::from(data))));
    }

    let rpc_quest = hyper::Request::builder()
        .method("POST")
        .uri(rpc_url.as_str())
        .header("Content-Type", "application/json")
        .body(Full::new(request_received.clone()))
        .unwrap();

    let rpc_response = client
        .request(rpc_quest)
        .await
        .unwrap()
        .collect()
        .await
        .unwrap()
        .to_bytes();

    let rpc_response_string = String::from_utf8_lossy(&rpc_response);

    // Avoid caching errors
    if !rpc_response_string.contains(r#""error":{"code":-"#) {
        db.put(request_hash, rpc_response_string.to_string())
            .unwrap();
    }

    Ok(Response::new(Full::new(rpc_response)))
}
