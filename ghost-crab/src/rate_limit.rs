use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::{Instant, Sleep};
use tower::Layer;
use tower::Service;

#[derive(Debug, Copy, Clone)]
pub struct Rate {
    limit: u64,
    period: Duration,
}

/// Enforces a rate limit on the number of requests the underlying
/// service can handle over a period of time.
#[derive(Debug, Clone)]
pub struct RateLimitLayer {
    rate: Rate,
}

impl RateLimitLayer {
    /// Create new rate limit layer.
    pub fn new(limit: u64, period: Duration) -> Self {
        let rate = Rate { limit, period };
        RateLimitLayer { rate }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimit<S>;

    fn layer(&self, service: S) -> Self::Service {
        RateLimit::new(service, self.rate)
    }
}

/// Enforces a rate limit on the number of requests the underlying
/// service can handle over a period of time.
#[derive(Debug, Clone)]
pub struct RateLimit<T> {
    inner: T,
    rate: Rate,
    state: Arc<Mutex<State>>,
}

#[derive(Debug)]
struct State {
    until: Instant,
    reserved: u64,
    timer: Pin<Box<Sleep>>,
}

impl<T> RateLimit<T> {
    /// Create a new rate limiter
    pub fn new(inner: T, rate: Rate) -> Self {
        let until = Instant::now() + rate.period;

        let state = Arc::new(Mutex::new(State {
            until,
            reserved: rate.limit,
            timer: Box::pin(tokio::time::sleep_until(until)),
        }));

        RateLimit { inner, rate, state }
    }
}

impl<S, Request> Service<Request> for RateLimit<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let now = Instant::now();
        let mut state = self.state.lock().unwrap();

        if now >= state.until {
            state.until = now + self.rate.period;
            state.reserved = 0;
            state.timer.as_mut().reset(now + self.rate.period);
        }

        if state.reserved >= self.rate.limit {
            ctx.waker().wake_by_ref();
            let _ = state.timer.as_mut().poll(ctx);
            return Poll::Pending;
        }

        match self.inner.poll_ready(ctx) {
            Poll::Ready(value) => {
                state.reserved += 1;
                Poll::Ready(value)
            }
            Poll::Pending => {
                ctx.waker().wake_by_ref();
                let _ = state.timer.as_mut().poll(ctx);
                Poll::Pending
            }
        }
    }

    fn call(&mut self, request: Request) -> Self::Future {
        self.inner.call(request)
    }
}
