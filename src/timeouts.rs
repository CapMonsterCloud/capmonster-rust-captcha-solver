//! Polling schedule of `getTaskResult` for one task type.

use std::time::Duration;

/// How long to wait before the first poll, how often to poll, and when to give up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GetResultTimeouts {
    /// Pause between `createTask` and the first `getTaskResult`.
    pub first_request_delay: Duration,
    /// Same as above for the "no cache" mode. Kept for parity with the other SDKs; currently unused.
    pub first_request_no_cache_delay: Duration,
    /// Pause between two consecutive `getTaskResult` calls.
    pub requests_interval: Duration,
    /// Total time after which `Error::Timeout` is returned.
    pub timeout: Duration,
}

impl GetResultTimeouts {
    pub fn new(
        first_request_delay: Duration,
        first_request_no_cache_delay: Duration,
        requests_interval: Duration,
        timeout: Duration,
    ) -> Self {
        Self {
            first_request_delay,
            first_request_no_cache_delay,
            requests_interval,
            timeout,
        }
    }

    /// Same as [`Self::new`] with the values given in seconds. Only used for the built-in
    /// per-type constants, which are always finite and non-negative.
    pub(crate) fn secs(
        first_delay: f64,
        first_no_cache_delay: f64,
        interval: f64,
        timeout: f64,
    ) -> Self {
        Self::new(
            Duration::from_secs_f64(first_delay),
            Duration::from_secs_f64(first_no_cache_delay),
            Duration::from_secs_f64(interval),
            Duration::from_secs_f64(timeout),
        )
    }
}
