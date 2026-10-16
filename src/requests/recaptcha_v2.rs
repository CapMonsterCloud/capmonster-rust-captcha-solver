use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// reCAPTCHA v2 (`RecaptchaV2Task`). Without `proxy` the task is solved from CapMonster's own IPs.
#[derive(Debug, Clone, Serialize)]
pub struct RecaptchaV2Request {
    /// URL of the page with the captcha.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// reCAPTCHA site key.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// One-time `data-s` token of some custom reCAPTCHA v2 implementations.
    #[serde(
        rename = "recaptchaDataSValue",
        skip_serializing_if = "Option::is_none"
    )]
    pub data_s_value: Option<String>,
    /// Current browser User-Agent (Windows).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Cookies as `"name1=value1; name2=value2"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<String>,
    #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
    pub is_invisible: Option<bool>,
}

impl RecaptchaV2Request {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            proxy: None,
            data_s_value: None,
            user_agent: None,
            cookies: None,
            is_invisible: None,
        }
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_data_s_value(mut self, value: impl Into<String>) -> Self {
        self.data_s_value = Some(value.into());
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn with_cookies(mut self, value: impl Into<String>) -> Self {
        self.cookies = Some(value.into());
        self
    }

    pub fn with_is_invisible(mut self, value: bool) -> Self {
        self.is_invisible = Some(value);
        self
    }
}

impl CaptchaRequest for RecaptchaV2Request {
    const TASK_TYPE: &'static str = "RecaptchaV2Task";

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
    use crate::proxy::{ProxyInfo, ProxyType};
    use serde_json::json;

    #[test]
    fn minimal_task() {
        let task = RecaptchaV2Request::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "RecaptchaV2Task", "websiteURL": "https://example.com", "websiteKey": "KEY"})
        );
    }

    #[test]
    fn full_task_flattens_proxy_and_renames_fields() {
        let task = RecaptchaV2Request::new("https://example.com", "KEY")
            .with_proxy(ProxyInfo::new(ProxyType::Socks5, "1.2.3.4", 1080).with_auth("u", "p"))
            .with_data_s_value("S")
            .with_user_agent("UA")
            .with_cookies("a=b")
            .with_is_invisible(true)
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "RecaptchaV2Task", "websiteURL": "https://example.com", "websiteKey": "KEY",
                "proxyType": "socks5", "proxyAddress": "1.2.3.4", "proxyPort": 1080,
                "proxyLogin": "u", "proxyPassword": "p",
                "recaptchaDataSValue": "S", "userAgent": "UA", "cookies": "a=b", "isInvisible": true
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = RecaptchaV2Request::new("", "KEY").to_task().unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = RecaptchaV2Request::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
