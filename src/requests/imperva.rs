use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Imperva (Incapsula) challenge data. Values are dynamic: extract them right before
/// creating the task.
#[derive(Debug, Clone, Serialize)]
pub struct ImpervaMetadata {
    /// Name of the Incapsula JS file, e.g. `_Incapsula_Resource?SWJIYLWA=719d34d3...`.
    #[serde(rename = "incapsulaScriptUrl")]
    pub incapsula_script_url: String,
    /// Incapsula cookies, e.g. `"incap_sess_*=...; visid_incap_*=..."`.
    #[serde(rename = "incapsulaCookies")]
    pub incapsula_cookies: String,
    /// Name of the endpoint the reese84 fingerprint is sent to (ends with `?d=site.com`).
    #[serde(rename = "reese84UrlEndpoint", skip_serializing_if = "Option::is_none")]
    pub reese84_url_endpoint: Option<String>,
}

impl ImpervaMetadata {
    pub fn new(
        incapsula_script_url: impl Into<String>,
        incapsula_cookies: impl Into<String>,
    ) -> Self {
        Self {
            incapsula_script_url: incapsula_script_url.into(),
            incapsula_cookies: incapsula_cookies.into(),
            reese84_url_endpoint: None,
        }
    }

    pub fn with_reese84_url_endpoint(mut self, value: impl Into<String>) -> Self {
        self.reese84_url_endpoint = Some(value.into());
        self
    }
}

/// Imperva / Incapsula (`CustomTask`, class `Imperva`). Your own proxy is required.
#[derive(Debug, Clone, Serialize)]
pub struct ImpervaCustomTaskRequest {
    /// Address of the main page where Incapsula is located.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    pub metadata: ImpervaMetadata,
    /// Required proxy (non-transparent, not local).
    #[serde(flatten)]
    pub proxy: ProxyInfo,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl ImpervaCustomTaskRequest {
    pub fn new(
        website_url: impl Into<String>,
        metadata: ImpervaMetadata,
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

impl CaptchaRequest for ImpervaCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("Imperva");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 1.0, 15.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty(
            "metadata.incapsulaScriptUrl",
            &self.metadata.incapsula_script_url,
        )?;
        require_non_empty(
            "metadata.incapsulaCookies",
            &self.metadata.incapsula_cookies,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::ProxyType;
    use serde_json::json;

    fn proxy() -> ProxyInfo {
        ProxyInfo::new(ProxyType::Socks5, "1.2.3.4", 1080).with_auth("u", "p")
    }

    #[test]
    fn minimal_task() {
        let task = ImpervaCustomTaskRequest::new(
            "https://example.com",
            ImpervaMetadata::new("_Incapsula_Resource?x=1", "incap_sess_1=a"),
            proxy(),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "Imperva", "websiteURL": "https://example.com",
                "metadata": {"incapsulaScriptUrl": "_Incapsula_Resource?x=1", "incapsulaCookies": "incap_sess_1=a"},
                "proxyType": "socks5", "proxyAddress": "1.2.3.4", "proxyPort": 1080,
                "proxyLogin": "u", "proxyPassword": "p"
            })
        );
    }

    #[test]
    fn full_task() {
        let task = ImpervaCustomTaskRequest::new(
            "https://example.com",
            ImpervaMetadata::new("S", "C").with_reese84_url_endpoint("E?d=example.com"),
            proxy(),
        )
        .with_user_agent("UA")
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "Imperva", "websiteURL": "https://example.com",
                "metadata": {"incapsulaScriptUrl": "S", "incapsulaCookies": "C", "reese84UrlEndpoint": "E?d=example.com"},
                "proxyType": "socks5", "proxyAddress": "1.2.3.4", "proxyPort": 1080,
                "proxyLogin": "u", "proxyPassword": "p", "userAgent": "UA"
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = ImpervaCustomTaskRequest::new("", ImpervaMetadata::new("S", "C"), proxy())
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = ImpervaCustomTaskRequest::new(
            "https://example.com",
            ImpervaMetadata::new("", "C"),
            proxy(),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.incapsulaScriptUrl"))
        );
        let err = ImpervaCustomTaskRequest::new(
            "https://example.com",
            ImpervaMetadata::new("S", ""),
            proxy(),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.incapsulaCookies"))
        );
    }
}
