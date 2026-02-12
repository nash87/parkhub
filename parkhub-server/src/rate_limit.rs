//! Rate Limiting

use governor::{clock::DefaultClock, middleware::NoOpMiddleware, Quota, RateLimiter};
use std::{num::NonZeroU32, sync::Arc};

/// Per-IP rate limiter
pub mod per_ip {
    use super::*;
    use governor::state::keyed::DashMapStateStore;
    use std::net::IpAddr;

    pub type IpRateLimiter =
        RateLimiter<IpAddr, DashMapStateStore<IpAddr>, DefaultClock, NoOpMiddleware>;

    pub fn create_ip_rate_limiter(requests_per_minute: u32) -> Arc<IpRateLimiter> {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap());
        Arc::new(RateLimiter::dashmap(quota))
    }
}

/// Specific rate limiters for different endpoints
#[allow(dead_code)]
pub struct EndpointRateLimiters {
    pub login: Arc<per_ip::IpRateLimiter>,
    pub register: Arc<per_ip::IpRateLimiter>,
}

impl EndpointRateLimiters {
    pub fn new() -> Self {
        Self {
            login: per_ip::create_ip_rate_limiter(5),
            register: per_ip::create_ip_rate_limiter(3),
        }
    }
}

impl Default for EndpointRateLimiters {
    fn default() -> Self {
        Self::new()
    }
}
