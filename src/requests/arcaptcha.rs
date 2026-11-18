use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::timeouts::GetResultTimeouts;

/// ARCaptcha (`ArCaptchaTask`). The solution token is in `solution.token`.
#[derive(Debug, Clone, Serialize)]
pub struct ArCaptchaRequest {
    /// The address of the main page where the captcha is located.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The siteKey value found on the page.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    /// `true` if the captcha is invisible (has a hidden verification field).
    #[serde(rename = "isInvisible", skip_serializing_if = "Option::is_none")]
    pub is_invisible: Option<bool>,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl ArCaptchaRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            is_invisible: None,
            user_agent: None,
        }
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

impl CaptchaRequest for ArCaptchaRequest {
    const TASK_TYPE: &'static str = "ArCaptchaTask";

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
        let task = ArCaptchaRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "ArCaptchaTask",
                "websiteURL": "https://example.com", "websiteKey": "KEY"
            })
        );
    }

    #[test]
    fn full_task() {
        let task = ArCaptchaRequest::new("https://example.com", "KEY")
            .with_is_invisible(false)
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "ArCaptchaTask",
                "websiteURL": "https://example.com", "websiteKey": "KEY",
                "isInvisible": false, "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = ArCaptchaRequest::new("", "KEY").to_task().unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = ArCaptchaRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
