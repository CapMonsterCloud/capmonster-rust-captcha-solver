use serde::ser::{Serialize, SerializeMap, Serializer};
use serde_json::Value;

use super::{require_non_empty, CaptchaRequest};
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// GeeTest version together with the parameters that only exist for that version.
///
/// Serialized (flattened into the task) as `"version": 3, "challenge": ...` or
/// `"version": 4, "initParameters": ...`.
#[derive(Debug, Clone, PartialEq)]
pub enum GeetestVersion {
    /// GeeTest v3. `challenge` is a dynamic key: obtain a fresh one before creating each task
    /// and before the captcha is initialized on the page (afterwards it becomes invalid).
    V3 { challenge: String },
    /// GeeTest v4 (`gt` is the `captcha_id`). `init_parameters` are extra v4 parameters used
    /// together with `riskType`, e.g. `{"riskType": "slide"}`.
    V4 { init_parameters: Option<Value> },
}

impl Serialize for GeetestVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            GeetestVersion::V3 { challenge } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("version", &3)?;
                map.serialize_entry("challenge", challenge)?;
                map.end()
            }
            GeetestVersion::V4 { init_parameters } => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("version", &4)?;
                if let Some(params) = init_parameters {
                    map.serialize_entry("initParameters", params)?;
                }
                map.end()
            }
        }
    }
}

/// GeeTest captcha (`GeeTestTask`), versions 3 and 4.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GeetestRequest {
    /// URL of the page where the captcha is initialized (for v3 usually the `Referer` of the
    /// `gettype.php` request, which may differ from the URL in the address bar).
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// The `gt` key of the domain (for v4: the `captcha_id`).
    pub gt: String,
    /// Version-specific parameters.
    #[serde(flatten)]
    pub version: GeetestVersion,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// GeeTest API server subdomain (must differ from `api.geetest.com`). May be required
    /// for some sites.
    #[serde(
        rename = "geetestApiServerSubdomain",
        skip_serializing_if = "Option::is_none"
    )]
    pub geetest_api_server_subdomain: Option<String>,
    /// Path to the captcha script used to display it on the page, **sent as a JSON string**.
    /// May be required for some sites.
    #[serde(rename = "geetestGetLib", skip_serializing_if = "Option::is_none")]
    pub geetest_get_lib: Option<String>,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl GeetestRequest {
    fn with_version(
        website_url: impl Into<String>,
        gt: impl Into<String>,
        version: GeetestVersion,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            gt: gt.into(),
            version,
            proxy: None,
            geetest_api_server_subdomain: None,
            geetest_get_lib: None,
            user_agent: None,
        }
    }

    /// GeeTest v3 request; `challenge` is required by the API for this version.
    pub fn new_v3(
        website_url: impl Into<String>,
        gt: impl Into<String>,
        challenge: impl Into<String>,
    ) -> Self {
        Self::with_version(
            website_url,
            gt,
            GeetestVersion::V3 {
                challenge: challenge.into(),
            },
        )
    }

    /// GeeTest v4 request without `initParameters`.
    pub fn new_v4(website_url: impl Into<String>, gt: impl Into<String>) -> Self {
        Self::with_version(
            website_url,
            gt,
            GeetestVersion::V4 {
                init_parameters: None,
            },
        )
    }

    /// GeeTest v4 request with `initParameters` (e.g. `json!({"riskType": "slide"})`).
    pub fn new_v4_with_init_parameters(
        website_url: impl Into<String>,
        gt: impl Into<String>,
        init_parameters: Value,
    ) -> Self {
        Self::with_version(
            website_url,
            gt,
            GeetestVersion::V4 {
                init_parameters: Some(init_parameters),
            },
        )
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_geetest_api_server_subdomain(mut self, value: impl Into<String>) -> Self {
        self.geetest_api_server_subdomain = Some(value.into());
        self
    }

    pub fn with_geetest_get_lib(mut self, value: impl Into<String>) -> Self {
        self.geetest_get_lib = Some(value.into());
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for GeetestRequest {
    const TASK_TYPE: &'static str = "GeeTestTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 1.0, 80.0)
    }

    fn validate(&self) -> crate::error::Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("gt", &self.gt)?;
        if let GeetestVersion::V3 { challenge } = &self.version {
            require_non_empty("challenge", challenge)?;
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
    fn v3_minimal() {
        let task = GeetestRequest::new_v3("https://example.com", "GT", "CH")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "GeeTestTask", "websiteURL": "https://example.com", "gt": "GT",
                   "version": 3, "challenge": "CH"})
        );
    }

    #[test]
    fn v3_full() {
        let task = GeetestRequest::new_v3("https://example.com", "GT", "CH")
            .with_proxy(ProxyInfo::new(ProxyType::Http, "1.2.3.4", 80).with_auth("u", "p"))
            .with_geetest_api_server_subdomain("api-na.geetest.com")
            .with_geetest_get_lib("{\"static_servers\":[\"x\"]}")
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "GeeTestTask", "websiteURL": "https://example.com", "gt": "GT",
                   "version": 3, "challenge": "CH",
                   "geetestApiServerSubdomain": "api-na.geetest.com",
                   "geetestGetLib": "{\"static_servers\":[\"x\"]}", "userAgent": "UA",
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 80,
                   "proxyLogin": "u", "proxyPassword": "p"})
        );
    }

    #[test]
    fn v4_minimal_has_no_challenge() {
        let task = GeetestRequest::new_v4("https://example.com", "CAPTCHA_ID")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "GeeTestTask", "websiteURL": "https://example.com",
                   "gt": "CAPTCHA_ID", "version": 4})
        );
    }

    #[test]
    fn v4_full() {
        let task = GeetestRequest::new_v4_with_init_parameters(
            "https://example.com",
            "CAPTCHA_ID",
            json!({"riskType": "slide"}),
        )
        .with_user_agent("UA")
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({"type": "GeeTestTask", "websiteURL": "https://example.com",
                   "gt": "CAPTCHA_ID", "version": 4,
                   "initParameters": {"riskType": "slide"}, "userAgent": "UA"})
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = GeetestRequest::new_v3("", "GT", "CH")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = GeetestRequest::new_v3("https://example.com", "", "CH")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("gt")));
        let err = GeetestRequest::new_v3("https://example.com", "GT", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("challenge")));
        let err = GeetestRequest::new_v4("https://example.com", "")
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("gt")));
    }
}
