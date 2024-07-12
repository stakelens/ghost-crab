use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Response;
use hyper_util::rt::TokioIo;
use hyper_util::rt::TokioTimer;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn start(&self) {
        let port = self.port;

        tokio::spawn(async move {
            let addr: SocketAddr = ([127, 0, 0, 1], port).into();
            let listener = TcpListener::bind(addr).await.unwrap();

            loop {
                let (tcp, _) = listener.accept().await.unwrap();
                let io = TokioIo::new(tcp);

                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .timer(TokioTimer::new())
                        .serve_connection(io, service_fn(|_| handler()))
                        .await
                    {
                        println!("Error serving connection: {:?}", err);
                    }
                });
            }
        });
    }
}

async fn handler() -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello world"))))
}
