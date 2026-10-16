use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Basilisk captcha (`CustomTask`, class `Basilisk`).
#[derive(Debug, Clone, Serialize)]
pub struct BasiliskCustomTaskRequest {
    /// Address of the main page where the captcha is solved.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Site key: the `data-sitekey` attribute of the captcha container, or the `site_key` field of
    /// the POST payload to `https://basiliskcaptcha.com/challenge/check-site`.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl BasiliskCustomTaskRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
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

impl CaptchaRequest for BasiliskCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("Basilisk");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 10.0, 3.0, 100.0)
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
        let task = BasiliskCustomTaskRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "CustomTask", "class": "Basilisk", "websiteURL": "https://example.com", "websiteKey": "KEY"})
        );
    }

    #[test]
    fn full_task() {
        let task = BasiliskCustomTaskRequest::new("https://example.com", "KEY")
            .with_proxy(ProxyInfo::new(ProxyType::Https, "h.example", 3128).with_auth("u", "p"))
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "Basilisk", "websiteURL": "https://example.com", "websiteKey": "KEY",
                "proxyType": "https", "proxyAddress": "h.example", "proxyPort": 3128,
                "proxyLogin": "u", "proxyPassword": "p", "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = BasiliskCustomTaskRequest::new("", "KEY")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = BasiliskCustomTaskRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
