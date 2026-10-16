use serde::Serialize;

use super::recaptcha_v3::check_min_score;
use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::timeouts::GetResultTimeouts;

/// reCAPTCHA v3 Enterprise (`RecaptchaV3EnterpriseTask`). Proxies are not supported.
#[derive(Debug, Clone, Serialize)]
pub struct RecaptchaV3EnterpriseRequest {
    /// Address of a webpage with Google reCAPTCHA.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The site key (`...enterprise.js?render=THIS_ONE`).
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    /// Minimum score of the token, `0.1..=0.9`.
    #[serde(rename = "minScore", skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f64>,
    /// Value of the `action` parameter, if it differs from the default `verify`.
    #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
    pub page_action: Option<String>,
}

impl RecaptchaV3EnterpriseRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            min_score: None,
            page_action: None,
        }
    }

    /// Minimum score, must be in `0.1..=0.9` (checked by `validate()`).
    pub fn with_min_score(mut self, value: f64) -> Self {
        self.min_score = Some(value);
        self
    }

    pub fn with_page_action(mut self, value: impl Into<String>) -> Self {
        self.page_action = Some(value.into());
        self
    }
}

impl CaptchaRequest for RecaptchaV3EnterpriseRequest {
    const TASK_TYPE: &'static str = "RecaptchaV3EnterpriseTask";

    /// Same schedule as the Python client actually uses for this type (`getRecaptchaV3Timeouts`,
    /// 180 s), not the unused `getRecaptchaV3EnterpriseTimeouts` (150 s).
    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 10.0, 3.0, 180.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("websiteKey", &self.website_key)?;
        check_min_score(self.min_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use serde_json::json;

    #[test]
    fn minimal_task() {
        let task = RecaptchaV3EnterpriseRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "RecaptchaV3EnterpriseTask", "websiteURL": "https://example.com", "websiteKey": "KEY"})
        );
    }

    #[test]
    fn full_task() {
        let task = RecaptchaV3EnterpriseRequest::new("https://example.com", "KEY")
            .with_min_score(0.9)
            .with_page_action("login")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "RecaptchaV3EnterpriseTask", "websiteURL": "https://example.com",
                   "websiteKey": "KEY", "minScore": 0.9, "pageAction": "login"})
        );
    }

    #[test]
    fn min_score_out_of_range() {
        let err = RecaptchaV3EnterpriseRequest::new("u", "k")
            .with_min_score(0.95)
            .to_task()
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        for (url, key, field) in [
            ("", "KEY", "websiteURL"),
            ("https://example.com", "", "websiteKey"),
        ] {
            let err = RecaptchaV3EnterpriseRequest::new(url, key)
                .to_task()
                .unwrap_err();
            assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains(field)));
        }
    }
}
