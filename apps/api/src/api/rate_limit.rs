use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct RateLimiter {
    state: Arc<Mutex<HashMap<String, (f64, Instant)>>>,
    max_tokens: f64,
    refill_rate_per_sec: f64,
    window: Duration,
    trust_proxy_headers: bool,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration, trust_proxy_headers: bool) -> Self {
        let max_tokens = max_requests as f64;
        let refill_rate_per_sec = max_tokens / window.as_secs_f64();
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            max_tokens,
            refill_rate_per_sec,
            window,
            trust_proxy_headers,
        }
    }

    /// Checks if a request for `key` is permitted. Returns Ok(()) if permitted, or Err(retry_after_secs)
    pub fn check(&self, key: &str) -> Result<(), u64> {
        let mut map = self.state.lock().unwrap();
        let now = Instant::now();

        // Evict stale entries when tracking table grows large
        if map.len() > 10_000 {
            map.retain(|_, (_, last_refill)| now.duration_since(*last_refill) < self.window * 2);
        }

        let entry = map.entry(key.to_string()).or_insert((self.max_tokens, now));
        let elapsed = now.duration_since(entry.1).as_secs_f64();
        entry.0 = (entry.0 + elapsed * self.refill_rate_per_sec).min(self.max_tokens);
        entry.1 = now;

        if entry.0 >= 1.0 {
            entry.0 -= 1.0;
            Ok(())
        } else {
            let missing = 1.0 - entry.0;
            let retry_after = (missing / self.refill_rate_per_sec).ceil() as u64;
            Err(retry_after.max(1))
        }
    }
}

pub fn client_ip(headers: &HeaderMap, peer: Option<SocketAddr>, trust_proxy: bool) -> String {
    if trust_proxy
        && let Some(ip) = headers.get("x-real-ip").and_then(|v| v.to_str().ok())
        && let Ok(ip) = ip.parse::<std::net::IpAddr>()
    {
        return ip.to_string();
    }
    peer.map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0);
    let client_ip = client_ip(req.headers(), peer, limiter.trust_proxy_headers);
    match limiter.check(&client_ip) {
        Ok(()) => next.run(req).await,
        Err(retry_after) => {
            let body = json!({
                "code": 429,
                "message": "Too many requests, please try again later"
            });
            (
                StatusCode::TOO_MANY_REQUESTS,
                [
                    (header::RETRY_AFTER, retry_after.to_string()),
                    (header::CONTENT_TYPE, "application/json".to_string()),
                ],
                serde_json::to_string(&body).unwrap_or_default(),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_burst_then_blocks() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60), false);
        assert!(limiter.check("ip-1").is_ok());
        assert!(limiter.check("ip-1").is_ok());
        assert!(limiter.check("ip-1").is_ok());
        let blocked = limiter.check("ip-1");
        assert!(blocked.is_err());
        assert!(blocked.unwrap_err() > 0);

        // Different IP is unaffected
        assert!(limiter.check("ip-2").is_ok());
    }

    #[test]
    fn test_client_ip_extraction() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "203.0.113.195, 70.41.3.18".parse().unwrap(),
        );
        let peer = "198.51.100.2:443".parse().unwrap();
        assert_eq!(client_ip(&headers, Some(peer), false), "198.51.100.2");

        let mut headers_real = HeaderMap::new();
        headers_real.insert("x-real-ip", "198.51.100.1".parse().unwrap());
        assert_eq!(client_ip(&headers_real, Some(peer), true), "198.51.100.1");
        assert_eq!(client_ip(&headers_real, Some(peer), false), "198.51.100.2");

        let empty = HeaderMap::new();
        assert_eq!(client_ip(&empty, None, false), "127.0.0.1");
    }
}
