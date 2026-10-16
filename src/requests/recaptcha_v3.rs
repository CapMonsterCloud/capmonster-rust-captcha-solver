use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::{Error, Result};
use crate::timeouts::GetResultTimeouts;

/// Checks `minScore`: it must be within `0.1..=0.9`.
pub(crate) fn check_min_score(min_score: Option<f64>) -> Result<()> {
    match min_score {
        Some(v) if !(0.1..=0.9).contains(&v) => Err(Error::Validation(format!(
            "min_score must be between 0.1 and 0.9, got {v}"
        ))),
        _ => Ok(()),
    }
}

/// reCAPTCHA v3 (`RecaptchaV3TaskProxyless`). Proxies are not supported.
#[derive(Debug, Clone, Serialize)]
pub struct RecaptchaV3ProxylessRequest {
    /// Address of a webpage with Google reCAPTCHA.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The reCAPTCHA v3 site key (`...api.js?render=THIS_ONE`).
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    /// Minimum score of the token, `0.1..=0.9`.
    #[serde(rename = "minScore", skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f64>,
    /// Widget action value, default `verify`.
    #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
    pub page_action: Option<String>,
    /// Solve as reCAPTCHA v3 Enterprise (same as `RecaptchaV3EnterpriseRequest`).
    #[serde(rename = "isEnterprise", skip_serializing_if = "Option::is_none")]
    pub is_enterprise: Option<bool>,
}

impl RecaptchaV3ProxylessRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            min_score: None,
            page_action: None,
            is_enterprise: None,
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

    pub fn with_is_enterprise(mut self, value: bool) -> Self {
        self.is_enterprise = Some(value);
        self
    }
}

impl CaptchaRequest for RecaptchaV3ProxylessRequest {
    const TASK_TYPE: &'static str = "RecaptchaV3TaskProxyless";

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
    use serde_json::json;

    #[test]
    fn minimal_task() {
        let task = RecaptchaV3ProxylessRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "RecaptchaV3TaskProxyless", "websiteURL": "https://example.com", "websiteKey": "KEY"})
        );
    }

    #[test]
    fn full_task() {
        let task = RecaptchaV3ProxylessRequest::new("https://example.com", "KEY")
            .with_min_score(0.7)
            .with_page_action("login")
            .with_is_enterprise(true)
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "RecaptchaV3TaskProxyless", "websiteURL": "https://example.com",
                   "websiteKey": "KEY", "minScore": 0.7, "pageAction": "login", "isEnterprise": true})
        );
    }

    #[test]
    fn min_score_bounds() {
        for bad in [0.0, 0.05, 0.95, 1.0, f64::NAN] {
            let err = RecaptchaV3ProxylessRequest::new("u", "k")
                .with_min_score(bad)
                .to_task()
                .unwrap_err();
            assert!(matches!(err, Error::Validation(_)), "{bad}");
        }
        for ok in [0.1, 0.5, 0.9] {
            assert!(RecaptchaV3ProxylessRequest::new("u", "k")
                .with_min_score(ok)
                .to_task()
                .is_ok());
        }
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        for (url, key, field) in [
            ("", "KEY", "websiteURL"),
            ("https://example.com", "", "websiteKey"),
        ] {
            let err = RecaptchaV3ProxylessRequest::new(url, key)
                .to_task()
                .unwrap_err();
            assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains(field)));
        }
    }
}
