use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// NetEase Yidun captcha (`YidunTask`). Setting `challenge` selects the Enterprise variant.
#[derive(Debug, Clone, Serialize)]
pub struct YidunRequest {
    /// Full URL of the page with the captcha.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The `siteKey` value found on the page.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Full URL (`https://...`) of the JS file that loads the captcha. Recommended when
    /// `challenge`, `hcg` or `hct` are present on the site.
    #[serde(rename = "yidunGetLib", skip_serializing_if = "Option::is_none")]
    pub yidun_get_lib: Option<String>,
    /// Yidun API server subdomain; must differ from `c.dun.163.com` / `c-v6.dun.163.com`
    /// and be given without `https://`. Needed for custom (often Enterprise) servers.
    #[serde(
        rename = "yidunApiServerSubdomain",
        skip_serializing_if = "Option::is_none"
    )]
    pub yidun_api_server_subdomain: Option<String>,
    /// Unique identifier of the current captcha; indicates the Enterprise captcha.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,
    /// Captcha hash used in the request; obtained together with `challenge`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hcg: Option<String>,
    /// Numeric timestamp used in Enterprise validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hct: Option<i64>,
}

impl YidunRequest {
    pub fn new(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            proxy: None,
            user_agent: None,
            yidun_get_lib: None,
            yidun_api_server_subdomain: None,
            challenge: None,
            hcg: None,
            hct: None,
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

    pub fn with_yidun_get_lib(mut self, value: impl Into<String>) -> Self {
        self.yidun_get_lib = Some(value.into());
        self
    }

    pub fn with_yidun_api_server_subdomain(mut self, value: impl Into<String>) -> Self {
        self.yidun_api_server_subdomain = Some(value.into());
        self
    }

    pub fn with_challenge(mut self, value: impl Into<String>) -> Self {
        self.challenge = Some(value.into());
        self
    }

    pub fn with_hcg(mut self, value: impl Into<String>) -> Self {
        self.hcg = Some(value.into());
        self
    }

    pub fn with_hct(mut self, value: i64) -> Self {
        self.hct = Some(value);
        self
    }
}

impl CaptchaRequest for YidunRequest {
    const TASK_TYPE: &'static str = "YidunTask";

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
        let task = YidunRequest::new("https://example.com", "KEY")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "YidunTask", "websiteURL": "https://example.com", "websiteKey": "KEY"})
        );
    }

    #[test]
    fn full_task() {
        let task = YidunRequest::new("https://example.com", "KEY")
            .with_proxy(ProxyInfo::new(ProxyType::Http, "1.2.3.4", 80))
            .with_user_agent("UA")
            .with_yidun_get_lib("https://cstaticdun.126.net/load.min.js")
            .with_yidun_api_server_subdomain("c.example.com")
            .with_challenge("ch")
            .with_hcg("hcg")
            .with_hct(1700000000000)
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "YidunTask", "websiteURL": "https://example.com", "websiteKey": "KEY",
                   "userAgent": "UA", "yidunGetLib": "https://cstaticdun.126.net/load.min.js",
                   "yidunApiServerSubdomain": "c.example.com", "challenge": "ch", "hcg": "hcg",
                   "hct": 1700000000000i64,
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 80})
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = YidunRequest::new("", "KEY").to_task().unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = YidunRequest::new("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
    }
}
