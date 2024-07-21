use blake3;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioIo;
use hyper_util::rt::TokioTimer;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use rocksdb::DB;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct RpcWithCache {
    rpc_url: Arc<String>,
    cache: Arc<DB>,
    port: u16,
}

#[derive(Debug)]
pub enum Error {
    DB(rocksdb::Error),
    CacheFileNotFound(std::io::Error),
}

type Result<T> = core::result::Result<T, Error>;

fn load_cache(network: &str) -> Result<DB> {
    let current_dir = std::env::current_dir().map_err(|e| Error::CacheFileNotFound(e))?;
    let cache_path = current_dir.join("cache").join(network);
    let db = DB::open_default(cache_path).map_err(|e| Error::DB(e))?;

    Ok(db)
}

impl RpcWithCache {
    pub fn new(network: &str, rpc_url: String, port: u16) -> Result<Self> {
        Ok(Self { rpc_url: Arc::new(rpc_url), cache: Arc::new(load_cache(network)?), port })
    }

    pub async fn run(&self) -> core::result::Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();
        let listener = TcpListener::bind(addr).await?;
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);

        loop {
            let (tcp, _) = listener.accept().await?;
            let io = TokioIo::new(tcp);
            let db = Arc::clone(&self.cache);
            let rpc_url = Arc::clone(&self.rpc_url);
            let client = client.clone();

            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .timer(TokioTimer::new())
                    .serve_connection(
                        io,
                        service_fn(|request| {
                            handler(request, Arc::clone(&rpc_url), Arc::clone(&db), client.clone())
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

fn divide_request_by_id(input: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    const ID_FIELD: &[u8; 5] = b"\"id\":";
    let id_field_index = input.windows(ID_FIELD.len()).position(|x| x == ID_FIELD)?;

    let value_start = id_field_index + ID_FIELD.len();
    let value_end = input[value_start..].iter().position(|&x| x == b',')?;

    Some((
        &input[..value_start],
        &input[value_start..value_start + value_end],
        &input[value_start + value_end..],
    ))
}

const INVALID_WORDS: &[&[u8]] =
    &[b"eth_blockNumber", b"earliest", b"latest", b"safe", b"finalized", b"pending"];

#[inline]
fn contains_invalid_word(input: &[u8]) -> bool {
    for search in INVALID_WORDS {
        if input.windows(search.len()).any(|x| &x == search) {
            return true;
        }
    }

    false
}

async fn handler(
    request: Request<hyper::body::Incoming>,
    rpc_url: Arc<String>,
    db: Arc<DB>,
    client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
) -> core::result::Result<Response<Full<Bytes>>, hyper::Error> {
    let request_received = request.collect().await?.to_bytes();

    if contains_invalid_word(&request_received) {
        let rpc_request = hyper::Request::builder()
            .method("POST")
            .uri(rpc_url.as_str())
            .header("Content-Type", "application/json")
            .body(Full::new(request_received))
            .unwrap();

        let rpc_response = client.request(rpc_request).await.unwrap().collect().await?.to_bytes();
        return Ok(Response::new(Full::new(rpc_response)));
    }

    // Sets the JSON RPC id to zero
    let (start, _value, end) = divide_request_by_id(&request_received).unwrap();
    let request_received = Bytes::from([start, b"0", end].concat());

    let request_hash = blake3::hash(&request_received).to_string();

    if let Ok(Some(value)) = db.get(&request_hash) {
        return Ok(Response::new(Full::new(Bytes::from(value))));
    }

    let rpc_request = hyper::Request::builder()
        .method("POST")
        .uri(rpc_url.as_str())
        .header("Content-Type", "application/json")
        .body(Full::new(request_received))
        .unwrap();

    let rpc_response = client.request(rpc_request).await.unwrap().collect().await?.to_bytes();
    let rpc_response_string = String::from_utf8_lossy(&rpc_response);

    // Avoid caching errors
    if !rpc_response_string.contains(r#""error":{"code":-"#) {
        if let Err(err) = db.put(request_hash, rpc_response_string.to_string()) {
            println!("WARNING: Error saving value to cache {err}");
        };
    }

    Ok(Response::new(Full::new(rpc_response)))
}
