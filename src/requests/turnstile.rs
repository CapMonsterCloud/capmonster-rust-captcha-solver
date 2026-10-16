use serde::{Serialize, Serializer};

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Regular Cloudflare Turnstile widget (no `cloudflareTaskType`). All subtypes (manual,
/// non-interactive, invisible) are supported automatically. The result is a token.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TurnstileStandard {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// The `action` found in the callback used when the captcha is loaded.
    #[serde(rename = "pageAction", skip_serializing_if = "Option::is_none")]
    pub page_action: Option<String>,
    /// The `cData` value of the widget.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

impl TurnstileStandard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn with_page_action(mut self, value: impl Into<String>) -> Self {
        self.page_action = Some(value.into());
        self
    }

    pub fn with_data(mut self, value: impl Into<String>) -> Self {
        self.data = Some(value.into());
        self
    }
}

/// Cloudflare Challenge solved to a **token** (`cloudflareTaskType = "token"`).
///
/// The values are the parameters passed to `window.turnstile.render(el, paramsObj)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TurnstileChallengeToken {
    /// The `action` of the callback; usually `managed` or `non-interactive`.
    #[serde(rename = "pageAction")]
    pub page_action: String,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    /// The `cData` parameter, e.g. `874291f4retD1366`.
    pub data: String,
    /// The `chlPageData` parameter, e.g. `HUHDWUHuhuwfiweh32..uh2uhuhyugYUG=`.
    #[serde(rename = "pageData")]
    pub page_data: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// Link to the captcha script.
    #[serde(rename = "apiJsUrl", skip_serializing_if = "Option::is_none")]
    pub api_js_url: Option<String>,
}

impl TurnstileChallengeToken {
    pub fn new(
        page_action: impl Into<String>,
        user_agent: impl Into<String>,
        data: impl Into<String>,
        page_data: impl Into<String>,
    ) -> Self {
        Self {
            page_action: page_action.into(),
            user_agent: user_agent.into(),
            data: data.into(),
            page_data: page_data.into(),
            proxy: None,
            api_js_url: None,
        }
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_api_js_url(mut self, value: impl Into<String>) -> Self {
        self.api_js_url = Some(value.into());
        self
    }
}

/// Cloudflare Challenge solved to the `cf_clearance` cookie (`cloudflareTaskType =
/// "cf_clearance"`). Your **own proxy is mandatory**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TurnstileChallengeCookie {
    /// Base64 of the "Just a moment" HTML page returned with status 403.
    #[serde(rename = "htmlPageBase64")]
    pub html_page_base64: String,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    #[serde(flatten)]
    pub proxy: ProxyInfo,
}

impl TurnstileChallengeCookie {
    pub fn new(
        html_page_base64: impl Into<String>,
        user_agent: impl Into<String>,
        proxy: ProxyInfo,
    ) -> Self {
        Self {
            html_page_base64: html_page_base64.into(),
            user_agent: user_agent.into(),
            proxy,
        }
    }
}

/// Cloudflare Waiting Room (`cloudflareTaskType = "wait_room"`); the result is cookies.
/// Your **own proxy is mandatory**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TurnstileWaitRoom {
    /// Base64 of the page containing `<title>Waiting Room powered by Cloudflare</title>`.
    #[serde(rename = "htmlPageBase64")]
    pub html_page_base64: String,
    /// Browser User-Agent (use a current Windows one).
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    #[serde(flatten)]
    pub proxy: ProxyInfo,
}

impl TurnstileWaitRoom {
    pub fn new(
        html_page_base64: impl Into<String>,
        user_agent: impl Into<String>,
        proxy: ProxyInfo,
    ) -> Self {
        Self {
            html_page_base64: html_page_base64.into(),
            user_agent: user_agent.into(),
            proxy,
        }
    }
}

/// What kind of Cloudflare protection is being solved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnstileMode {
    Standard(TurnstileStandard),
    ChallengeToken(TurnstileChallengeToken),
    ChallengeCookie(TurnstileChallengeCookie),
    WaitRoom(TurnstileWaitRoom),
}

