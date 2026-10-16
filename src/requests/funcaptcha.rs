use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// FunCaptcha / Arkose Labs (`FunCaptchaTask`).
///
/// The docs require your own proxy for this task (use the same proxy to obtain the captcha
/// parameters and to create the task), so `proxy` is mandatory.
#[derive(Debug, Clone, Serialize)]
pub struct FuncaptchaRequest {
    /// URL of the page where the captcha is located.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// FunCaptcha public key (`pk`), e.g. `EX72CCFB-26EX-40E5-91E6-85EX70BE98ED`.
    #[serde(rename = "websitePublicKey")]
    pub website_public_key: String,
    /// Mandatory proxy used by the worker.
    #[serde(flatten)]
    pub proxy: ProxyInfo,
    /// Arkose Labs subdomain (`surl`) without `https://`, only if it differs from
    /// `client-api.arkoselabs.com`.
    #[serde(
        rename = "funcaptchaApiJSSubdomain",
        skip_serializing_if = "Option::is_none"
    )]
    pub funcaptcha_api_js_subdomain: Option<String>,
    /// Additional parameter, required if the site uses `data[blob]`, e.g. `{"blob":"..."}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Cookies as `"name1=value1; name2=value2"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<String>,
    /// Browser User-Agent (use the value supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl FuncaptchaRequest {
    pub fn new(
        website_url: impl Into<String>,
        website_public_key: impl Into<String>,
        proxy: ProxyInfo,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_public_key: website_public_key.into(),
            proxy,
            funcaptcha_api_js_subdomain: None,
            data: None,
            cookies: None,
            user_agent: None,
        }
    }

    pub fn with_funcaptcha_api_js_subdomain(mut self, value: impl Into<String>) -> Self {
        self.funcaptcha_api_js_subdomain = Some(value.into());
        self
    }

    pub fn with_data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }

    pub fn with_cookies(mut self, value: impl Into<String>) -> Self {
        self.cookies = Some(value.into());
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for FuncaptchaRequest {
    const TASK_TYPE: &'static str = "FunCaptchaTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 10.0, 1.0, 80.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("websitePublicKey", &self.website_public_key)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::ProxyType;
    use serde_json::json;

    fn proxy() -> ProxyInfo {
        ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080)
    }

    #[test]
    fn minimal_task() {
        let task = FuncaptchaRequest::new("https://example.com", "PK", proxy())
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "FunCaptchaTask", "websiteURL": "https://example.com",
                   "websitePublicKey": "PK",
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080})
        );
    }

    #[test]
    fn full_task() {
        let task = FuncaptchaRequest::new("https://example.com", "PK", proxy().with_auth("u", "p"))
            .with_funcaptcha_api_js_subdomain("x-api.arkoselabs.com")
            .with_data("{\"blob\":\"B\"}")
            .with_cookies("a=b")
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "FunCaptchaTask", "websiteURL": "https://example.com",
                   "websitePublicKey": "PK",
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                   "proxyLogin": "u", "proxyPassword": "p",
                   "funcaptchaApiJSSubdomain": "x-api.arkoselabs.com",
                   "data": "{\"blob\":\"B\"}", "cookies": "a=b", "userAgent": "UA"})
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = FuncaptchaRequest::new("", "PK", proxy())
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = FuncaptchaRequest::new("https://example.com", "", proxy())
            .to_task()
            .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("websitePublicKey"))
        );
    }
}
