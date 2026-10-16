use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::{Error, Result};
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// TSPD challenge data. Values are dynamic: extract them right before creating the task.
#[derive(Debug, Clone, Serialize)]
pub struct TspdMetadata {
    /// Cookies obtained on the TSPD challenge page, e.g. `"TS386a400d029=0826...; ...;"`.
    #[serde(rename = "tspdCookie")]
    pub tspd_cookie: String,
    /// The entire TSPD page encoded in base64.
    #[serde(rename = "htmlPageBase64")]
    pub html_page_base64: String,
}

impl TspdMetadata {
    pub fn new(tspd_cookie: impl Into<String>, html_page_base64: impl Into<String>) -> Self {
        Self {
            tspd_cookie: tspd_cookie.into(),
            html_page_base64: html_page_base64.into(),
        }
    }
}

/// TSPD (`CustomTask`, class `tspd`). Your own proxy is required.
#[derive(Debug, Clone, Serialize)]
pub struct TspdCustomTaskRequest {
    /// URL of the page where the TSPD challenge is located.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    pub metadata: TspdMetadata,
    /// Required proxy. The proxy login must not contain the characters `;`, `|`, `)`.
    #[serde(flatten)]
    pub proxy: ProxyInfo,
    /// Browser User-Agent (current Windows UA); should match the one used when the cookie and
    /// the page were obtained.
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl TspdCustomTaskRequest {
    pub fn new(website_url: impl Into<String>, metadata: TspdMetadata, proxy: ProxyInfo) -> Self {
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

impl CaptchaRequest for TspdCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("tspd");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 1.0, 60.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("metadata.tspdCookie", &self.metadata.tspd_cookie)?;
        require_non_empty("metadata.htmlPageBase64", &self.metadata.html_page_base64)?;
        if let Some(login) = &self.proxy.proxy_login {
            if login.contains([';', '|', ')']) {
                return Err(Error::Validation(
                    "proxy login must not contain the characters ';', '|', ')' for TSPD".into(),
                ));
            }
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
        let task = TspdCustomTaskRequest::new(
            "https://example.com",
            TspdMetadata::new("TS1=a", "PGh0bWw+"),
            proxy(),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "tspd", "websiteURL": "https://example.com",
                "metadata": {"tspdCookie": "TS1=a", "htmlPageBase64": "PGh0bWw+"},
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                "proxyLogin": "u", "proxyPassword": "p"
            })
        );
    }

    #[test]
    fn full_task() {
        let task =
            TspdCustomTaskRequest::new("https://example.com", TspdMetadata::new("C", "H"), proxy())
                .with_user_agent("UA")
                .to_task()
                .unwrap();
        assert_eq!(task["userAgent"], "UA");
        assert_eq!(
            task["metadata"],
            json!({"tspdCookie": "C", "htmlPageBase64": "H"})
        );
    }

    #[test]
    fn rejects_forbidden_login_chars() {
        for bad in ["a;b", "a|b", "a)b"] {
            let p = ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080).with_auth(bad, "p");
            let req = TspdCustomTaskRequest::new("https://e.com", TspdMetadata::new("C", "H"), p);
            assert!(matches!(req.to_task(), Err(Error::Validation(_))), "{bad}");
        }
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        for (url, cookie, html, field) in [
            ("", "C", "H", "websiteURL"),
            ("https://example.com", "", "H", "metadata.tspdCookie"),
            ("https://example.com", "C", "", "metadata.htmlPageBase64"),
        ] {
            let err = TspdCustomTaskRequest::new(url, TspdMetadata::new(cookie, html), proxy())
                .to_task()
                .unwrap_err();
            assert!(matches!(err, Error::Validation(ref m) if m.contains(field)));
        }
    }
}