macro_rules! impl_from_mode {
    ($($ty:ident => $variant:ident),*) => {$(
        impl From<$ty> for TurnstileMode {
            fn from(v: $ty) -> Self {
                Self::$variant(v)
            }
        }
    )*};
}
impl_from_mode!(
    TurnstileStandard => Standard,
    TurnstileChallengeToken => ChallengeToken,
    TurnstileChallengeCookie => ChallengeCookie,
    TurnstileWaitRoom => WaitRoom
);

#[derive(Serialize)]
struct Tagged<'a, T: Serialize> {
    #[serde(rename = "cloudflareTaskType")]
    cloudflare_task_type: &'static str,
    #[serde(flatten)]
    inner: &'a T,
}

impl Serialize for TurnstileMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            TurnstileMode::Standard(v) => v.serialize(serializer),
            TurnstileMode::ChallengeToken(v) => Tagged {
                cloudflare_task_type: "token",
                inner: v,
            }
            .serialize(serializer),
            TurnstileMode::ChallengeCookie(v) => Tagged {
                cloudflare_task_type: "cf_clearance",
                inner: v,
            }
            .serialize(serializer),
            TurnstileMode::WaitRoom(v) => Tagged {
                cloudflare_task_type: "wait_room",
                inner: v,
            }
            .serialize(serializer),
        }
    }
}

/// Cloudflare Turnstile / Challenge / Waiting Room (`TurnstileTask`).
///
/// Pick the mode with [`TurnstileMode`] (or pass one of the parameter structs directly):
/// [`TurnstileStandard`], [`TurnstileChallengeToken`], [`TurnstileChallengeCookie`],
/// [`TurnstileWaitRoom`]. Each one only has the fields the API accepts for that mode, and
/// the required ones are constructor arguments.
#[derive(Debug, Clone, Serialize)]
pub struct TurnstileRequest {
    /// URL of the page where the captcha is solved.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Turnstile site key, e.g. `0x4AAAAAAABUYP0XeMJF0xoy`. For `cf_clearance` / `wait_room`
    /// any string may be passed (e.g. `xxxxxx`).
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    #[serde(flatten)]
    pub mode: TurnstileMode,
}

impl TurnstileRequest {
    pub fn new(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        mode: impl Into<TurnstileMode>,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
            mode: mode.into(),
        }
    }

    /// Regular Turnstile with no optional parameters.
    pub fn standard(website_url: impl Into<String>, website_key: impl Into<String>) -> Self {
        Self::new(website_url, website_key, TurnstileStandard::new())
    }
}

