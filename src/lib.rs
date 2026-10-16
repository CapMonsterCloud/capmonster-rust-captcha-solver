//! Async Rust client for the [CapMonster Cloud](https://capmonster.cloud) captcha-solving API.
//!
//! ```no_run
//! use capmonstercloud_client::{CapMonsterClient, ClientOptions};
//! use capmonstercloud_client::requests::RecaptchaV2Request;
//!
//! # async fn run() -> capmonstercloud_client::Result<()> {
//! let client = CapMonsterClient::new(ClientOptions::new("<API_KEY>"))?;
//! let request = RecaptchaV2Request::new(
//!     "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
//!     "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
//! );
//! let solution = client.solve_captcha(&request).await?;
//! println!("{}", solution["gRecaptchaResponse"]);
//! # Ok(()) }
//! ```

mod client;
mod error;
mod proxy;
pub mod requests;
mod timeouts;

pub use client::{
    CapMonsterClient, ClientOptions, TaskStatus, DEFAULT_SERVICE_URL, DEFAULT_SOFT_ID,
};
pub use error::{Error, Result};
pub use proxy::{ClientProxy, ProxyInfo, ProxyType};
pub use timeouts::GetResultTimeouts;
