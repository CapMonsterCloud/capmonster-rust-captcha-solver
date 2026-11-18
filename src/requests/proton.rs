use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::{Error, Result};
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Proton Captcha challenge data (`metadata`).
#[derive(Debug, Clone, Serialize)]
pub struct ProtonMetadata {
    /// Token from the site's response.
    #[serde(rename = "HumanVerificationToken")]
    pub human_verification_token: String,
    /// Action on the site: `login` or `signup`.
    #[serde(rename = "pageAction")]
    pub page_action: String,
}

/// Proton Captcha (`CustomTask`, class `proton`). Your own proxy with login and password is
/// required. The solution token is in `solution.data.tokens`.
#[derive(Debug, Clone, Serialize)]
pub struct ProtonRequest {
    /// The address of the main page where the captcha is located.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    pub metadata: ProtonMetadata,
    /// Required proxy (non-transparent, not local).
    #[serde(flatten)]
    pub proxy: ProxyInfo,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl ProtonRequest {
    /// `page_action` is `login` or `signup`.
    pub fn new(
        website_url: impl Into<String>,
        human_verification_token: impl Into<String>,
        page_action: impl Into<String>,
        proxy: ProxyInfo,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            metadata: ProtonMetadata {
                human_verification_token: human_verification_token.into(),
                page_action: page_action.into(),
            },
            proxy,
            user_agent: None,
        }
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for ProtonRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("proton");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 3.0, 180.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty(
            "metadata.HumanVerificationToken",
            &self.metadata.human_verification_token,
        )?;
        if self.metadata.page_action != "login" && self.metadata.page_action != "signup" {
            return Err(Error::Validation(
                "metadata.pageAction must be \"login\" or \"signup\"".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::ProxyType;
    use serde_json::json;

    fn proxy() -> ProxyInfo {
        ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080).with_auth("u", "p")
    }

    #[test]
    fn minimal_task() {
        let task = ProtonRequest::new("https://example.com", "TOKEN", "login", proxy())
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "proton", "websiteURL": "https://example.com",
                "metadata": {"HumanVerificationToken": "TOKEN", "pageAction": "login"},
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                "proxyLogin": "u", "proxyPassword": "p"
            })
        );
    }

    #[test]
    fn full_task() {
        let task = ProtonRequest::new("https://example.com", "TOKEN", "signup", proxy())
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "proton", "websiteURL": "https://example.com",
                "metadata": {"HumanVerificationToken": "TOKEN", "pageAction": "signup"},
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                "proxyLogin": "u", "proxyPassword": "p", "userAgent": "UA"
            })
        );
    }

    #[test]
    fn invalid_page_action_is_rejected() {
        let err = ProtonRequest::new("https://example.com", "TOKEN", "logout", proxy())
            .to_task()
            .unwrap_err();
        assert!(matches!(err, Error::Validation(ref m) if m.contains("metadata.pageAction")));
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = ProtonRequest::new("", "TOKEN", "login", proxy())
            .to_task()
            .unwrap_err();
        assert!(matches!(err, Error::Validation(ref m) if m.contains("websiteURL")));
        let err = ProtonRequest::new("https://example.com", "", "login", proxy())
            .to_task()
            .unwrap_err();
        assert!(
            matches!(err, Error::Validation(ref m) if m.contains("metadata.HumanVerificationToken"))
        );
    }
}