impl CaptchaRequest for TurnstileRequest {
    const TASK_TYPE: &'static str = "TurnstileTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 1.0, 80.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("websiteKey", &self.website_key)?;
        match &self.mode {
            TurnstileMode::Standard(_) => {}
            TurnstileMode::ChallengeToken(v) => {
                require_non_empty("pageAction", &v.page_action)?;
                require_non_empty("userAgent", &v.user_agent)?;
                require_non_empty("data", &v.data)?;
                require_non_empty("pageData", &v.page_data)?;
            }
            TurnstileMode::ChallengeCookie(v) => {
                require_non_empty("htmlPageBase64", &v.html_page_base64)?;
                require_non_empty("userAgent", &v.user_agent)?;
            }
            TurnstileMode::WaitRoom(v) => {
                require_non_empty("htmlPageBase64", &v.html_page_base64)?;
                require_non_empty("userAgent", &v.user_agent)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::proxy::ProxyType;
    use serde_json::json;

    fn proxy() -> ProxyInfo {
        ProxyInfo::new(ProxyType::Http, "1.2.3.4", 8080).with_auth("u", "p")
    }

    #[test]
    fn standard_minimal() {
        let task = TurnstileRequest::standard("https://example.com", "0x4A")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "TurnstileTask", "websiteURL": "https://example.com",
                   "websiteKey": "0x4A"})
        );
    }

    #[test]
    fn standard_full() {
        let task = TurnstileRequest::new(
            "https://example.com",
            "0x4A",
            TurnstileStandard::new()
                .with_proxy(proxy())
                .with_user_agent("UA")
                .with_page_action("login")
                .with_data("cdata"),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({"type": "TurnstileTask", "websiteURL": "https://example.com",
                   "websiteKey": "0x4A", "userAgent": "UA", "pageAction": "login",
                   "data": "cdata", "proxyType": "http", "proxyAddress": "1.2.3.4",
                   "proxyPort": 8080, "proxyLogin": "u", "proxyPassword": "p"})
        );
    }

    #[test]
    fn challenge_token_minimal_and_full() {
        let minimal = TurnstileRequest::new(
            "https://example.com",
            "0x4A",
            TurnstileChallengeToken::new("managed", "UA", "cdata", "pdata"),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            minimal,
            json!({"type": "TurnstileTask", "websiteURL": "https://example.com",
                   "websiteKey": "0x4A", "cloudflareTaskType": "token",
                   "pageAction": "managed", "userAgent": "UA", "data": "cdata",
                   "pageData": "pdata"})
        );

        let full = TurnstileRequest::new(
            "https://example.com",
            "0x4A",
            TurnstileChallengeToken::new("managed", "UA", "cdata", "pdata")
                .with_proxy(proxy())
                .with_api_js_url("https://challenges.cloudflare.com/api.js"),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            full,
            json!({"type": "TurnstileTask", "websiteURL": "https://example.com",
                   "websiteKey": "0x4A", "cloudflareTaskType": "token",
                   "pageAction": "managed", "userAgent": "UA", "data": "cdata",
                   "pageData": "pdata", "apiJsUrl": "https://challenges.cloudflare.com/api.js",
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                   "proxyLogin": "u", "proxyPassword": "p"})
        );
    }

    #[test]
    fn cf_clearance_task() {
        let task = TurnstileRequest::new(
            "https://example.com",
            "xxxxxx",
            TurnstileChallengeCookie::new("PGh0bWw+", "UA", proxy()),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({"type": "TurnstileTask", "websiteURL": "https://example.com",
                   "websiteKey": "xxxxxx", "cloudflareTaskType": "cf_clearance",
                   "htmlPageBase64": "PGh0bWw+", "userAgent": "UA",
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                   "proxyLogin": "u", "proxyPassword": "p"})
        );
    }

    #[test]
    fn wait_room_task() {
        let task = TurnstileRequest::new(
            "https://example.com",
            "xxxxxx",
            TurnstileWaitRoom::new("PGh0bWw+", "UA", proxy()),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({"type": "TurnstileTask", "websiteURL": "https://example.com",
                   "websiteKey": "xxxxxx", "cloudflareTaskType": "wait_room",
                   "htmlPageBase64": "PGh0bWw+", "userAgent": "UA",
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 8080,
                   "proxyLogin": "u", "proxyPassword": "p"})
        );
    }

    #[test]
    fn empty_html_is_rejected() {
        let err = TurnstileRequest::new(
            "https://example.com",
            "xxxxxx",
            TurnstileWaitRoom::new("", "UA", proxy()),
        )
        .to_task()
        .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let token = |a: &str, u: &str, d: &str, p: &str| {
            TurnstileRequest::new(
                "https://example.com",
                "0x4A",
                TurnstileChallengeToken::new(a, u, d, p),
            )
        };
        for (req, field) in [
            (TurnstileRequest::standard("", "0x4A"), "websiteURL"),
            (
                TurnstileRequest::standard("https://example.com", ""),
                "websiteKey",
            ),
            (token("", "UA", "d", "p"), "pageAction"),
            (token("a", "", "d", "p"), "userAgent"),
            (token("a", "UA", "", "p"), "data"),
            (token("a", "UA", "d", ""), "pageData"),
        ] {
            let err = req.to_task().unwrap_err();
            assert!(
                matches!(err, Error::Validation(ref m) if m.contains(field)),
                "{field}"
            );
        }
        let err = TurnstileRequest::new(
            "https://example.com",
            "0x4A",
            TurnstileChallengeCookie::new("PGh0bWw+", "", proxy()),
        )
        .to_task()
        .unwrap_err();
        assert!(matches!(err, Error::Validation(ref m) if m.contains("userAgent")));
    }
}
