use serde::ser::{Serialize, SerializeMap, Serializer};
use serde::Serialize as DeriveSerialize;

use super::{require_non_empty, CaptchaRequest};
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// What the HUNT solver should do. `data` and `widgetUrl` are mutually exclusive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HuntMode {
    /// Only generate a fingerprint (just `apiGetLib` is sent).
    Fingerprint,
    /// Solve the captcha via the `meta.token` value received from the website (`data`).
    Token(String),
    /// Solve the captcha via the full URL of the HUNT widget, including domain, path and query
    /// (`widgetUrl`), e.g. `https://captcha.example.com/widget?hash=<widget-hash>`.
    WidgetUrl(String),
}

/// HUNT parameters. Values are dynamic: extract them right before creating the task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HuntMetadata {
    /// Full URL of the `api.js` file (look for `hd-api` or `api.js` in the Network tab), e.g.
    /// `https://example.com/hd-api/external/apps/<app-hash>/api.js`.
    pub api_get_lib: String,
    pub mode: HuntMode,
}

impl HuntMetadata {
    /// Fingerprint generation: only `apiGetLib` is sent.
    pub fn fingerprint(api_get_lib: impl Into<String>) -> Self {
        Self {
            api_get_lib: api_get_lib.into(),
            mode: HuntMode::Fingerprint,
        }
    }

    /// Captcha solving via the `meta.token` value (`data`).
    pub fn with_token(api_get_lib: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            api_get_lib: api_get_lib.into(),
            mode: HuntMode::Token(data.into()),
        }
    }

    /// Captcha solving via the full widget URL (`widgetUrl`).
    pub fn with_widget_url(api_get_lib: impl Into<String>, widget_url: impl Into<String>) -> Self {
        Self {
            api_get_lib: api_get_lib.into(),
            mode: HuntMode::WidgetUrl(widget_url.into()),
        }
    }
}

impl Serialize for HuntMetadata {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = if self.mode == HuntMode::Fingerprint {
            1
        } else {
            2
        };
        let mut map = serializer.serialize_map(Some(len))?;
        map.serialize_entry("apiGetLib", &self.api_get_lib)?;
        match &self.mode {
            HuntMode::Fingerprint => {}
            HuntMode::Token(data) => map.serialize_entry("data", data)?,
            HuntMode::WidgetUrl(url) => map.serialize_entry("widgetUrl", url)?,
        }
        map.end()
    }
}

/// HUNT captcha (`CustomTask`, class `HUNT`). Your own proxy is required.
#[derive(Debug, Clone, DeriveSerialize)]
pub struct HuntCustomTaskRequest {
    /// Full URL of the page where HUNT is used.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    pub metadata: HuntMetadata,
    /// Required proxy (non-transparent, not local).
    #[serde(flatten)]
    pub proxy: ProxyInfo,
    /// Browser User-Agent (current Windows UA supported by CapMonster Cloud).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl HuntCustomTaskRequest {
    pub fn new(website_url: impl Into<String>, metadata: HuntMetadata, proxy: ProxyInfo) -> Self {
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

impl CaptchaRequest for HuntCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("HUNT");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 3.0, 180.0)
    }

    fn validate(&self) -> crate::error::Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("metadata.apiGetLib", &self.metadata.api_get_lib)?;
        match &self.metadata.mode {
            HuntMode::Fingerprint => {}
            HuntMode::Token(data) => require_non_empty("metadata.data", data)?,
            HuntMode::WidgetUrl(url) => require_non_empty("metadata.widgetUrl", url)?,
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
    fn minimal_fingerprint_task() {
        let task = HuntCustomTaskRequest::new(
            "https://example.com",
            HuntMetadata::fingerprint("https://example.com/api.js"),
            proxy(),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "HUNT", "websiteURL": "https://example.com",
                "metadata": {"apiGetLib": "https://example.com/api.js"},
                "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                "proxyLogin": "u", "proxyPassword": "p"
            })
        );
    }

    #[test]
    fn full_token_task() {
        let task = HuntCustomTaskRequest::new(
            "https://example.com",
            HuntMetadata::with_token("L", "TOKEN"),
            proxy(),
        )
        .with_user_agent("UA")
        .to_task()
        .unwrap();
        assert_eq!(task["metadata"], json!({"apiGetLib": "L", "data": "TOKEN"}));
        assert_eq!(task["userAgent"], "UA");
    }

    #[test]
    fn widget_url_task() {
        let task = HuntCustomTaskRequest::new(
            "https://example.com",
            HuntMetadata::with_widget_url("L", "https://c.example.com/widget?hash=h"),
            proxy(),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task["metadata"],
            json!({"apiGetLib": "L", "widgetUrl": "https://c.example.com/widget?hash=h"})
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = HuntCustomTaskRequest::new("", HuntMetadata::fingerprint("L"), proxy())
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = HuntCustomTaskRequest::new(
            "https://example.com",
            HuntMetadata::fingerprint(""),
            proxy(),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.apiGetLib"))
        );
        let err = HuntCustomTaskRequest::new(
            "https://example.com",
            HuntMetadata::with_token("L", ""),
            proxy(),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.data"))
        );
        let err = HuntCustomTaskRequest::new(
            "https://example.com",
            HuntMetadata::with_widget_url("L", ""),
            proxy(),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.widgetUrl"))
        );
    }
}
