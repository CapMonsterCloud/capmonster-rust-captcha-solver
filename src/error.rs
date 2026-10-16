//! Error type returned by every fallible operation of the SDK.

use std::time::Duration;

/// Convenient alias: `Result<T>` == `std::result::Result<T, capmonstercloud_client::Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong while talking to CapMonster Cloud.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request object is inconsistent (missing/forbidden field, value out of range...).
    /// Raised locally, before anything is sent to the network.
    #[error("invalid request: {0}")]
    Validation(String),

    /// Network / TLS / timeout error from the HTTP layer.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// The server answered with an unexpected HTTP status code.
    #[error("unexpected HTTP status {status} from {method}")]
    Status { method: &'static str, status: u16 },

    /// `createTask` returned `errorId != 0`.
    #[error("cannot create task: [{code}] {description}")]
    CreateTask { code: String, description: String },

    /// `getTaskResult` returned `errorId != 0`.
    #[error("cannot get task result: [{code}] {description}")]
    GetResult { code: String, description: String },

    /// `getBalance` returned `errorId != 0`.
    #[error("cannot get balance: [{code}] {description}")]
    GetBalance { code: String, description: String },

    /// The server's JSON was missing a field we need (e.g. `taskId`).
    #[error("malformed API response: {0}")]
    MalformedResponse(String),

    /// The solution did not arrive within the maximum waiting interval of this task type.
    #[error("failed to get a solution within {0:?}")]
    Timeout(Duration),

    /// The proxy type given in `ClientOptions::client_proxy` is not supported (only http/https).
    #[error("unsupported client proxy type: {0}")]
    UnsupportedProxyType(String),

    /// (De)serialization failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
