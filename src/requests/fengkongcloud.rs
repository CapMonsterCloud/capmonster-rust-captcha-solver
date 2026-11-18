use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::timeouts::GetResultTimeouts;

/// FengkongCloud-specific data (`metadata`).
#[derive(Debug, Clone, Serialize)]
pub struct FengkongCloudMetadata {
    /// Link to `smcp.min.js`, e.g. `https://example.com/pr/vX.X.X/smcp.min.js`.
    #[serde(rename = "apiJsUrl", skip_serializing_if = "Option::is_none")]
    pub api_js_url: Option<String>,
}

/// FengkongCloud captcha (`CustomTask`, class `fengkongcloud`). The solution token is in
/// `solution.data.token`.
#[derive(Debug, Clone, Serialize)]
pub struct FengkongCloudRequest {
    /// The address of the main page where the captcha is located.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The `appid` value found on the page.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    /// Optional challenge data; sent only when `api_js_url` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FengkongCloudMetadata>,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl FengkongCloudRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            metadata: None,
            user_agent: None,
        }
    }

    /// Sets `metadata.apiJsUrl`, the link to `smcp.min.js`.
    pub fn with_api_js_url(mut self, value: impl Into<String>) -> Self {
        self.metadata = Some(FengkongCloudMetadata {
            api_js_url: Some(value.into()),
        });
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for FengkongCloudRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("fengkongcloud");

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
        let task = FengkongCloudRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "fengkongcloud",
                "websiteURL": "https://example.com", "websiteKey": "KEY"
            })
        );
    }

    #[test]
    fn full_task() {
        let task = FengkongCloudRequest::new("https://example.com", "KEY")
            .with_api_js_url("https://example.com/smcp.min.js")
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "fengkongcloud",
                "websiteURL": "https://example.com", "websiteKey": "KEY",
                "metadata": {"apiJsUrl": "https://example.com/smcp.min.js"},
                "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = FengkongCloudRequest::new("", "KEY").to_task().unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = FengkongCloudRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
