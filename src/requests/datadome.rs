use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// DataDome solving method version (`datadomeVersion`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DataDomeVersion {
    /// Updated solving method supporting both `i.js` and `c.js` scripts (sent as `"new"`).
    #[serde(rename = "new")]
    New,
}

/// DataDome-specific challenge data. All values are dynamic: extract them right before
/// creating the task.
#[derive(Debug, Clone, Serialize)]
pub struct DataDomeMetadata {
    /// CAPTCHA URL with the latest dynamic parameters, usually
    /// `https://geo.captcha-delivery.com/captcha/?initialCid=...&hash=...&cid=...&t=fe&referer=...&s=...`.
    /// Submit it only if its `t` parameter is `fe` (`t=bv` means the proxy was blocked).
    #[serde(rename = "captchaUrl")]
    pub captcha_url: String,
    /// Your DataDome cookies (`datadome=...`).
    #[serde(rename = "datadomeCookie")]
    pub datadome_cookie: String,
    /// Solving method version. When absent the legacy method (only `i.js`) is used.
    #[serde(rename = "datadomeVersion", skip_serializing_if = "Option::is_none")]
    pub datadome_version: Option<DataDomeVersion>,
}

impl DataDomeMetadata {
    pub fn new(captcha_url: impl Into<String>, datadome_cookie: impl Into<String>) -> Self {
        Self {
            captcha_url: captcha_url.into(),
            datadome_cookie: datadome_cookie.into(),
            datadome_version: None,
        }
    }

    pub fn with_datadome_version(mut self, value: DataDomeVersion) -> Self {
        self.datadome_version = Some(value);
        self
    }
}

/// DataDome captcha (`CustomTask`, class `DataDome`). Your own proxy is required.
/// If the proxy uses IP authorization, whitelist `65.21.190.34`. The solution contains
/// cookies that must be set in the browser.
#[derive(Debug, Clone, Serialize)]
pub struct DataDomeCustomTaskRequest {
    /// The main page URL where the captcha is solved.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    pub metadata: DataDomeMetadata,
    /// Required proxy (non-transparent, not local).
    #[serde(flatten)]
    pub proxy: ProxyInfo,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl DataDomeCustomTaskRequest {
    pub fn new(
        website_url: impl Into<String>,
        metadata: DataDomeMetadata,
        proxy: ProxyInfo,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            metadata,
            proxy,
            user_agent: None,
        }
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for DataDomeCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("DataDome");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 3.0, 180.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("metadata.captchaUrl", &self.metadata.captcha_url)?;
        require_non_empty("metadata.datadomeCookie", &self.metadata.datadome_cookie)?;
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
        let task = DataDomeCustomTaskRequest::new(
            "https://example.com",
            DataDomeMetadata::new(
                "https://geo.captcha-delivery.com/captcha/?t=fe",
                "datadome=abc",
            ),
            proxy(),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "DataDome", "websiteURL": "https://example.com",
                "metadata": {"captchaUrl": "https://geo.captcha-delivery.com/captcha/?t=fe", "datadomeCookie": "datadome=abc"},
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                "proxyLogin": "u", "proxyPassword": "p"
            })
        );
    }

    #[test]
    fn full_task() {
        let task = DataDomeCustomTaskRequest::new(
            "https://example.com",
            DataDomeMetadata::new("U", "C").with_datadome_version(DataDomeVersion::New),
            proxy(),
        )
        .with_user_agent("UA")
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "DataDome", "websiteURL": "https://example.com",
                "metadata": {"captchaUrl": "U", "datadomeCookie": "C", "datadomeVersion": "new"},
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                "proxyLogin": "u", "proxyPassword": "p", "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = DataDomeCustomTaskRequest::new(
            "",
            DataDomeMetadata::new("U", "C"),
            ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080),
        )
        .to_task()
        .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = DataDomeCustomTaskRequest::new(
            "https://example.com",
            DataDomeMetadata::new("", "C"),
            ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.captchaUrl"))
        );
        let err = DataDomeCustomTaskRequest::new(
            "https://example.com",
            DataDomeMetadata::new("U", ""),
            ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.datadomeCookie"))
        );
    }
}
