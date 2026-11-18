use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::timeouts::GetResultTimeouts;

/// HeyTap-specific data (`metadata`).
#[derive(Debug, Clone, Serialize)]
pub struct HeyTapMetadata {
    /// Subdomain of the HeyTap API server, without the `https://` prefix. Must be specified
    /// when it differs from the default server.
    #[serde(rename = "apiServer", skip_serializing_if = "Option::is_none")]
    pub api_server: Option<String>,
}

/// HeyTap captcha (`CustomTask`, class `heytap`). The solution token is in
/// `solution.data.token`.
#[derive(Debug, Clone, Serialize)]
pub struct HeyTapRequest {
    /// The address of the main page where the captcha is located.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The siteKey value found on the page (the `ak` parameter of the HeyTap API URL).
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    /// Optional challenge data; sent only when `api_server` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HeyTapMetadata>,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl HeyTapRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            metadata: None,
            user_agent: None,
        }
    }

    /// Sets `metadata.apiServer` (domain only, without `https://`).
    pub fn with_api_server(mut self, value: impl Into<String>) -> Self {
        self.metadata = Some(HeyTapMetadata {
            api_server: Some(value.into()),
        });
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for HeyTapRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("heytap");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 3.0, 180.0)
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
    use serde_json::json;

    #[test]
    fn minimal_task() {
        let task = HeyTapRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "heytap",
                "websiteURL": "https://example.com", "websiteKey": "KEY"
            })
        );
    }

    #[test]
    fn full_task() {
        let task = HeyTapRequest::new("https://example.com", "KEY")
            .with_api_server("api.example.com")
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "heytap",
                "websiteURL": "https://example.com", "websiteKey": "KEY",
                "metadata": {"apiServer": "api.example.com"},
                "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = HeyTapRequest::new("", "KEY").to_task().unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = HeyTapRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
