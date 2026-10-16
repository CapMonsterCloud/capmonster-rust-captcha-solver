use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Friendly Captcha parameters.
#[derive(Debug, Clone, Serialize)]
pub struct FriendlyMetadata {
    /// URL of the JS file, used to detect the captcha version.
    /// V1: `https://cdn.jsdelivr.net/npm/friendly-challenge@X.Y.Z/widget.module.min.js`
    /// (`X.Y.Z` is the client version from the `x-frc-client` header).
    /// V2: URL of the `site.min.js` file loaded on the page, e.g.
    /// `https://cdn.jsdelivr.net/npm/@friendlycaptcha/sdk@X.Y.Z/site.min.js`.
    #[serde(rename = "apiGetLib")]
    pub api_get_lib: String,
}

impl FriendlyMetadata {
    pub fn new(api_get_lib: impl Into<String>) -> Self {
        Self {
            api_get_lib: api_get_lib.into(),
        }
    }
}

/// Friendly Captcha (`CustomTask`, class `friendly`).
#[derive(Debug, Clone, Serialize)]
pub struct FriendlyCustomTaskRequest {
    /// Full URL of the page with the captcha.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Friendly Captcha site key.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    pub metadata: FriendlyMetadata,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl FriendlyCustomTaskRequest {
    pub fn new(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        metadata: FriendlyMetadata,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            metadata,
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

impl CaptchaRequest for FriendlyCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("friendly");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 3.0, 180.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("websiteKey", &self.website_key)?;
        require_non_empty("metadata.apiGetLib", &self.metadata.api_get_lib)?;
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
        let task = FriendlyCustomTaskRequest::new(
            "https://example.com",
            "FC123",
            FriendlyMetadata::new("https://x/site.min.js"),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "friendly", "websiteURL": "https://example.com", "websiteKey": "FC123",
                "metadata": {"apiGetLib": "https://x/site.min.js"}
            })
        );
    }

    #[test]
    fn full_task() {
        let task = FriendlyCustomTaskRequest::new(
            "https://example.com",
            "FC123",
            FriendlyMetadata::new("L"),
        )
        .with_proxy(ProxyInfo::new(ProxyType::Socks4, "1.2.3.4", 1080).with_auth("u", "p"))
        .with_user_agent("UA")
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "friendly", "websiteURL": "https://example.com", "websiteKey": "FC123",
                "metadata": {"apiGetLib": "L"},
                "proxyType": "socks4", "proxyAddress": "1.2.3.4", "proxyPort": 1080,
                "proxyLogin": "u", "proxyPassword": "p", "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = FriendlyCustomTaskRequest::new("", "K", FriendlyMetadata::new("L"))
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err =
            FriendlyCustomTaskRequest::new("https://example.com", "", FriendlyMetadata::new("L"))
                .to_task()
                .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
        let err =
            FriendlyCustomTaskRequest::new("https://example.com", "K", FriendlyMetadata::new(""))
                .to_task()
                .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.apiGetLib"))
        );
    }
}
