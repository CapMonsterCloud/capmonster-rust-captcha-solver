use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Binance custom captcha (`BinanceTask`). Use only to log in with your own account.
///
/// `validate_id` is dynamic: extract it right before creating the task.
#[derive(Debug, Clone, Serialize)]
pub struct BinanceTaskRequest {
    /// Address of the main page where the captcha is solved.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Identifies the section of the site: value of `bizId`, `bizType` or `bizCode` taken from
    /// the traffic, e.g. `login`.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    /// Dynamic key: value of `validateId`, `securityId` or `securityCheckResponseValidateId`
    /// taken from the traffic.
    #[serde(rename = "validateId")]
    pub validate_id: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl BinanceTaskRequest {
    pub fn new(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        validate_id: impl Into<String>,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            validate_id: validate_id.into(),
            proxy: None,
            user_agent: None,
        }
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for BinanceTaskRequest {
    const TASK_TYPE: &'static str = "BinanceTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 1.0, 20.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("websiteKey", &self.website_key)?;
        require_non_empty("validateId", &self.validate_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::ProxyType;
    use serde_json::json;

    #[test]
    fn minimal_task() {
        let task = BinanceTaskRequest::new("https://example.com", "login", "vid")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "BinanceTask", "websiteURL": "https://example.com",
                   "websiteKey": "login", "validateId": "vid"})
        );
    }

    #[test]
    fn full_task() {
        let task = BinanceTaskRequest::new("https://example.com", "login", "vid")
            .with_proxy(ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080).with_auth("u", "p"))
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "BinanceTask", "websiteURL": "https://example.com",
                   "websiteKey": "login", "validateId": "vid", "userAgent": "UA",
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                   "proxyLogin": "u", "proxyPassword": "p"})
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = BinanceTaskRequest::new("", "k", "v").to_task().unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = BinanceTaskRequest::new("https://example.com", "", "v")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
        let err = BinanceTaskRequest::new("https://example.com", "k", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("validateId")));
    }
}
