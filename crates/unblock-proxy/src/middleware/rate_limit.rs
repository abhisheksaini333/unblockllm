//! Rate limiting middleware. In-memory (tower_governor) or Redis-backed (SEC-04) when REDIS_URL set.
//! RATE_LIMIT_PER_SECOND (default 1000); burst = 2 * per_second for in-memory.

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, Response, StatusCode};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::PeerIpKeyExtractor;
use tower_governor::GovernorLayer;

type GovernorLayerType = GovernorLayer<
    PeerIpKeyExtractor,
    governor::middleware::NoOpMiddleware<governor::clock::QuantaInstant>,
>;

const RL_PREFIX: &str = "unblock:rl:";

/// Per-second limit from env.
pub fn rate_limit_per_second() -> u64 {
    std::env::var("RATE_LIMIT_PER_SECOND")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1000)
}

/// Build in-memory GovernorLayer from env.
pub fn rate_limit_layer() -> Result<GovernorLayerType, Box<dyn std::error::Error + Send + Sync>> {
    let per_second = rate_limit_per_second();
    let burst = (per_second * 2).min(u32::MAX as u64) as u32;
    let period_ms = if per_second > 0 { 1000 / per_second } else { 1 };
    let mut builder = GovernorConfigBuilder::default()
        .const_per_millisecond(period_ms.max(1))
        .const_burst_size(burst.max(1));
    let config = builder.finish().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "rate limit config: invalid builder state",
        )
    })?;
    Ok(GovernorLayer {
        config: Arc::new(config),
    })
}

/// Redis-backed rate limit (SEC-04): shared across proxy instances. 1-second fixed window per IP.
#[derive(Clone)]
pub struct RedisRateLimitLayer {
    conn: Arc<tokio::sync::Mutex<redis::aio::ConnectionManager>>,
    per_second: u64,
}

impl RedisRateLimitLayer {
    pub fn new(conn: redis::aio::ConnectionManager, per_second: u64) -> Self {
        Self {
            conn: Arc::new(tokio::sync::Mutex::new(conn)),
            per_second: per_second.max(1),
        }
    }
}

impl<S> Layer<S> for RedisRateLimitLayer {
    type Service = RedisRateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RedisRateLimitService {
            inner,
            conn: Arc::clone(&self.conn),
            per_second: self.per_second,
        }
    }
}

#[derive(Clone)]
pub struct RedisRateLimitService<S> {
    inner: S,
    conn: Arc<tokio::sync::Mutex<redis::aio::ConnectionManager>>,
    per_second: u64,
}

impl<S, ReqBody> Service<Request<ReqBody>> for RedisRateLimitService<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let conn = Arc::clone(&self.conn);
        let per_second = self.per_second;
        let peer = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0);
        let mut inner = self.inner.clone();

        let fut = async move {
            let addr = match peer {
                Some(a) => a.to_string(),
                None => "unknown".to_string(),
            };
            let key = format!("{}{}", RL_PREFIX, addr.replace(':', "_"));
            let mut conn = conn.lock().await;
            let count: i64 = match redis::cmd("INCR").arg(&key).query_async(&mut *conn).await {
                Ok(c) => c,
                Err(_) => {
                    drop(conn);
                    return Ok(Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .body(Body::from("rate limit store unavailable"))
                        .expect("valid response"));
                }
            };
            if count == 1 {
                let _: Result<(), _> = redis::cmd("EXPIRE")
                    .arg(&key)
                    .arg(1)
                    .query_async(&mut *conn)
                    .await;
            }
            drop(conn);
            if count > per_second as i64 {
                return Ok(Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .body(Body::from("rate limit exceeded"))
                    .expect("valid response"));
            }
            inner.call(req).await
        };
        Box::pin(fut)
    }
}

/// Build Redis-backed rate limit layer when REDIS_URL is set.
pub async fn rate_limit_layer_redis(
    redis_url: &str,
) -> Result<RedisRateLimitLayer, Box<dyn std::error::Error + Send + Sync>> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_connection_manager().await?;
    let per_second = rate_limit_per_second();
    Ok(RedisRateLimitLayer::new(conn, per_second))
}
