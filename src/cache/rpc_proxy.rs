use crate::db::{add_cache, establish_connection, get_cache, AddCache};
use diesel::PgConnection;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

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
use tokio::net::TcpListener;

pub struct RpcWithCache {
    connection: Arc<Mutex<PgConnection>>,
    rpc_url: Arc<String>,
    port: u16,
}

impl RpcWithCache {
    pub fn new(database_url: String, rpc_url: String, port: u16) -> Self {
        let connection = Arc::new(Mutex::new(establish_connection(database_url)));

        Self {
            rpc_url: Arc::new(rpc_url),
            connection,
            port,
        }
    }

    pub async fn run(&self) {
        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();
        let listener = TcpListener::bind(addr).await.unwrap();

        loop {
            let (tcp, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(tcp);

            let rpc_url = Arc::clone(&self.rpc_url);
            let connection = Arc::clone(&self.connection);

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .timer(TokioTimer::new())
                    .serve_connection(
                        io,
                        service_fn(|request| {
                            handler(request, Arc::clone(&rpc_url), Arc::clone(&connection))
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

pub async fn handler(
    request: Request<hyper::body::Incoming>,
    rpc_url: Arc<String>,
    connection: Arc<Mutex<PgConnection>>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let https = HttpsConnector::new();
    let client = Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);
    let request_received = request.collect().await.unwrap().to_bytes();

    let rpc_url_bytes = Bytes::from(rpc_url.to_string());
    let request_hash =
        blake3::hash(&[request_received.clone(), rpc_url_bytes].concat()).to_string();

    {
        let mut conn = connection.lock().await;

        if let Some(data) = get_cache(&mut conn, &request_hash) {
            return Ok(Response::new(Full::new(Bytes::from(data))));
        }

        drop(conn);
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

    let mut conn = connection.lock().await;
    let rpc_response_string = String::from_utf8_lossy(&rpc_response);

    // Avoid caching errors
    if !rpc_response_string.contains(r#""error":{"code":-"#) {
        add_cache(
            &mut conn,
            AddCache {
                id: request_hash,
                data: rpc_response_string.to_string(),
            },
        );
    }

    Ok(Response::new(Full::new(rpc_response)))
}
