use alloy::rpc::json_rpc::{
    Id, RequestPacket, Response, ResponsePacket, ResponsePayload, SerializedRequest,
};
use alloy::transports::{RpcError, TransportError, TransportErrorKind};
use rocksdb::DB;
use serde_json::value::RawValue;
use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

pub struct CacheLayer {
    db: Arc<DB>,
}

impl CacheLayer {
    pub fn new(db: DB) -> Self {
        Self { db: Arc::new(db) }
    }
}

impl<S> Layer<S> for CacheLayer {
    type Service = CacheService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheService { inner, db: Arc::clone(&self.db) }
    }
}

#[derive(Debug, Clone)]
pub struct CacheService<S> {
    inner: S,
    db: Arc<DB>,
}

impl<S> CacheService<S> {
    fn convert_to_response(
        &self,
        raw_response: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = Result<ResponsePacket, TransportError>> + Send>> {
        let raw_value = String::from_utf8(raw_response).unwrap();
        let raw_value = RawValue::from_string(raw_value).unwrap();

        let response_payload: ResponsePayload<Box<RawValue>, Box<RawValue>>;
        response_payload = ResponsePayload::Success(raw_value);

        let response_single = Response { id: Id::Number(0), payload: response_payload };

        let response_packet: ResponsePacket;
        response_packet = ResponsePacket::Single(response_single);

        let response: Result<ResponsePacket, RpcError<TransportErrorKind>>;
        response = Ok(response_packet);

        return Box::pin(async move {
            return response;
        });
    }
}

const INVALID_WORDS: &[&[u8]] = &[b"earliest", b"latest", b"safe", b"finalized", b"pending"];

#[inline]
fn contains_invalid_word(input: &[u8]) -> bool {
    for search in INVALID_WORDS {
        if input.windows(search.len()).any(|x| &x == search) {
            return true;
        }
    }

    false
}

fn cacheable_request(request: &SerializedRequest) -> bool {
    if !matches!(request.method(), "eth_getBlockByNumber" | "eth_getLogs" | "eth_call") {
        return false;
    }

    let raw_request = request.serialized().get();

    if contains_invalid_word(raw_request.as_bytes()) {
        return false;
    }

    return true;
}

impl<S> Service<RequestPacket> for CacheService<S>
where
    S: Service<RequestPacket, Response = ResponsePacket, Error = TransportError>,
    S::Future: Send + 'static,
    S::Response: Send + 'static + Debug,
    S::Error: Send + 'static + Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: RequestPacket) -> Self::Future {
        if let RequestPacket::Single(single) = &request {
            if cacheable_request(single) {
                let raw_request = single.serialized().get();

                // Set id to zero
                let id = single.id();
                let id_old = format!("\"id\":{id}");
                let id_new = "\"id\":0";
                let raw_request = raw_request.replace(&id_old, id_new);

                if let Ok(Some(raw_data)) = self.db.get(&raw_request) {
                    return self.convert_to_response(raw_data);
                }

                let db = Arc::clone(&self.db);
                let future = self.inner.call(request);

                return Box::pin(async move {
                    let response = future.await;

                    if let Ok(response) = &response {
                        if let ResponsePacket::Single(single) = response {
                            if let ResponsePayload::Success(payload) = &single.payload {
                                let raw_response = payload.get();
                                db.put(raw_request, raw_response).unwrap();
                            }
                        }
                    }

                    response
                });
            }
        }

        Box::pin(self.inner.call(request))
    }
}
