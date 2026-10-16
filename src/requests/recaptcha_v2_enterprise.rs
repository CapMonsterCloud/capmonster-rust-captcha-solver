use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Additional reCAPTCHA Enterprise parameters (`enterprisePayload` object).
///
/// Documented key: `s`, an additional parameter that may be passed to
/// `grecaptcha.enterprise.render` together with the `sitekey`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnterprisePayload {
    /// Value of the `s` parameter of `grecaptcha.enterprise.render`.
    pub s: String,
}

impl EnterprisePayload {
    pub fn new(s: impl Into<String>) -> Self {
        Self { s: s.into() }
    }
}

/// reCAPTCHA v2 Enterprise (`RecaptchaV2EnterpriseTask`).
///
/// Without `proxy` the task is solved from CapMonster's own IPs; a proxy is only needed when the
/// website does not accept the token.
#[derive(Debug, Clone, Serialize)]
pub struct RecaptchaV2EnterpriseRequest {
    /// Address of a webpage with Google reCAPTCHA Enterprise.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The reCAPTCHA site key on the target page.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    /// Proxy used by the worker (optional).
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Additional reCAPTCHA Enterprise parameters.
    #[serde(rename = "enterprisePayload", skip_serializing_if = "Option::is_none")]
    pub enterprise_payload: Option<EnterprisePayload>,
    /// Domain to load reCAPTCHA Enterprise from, e.g. `www.google.com` or `www.recaptcha.net`.
    #[serde(rename = "apiDomain", skip_serializing_if = "Option::is_none")]
    pub api_domain: Option<String>,
    /// Value of the `action` parameter, if it differs from the default `verify`.
    #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
    pub page_action: Option<String>,
    /// Browser User-Agent (use the value supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Cookies as `"name1=value1; name2=value2"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<String>,
}

impl RecaptchaV2EnterpriseRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            proxy: None,
            enterprise_payload: None,
            api_domain: None,
            page_action: None,
            user_agent: None,
            cookies: None,
        }
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_enterprise_payload(mut self, value: EnterprisePayload) -> Self {
        self.enterprise_payload = Some(value);
        self
    }

    /// Shortcut for `with_enterprise_payload(EnterprisePayload::new(s))`.
    pub fn with_enterprise_s(self, s: impl Into<String>) -> Self {
        self.with_enterprise_payload(EnterprisePayload::new(s))
    }

    pub fn with_api_domain(mut self, value: impl Into<String>) -> Self {
        self.api_domain = Some(value.into());
        self
    }

    pub fn with_page_action(mut self, value: impl Into<String>) -> Self {
        self.page_action = Some(value.into());
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
}

impl CaptchaRequest for RecaptchaV2EnterpriseRequest {
    const TASK_TYPE: &'static str = "RecaptchaV2EnterpriseTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 10.0, 3.0, 180.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("websiteKey", &self.website_key)?;
        if let Some(payload) = &self.enterprise_payload {
            require_non_empty("enterprisePayload.s", &payload.s)?;
        }
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
        let task = RecaptchaV2EnterpriseRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "RecaptchaV2EnterpriseTask", "websiteURL": "https://example.com", "websiteKey": "KEY"})
        );
    }

    #[test]
    fn full_task() {
        let task = RecaptchaV2EnterpriseRequest::new("https://example.com", "KEY")
            .with_proxy(ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080).with_auth("u", "p"))
            .with_enterprise_s("SVAL")
            .with_api_domain("www.recaptcha.net")
            .with_page_action("login")
            .with_user_agent("UA")
            .with_cookies("a=b")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "RecaptchaV2EnterpriseTask", "websiteURL": "https://example.com",
                "websiteKey": "KEY",
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                "proxyLogin": "u", "proxyPassword": "p",
                "enterprisePayload": {"s": "SVAL"}, "apiDomain": "www.recaptcha.net",
                "pageAction": "login", "userAgent": "UA", "cookies": "a=b"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = RecaptchaV2EnterpriseRequest::new("", "KEY")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = RecaptchaV2EnterpriseRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
        let err = RecaptchaV2EnterpriseRequest::new("https://example.com", "KEY")
            .with_enterprise_s("")
            .to_task()
            .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("enterprisePayload.s"))
        );
    }
}
