use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// MTCaptcha (`MTCaptchaTask`).
#[derive(Debug, Clone, Serialize)]
pub struct MtCaptchaRequest {
    /// URL of the page where the captcha is solved.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// MTCaptcha key, passed on the page as `sk`; typically `MTPublic-abCDEFJAB`.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// The `act` parameter. Send it only if it differs from the default `%24`.
    #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
    pub page_action: Option<String>,
    /// `true` if the captcha is invisible (has a hidden confirmation field).
    #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
    pub is_invisible: Option<bool>,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl MtCaptchaRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            proxy: None,
            page_action: None,
            is_invisible: None,
            user_agent: None,
        }
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_page_action(mut self, value: impl Into<String>) -> Self {
        self.page_action = Some(value.into());
        self
    }

    pub fn with_is_invisible(mut self, value: bool) -> Self {
        self.is_invisible = Some(value);
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for MtCaptchaRequest {
    const TASK_TYPE: &'static str = "MTCaptchaTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 10.0, 3.0, 180.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("websiteKey", &self.website_key)?;
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
        let task = MtCaptchaRequest::new("https://example.com", "MTPublic-x")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "MTCaptchaTask", "websiteURL": "https://example.com",
                   "websiteKey": "MTPublic-x"})
        );
    }

    #[test]
    fn full_task() {
        let task = MtCaptchaRequest::new("https://example.com", "MTPublic-x")
            .with_proxy(ProxyInfo::new(ProxyType::Socks5, "1.2.3.4", 1080))
            .with_page_action("login")
            .with_is_invisible(true)
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "MTCaptchaTask", "websiteURL": "https://example.com",
                   "websiteKey": "MTPublic-x", "pageAction": "login", "isInvisible": true,
                   "userAgent": "UA", "proxyType": "socks5", "proxyAddress": "1.2.3.4",
                   "proxyPort": 1080})
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = MtCaptchaRequest::new("", "KEY").to_task().unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = MtCaptchaRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
