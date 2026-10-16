use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Alibaba captcha parameters. Only `scene_id` and `prefix` are always required; set the other
/// ones only if the target website uses them.
#[derive(Debug, Clone, Serialize)]
pub struct AlibabaMetadata {
    /// Captcha scenario identifier, e.g. `"1ww7426c"`.
    #[serde(rename = "sceneId")]
    pub scene_id: String,
    /// Captcha initialization parameter: the subdomain of the request URL that loads the task,
    /// e.g. `dlw3kug` for `https://dlw3kug.captcha-open.example.aliyuncs.com/`.
    pub prefix: String,
    /// Unique user/session identifier on the website side.
    #[serde(rename = "userId", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Additional (secondary) user identifier.
    #[serde(rename = "userUserId", skip_serializing_if = "Option::is_none")]
    pub user_user_id: Option<String>,
    /// Version/type of the verification mechanism, e.g. `"1.0"`.
    #[serde(rename = "verifyType", skip_serializing_if = "Option::is_none")]
    pub verify_type: Option<String>,
    /// Server region, e.g. `"sgp"`. Affects the token type (`cn` gives `WEB#`, `sgp` gives `SG_WEB#`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Verification ID of the current captcha session.
    #[serde(rename = "UserCertifyId", skip_serializing_if = "Option::is_none")]
    pub user_certify_id: Option<String>,
    /// Link to the captcha JS library used by the website (may change on every page render).
    #[serde(rename = "apiGetLib", skip_serializing_if = "Option::is_none")]
    pub api_get_lib: Option<String>,
    /// Full URL (with query) of the verification page, for websites where Alibaba returns a
    /// separate `/punish` URL.
    #[serde(rename = "punishUrl", skip_serializing_if = "Option::is_none")]
    pub punish_url: Option<String>,
    /// Return cookies together with the solution (use `true` for Alibaba WAF sites).
    #[serde(rename = "cookieRequired", skip_serializing_if = "Option::is_none")]
    pub cookie_required: Option<bool>,
}

impl AlibabaMetadata {
    pub fn new(scene_id: impl Into<String>, prefix: impl Into<String>) -> Self {
        Self {
            scene_id: scene_id.into(),
            prefix: prefix.into(),
            user_id: None,
            user_user_id: None,
            verify_type: None,
            region: None,
            user_certify_id: None,
            api_get_lib: None,
            punish_url: None,
            cookie_required: None,
        }
    }

    pub fn with_user_id(mut self, value: impl Into<String>) -> Self {
        self.user_id = Some(value.into());
        self
    }

    pub fn with_user_user_id(mut self, value: impl Into<String>) -> Self {
        self.user_user_id = Some(value.into());
        self
    }

    pub fn with_verify_type(mut self, value: impl Into<String>) -> Self {
        self.verify_type = Some(value.into());
        self
    }

    pub fn with_region(mut self, value: impl Into<String>) -> Self {
        self.region = Some(value.into());
        self
    }

    pub fn with_user_certify_id(mut self, value: impl Into<String>) -> Self {
        self.user_certify_id = Some(value.into());
        self
    }

    pub fn with_api_get_lib(mut self, value: impl Into<String>) -> Self {
        self.api_get_lib = Some(value.into());
        self
    }

    pub fn with_punish_url(mut self, value: impl Into<String>) -> Self {
        self.punish_url = Some(value.into());
        self
    }

    pub fn with_cookie_required(mut self, value: bool) -> Self {
        self.cookie_required = Some(value);
        self
    }
}

/// Alibaba CAPTCHA / WAF (`CustomTask`, class `alibaba`).
#[derive(Debug, Clone, Serialize)]
pub struct AlibabaCustomTaskRequest {
    /// Full URL of the page with the captcha.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    pub metadata: AlibabaMetadata,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl AlibabaCustomTaskRequest {
    pub fn new(website_url: impl Into<String>, metadata: AlibabaMetadata) -> Self {
        Self {
            website_url: website_url.into(),
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

impl CaptchaRequest for AlibabaCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("alibaba");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 3.0, 120.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("metadata.sceneId", &self.metadata.scene_id)?;
        require_non_empty("metadata.prefix", &self.metadata.prefix)?;
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
        let task = AlibabaCustomTaskRequest::new(
            "https://example.com",
            AlibabaMetadata::new("1ww7426c", "dlw3kug"),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "alibaba", "websiteURL": "https://example.com",
                "metadata": {"sceneId": "1ww7426c", "prefix": "dlw3kug"}
            })
        );
    }

    #[test]
    fn full_task() {
        let meta = AlibabaMetadata::new("S", "P")
            .with_user_id("U")
            .with_user_user_id("UU")
            .with_verify_type("1.0")
            .with_region("sgp")
            .with_user_certify_id("CID")
            .with_api_get_lib("https://x/AliyunCaptcha.js")
            .with_punish_url("https://x/punish?a=b")
            .with_cookie_required(true);
        let task = AlibabaCustomTaskRequest::new("https://example.com", meta)
            .with_proxy(ProxyInfo::new(ProxyType::Http, "1.2.3.4", 80))
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "alibaba", "websiteURL": "https://example.com",
                "metadata": {
                    "sceneId": "S", "prefix": "P", "userId": "U", "userUserId": "UU",
                    "verifyType": "1.0", "region": "sgp", "UserCertifyId": "CID",
                    "apiGetLib": "https://x/AliyunCaptcha.js", "punishUrl": "https://x/punish?a=b",
                    "cookieRequired": true
                },
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 80, "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = AlibabaCustomTaskRequest::new("", AlibabaMetadata::new("S", "P"))
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err =
            AlibabaCustomTaskRequest::new("https://example.com", AlibabaMetadata::new("", "P"))
                .to_task()
                .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.sceneId"))
        );
        let err =
            AlibabaCustomTaskRequest::new("https://example.com", AlibabaMetadata::new("S", ""))
                .to_task()
                .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.prefix"))
        );
    }
}
