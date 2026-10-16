use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Optional TenDI (Tencent captcha) parameters.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TenDiMetadata {
    /// Link to the captcha script, usually `TCaptcha.js` / `TCaptcha-global.js`
    /// (`TJNCaptcha*.js` / `TJCaptcha*.js` for the second version).
    #[serde(rename = "captchaUrl", skip_serializing_if = "Option::is_none")]
    pub captcha_url: Option<String>,
    /// `Aid` value obtained from the target website; single-use, fetch a new one for every task.
    /// Not used on all websites: omit it if the site does not return an Aid.
    #[serde(rename = "aidEncrypted", skip_serializing_if = "Option::is_none")]
    pub aid_encrypted: Option<String>,
}

impl TenDiMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_captcha_url(mut self, value: impl Into<String>) -> Self {
        self.captcha_url = Some(value.into());
        self
    }

    pub fn with_aid_encrypted(mut self, value: impl Into<String>) -> Self {
        self.aid_encrypted = Some(value.into());
        self
    }
}

/// TenDI / Tencent captcha (`CustomTask`, class `TenDI`).
#[derive(Debug, Clone, Serialize)]
pub struct TenDiCustomTaskRequest {
    /// Address of the main page where the captcha is solved.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// `captchaAppId`, the `aid` value (e.g. `"189123456"`), unique per website.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TenDiMetadata>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl TenDiCustomTaskRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            metadata: None,
            proxy: None,
            user_agent: None,
        }
    }

    pub fn with_metadata(mut self, value: TenDiMetadata) -> Self {
        self.metadata = Some(value);
        self
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

impl CaptchaRequest for TenDiCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("TenDI");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 10.0, 3.0, 180.0)
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
        let task = TenDiCustomTaskRequest::new("https://example.com", "189123456")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "CustomTask", "class": "TenDI", "websiteURL": "https://example.com", "websiteKey": "189123456"})
        );
    }

    #[test]
    fn full_task() {
        let task = TenDiCustomTaskRequest::new("https://example.com", "189123456")
            .with_metadata(
                TenDiMetadata::new()
                    .with_captcha_url("https://x/TCaptcha.js")
                    .with_aid_encrypted("AID"),
            )
            .with_proxy(ProxyInfo::new(ProxyType::Http, "1.2.3.4", 80))
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "TenDI", "websiteURL": "https://example.com", "websiteKey": "189123456",
                "metadata": {"captchaUrl": "https://x/TCaptcha.js", "aidEncrypted": "AID"},
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 80, "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = TenDiCustomTaskRequest::new("", "KEY")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = TenDiCustomTaskRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
