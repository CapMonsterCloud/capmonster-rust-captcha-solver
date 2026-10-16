use serde::ser::{Serialize, SerializeMap, Serializer};

use super::{require_non_empty, CaptchaRequest};
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Option 1: visible AWS WAF captcha (`jsapi.js` / `captcha.js` only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmazonVisibleCaptcha {
    /// Value of the `apiKey` field used when rendering the captcha.
    pub website_key: String,
    /// Link to `jsapi.js` on the HTML page (`<Integration URL>jsapi.js`).
    pub captcha_script: String,
    /// Browser User-Agent (current Windows one). Only the visible-captcha option accepts it.
    pub user_agent: Option<String>,
}

impl AmazonVisibleCaptcha {
    pub fn new(website_key: impl Into<String>, captcha_script: impl Into<String>) -> Self {
        Self {
            website_key: website_key.into(),
            captcha_script: captcha_script.into(),
            user_agent: None,
        }
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

/// Option 2: challenge plus captcha. The values come from `window.gokuProps`
/// (`key`, `context`, `iv`) on the 405 page of the protected site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmazonChallengeAndCaptcha {
    /// Link to `challenge.js`.
    pub challenge_script: String,
    /// `window.gokuProps.key`.
    pub website_key: String,
    /// `window.gokuProps.context`.
    pub context: String,
    /// `window.gokuProps.iv`.
    pub iv: String,
    /// Link to `captcha.js`; may be absent if only a challenge is present.
    pub captcha_script: Option<String>,
}

impl AmazonChallengeAndCaptcha {
    pub fn new(
        challenge_script: impl Into<String>,
        website_key: impl Into<String>,
        context: impl Into<String>,
        iv: impl Into<String>,
    ) -> Self {
        Self {
            challenge_script: challenge_script.into(),
            website_key: website_key.into(),
            context: context.into(),
            iv: iv.into(),
            captcha_script: None,
        }
    }

    pub fn with_captcha_script(mut self, value: impl Into<String>) -> Self {
        self.captcha_script = Some(value.into());
        self
    }
}

/// Option 3: invisible captcha, only `challenge.js` is loaded. `context` and `iv` are sent
/// as empty strings, as the API requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmazonChallengeOnly {
    /// Link to `challenge.js`.
    pub challenge_script: String,
}

impl AmazonChallengeOnly {
    pub fn new(challenge_script: impl Into<String>) -> Self {
        Self {
            challenge_script: challenge_script.into(),
        }
    }
}

/// The three mutually exclusive parameter sets accepted by the API for `AmazonTask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmazonWafVariant {
    VisibleCaptcha(AmazonVisibleCaptcha),
    ChallengeAndCaptcha(AmazonChallengeAndCaptcha),
    ChallengeOnly(AmazonChallengeOnly),
}

impl From<AmazonVisibleCaptcha> for AmazonWafVariant {
    fn from(v: AmazonVisibleCaptcha) -> Self {
        Self::VisibleCaptcha(v)
    }
}

impl From<AmazonChallengeAndCaptcha> for AmazonWafVariant {
    fn from(v: AmazonChallengeAndCaptcha) -> Self {
        Self::ChallengeAndCaptcha(v)
    }
}

impl From<AmazonChallengeOnly> for AmazonWafVariant {
    fn from(v: AmazonChallengeOnly) -> Self {
        Self::ChallengeOnly(v)
    }
}

impl Serialize for AmazonWafVariant {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        match self {
            AmazonWafVariant::VisibleCaptcha(v) => {
                map.serialize_entry("websiteKey", &v.website_key)?;
                map.serialize_entry("captchaScript", &v.captcha_script)?;
                if let Some(ua) = &v.user_agent {
                    map.serialize_entry("userAgent", ua)?;
                }
            }
            AmazonWafVariant::ChallengeAndCaptcha(v) => {
                map.serialize_entry("challengeScript", &v.challenge_script)?;
                map.serialize_entry("websiteKey", &v.website_key)?;
                map.serialize_entry("context", &v.context)?;
                map.serialize_entry("iv", &v.iv)?;
                if let Some(script) = &v.captcha_script {
                    map.serialize_entry("captchaScript", script)?;
                }
            }
            AmazonWafVariant::ChallengeOnly(v) => {
                map.serialize_entry("challengeScript", &v.challenge_script)?;
                map.serialize_entry("context", "")?;
                map.serialize_entry("iv", "")?;
            }
        }
        map.end()
    }
}

/// Amazon AWS WAF captcha and challenge (`AmazonTask`).
///
/// Build it with [`AmazonWafRequest::new`] and one of [`AmazonVisibleCaptcha`],
/// [`AmazonChallengeAndCaptcha`], [`AmazonChallengeOnly`]; the type system makes it impossible
/// to mix parameters of different options.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AmazonWafRequest {
    /// Address of the main page where the captcha is solved.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Which of the three parameter sets is used.
    #[serde(flatten)]
    pub variant: AmazonWafVariant,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    /// `false` (default): the response has `captcha_voucher` and `existing_token`.
    /// `true`: the response has the `aws-waf-token` cookie.
    #[serde(rename = "cookieSolution", skip_serializing_if = "Option::is_none")]
    pub cookie_solution: Option<bool>,
}

impl AmazonWafRequest {
    pub fn new(website_url: impl Into<String>, variant: impl Into<AmazonWafVariant>) -> Self {
        Self {
            website_url: website_url.into(),
            variant: variant.into(),
            proxy: None,
            cookie_solution: None,
        }
    }

    pub fn with_proxy(mut self, proxy: ProxyInfo) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_cookie_solution(mut self, value: bool) -> Self {
        self.cookie_solution = Some(value);
        self
    }
}

impl CaptchaRequest for AmazonWafRequest {
    const TASK_TYPE: &'static str = "AmazonTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 10.0, 3.0, 180.0)
    }

    fn validate(&self) -> crate::error::Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        match &self.variant {
            AmazonWafVariant::VisibleCaptcha(v) => {
                require_non_empty("websiteKey", &v.website_key)?;
                require_non_empty("captchaScript", &v.captcha_script)?;
            }
            AmazonWafVariant::ChallengeAndCaptcha(v) => {
                require_non_empty("challengeScript", &v.challenge_script)?;
                require_non_empty("websiteKey", &v.website_key)?;
                require_non_empty("context", &v.context)?;
                require_non_empty("iv", &v.iv)?;
            }
            // `context` and `iv` are sent as "" by design for this variant.
            AmazonWafVariant::ChallengeOnly(v) => {
                require_non_empty("challengeScript", &v.challenge_script)?;
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

    #[test]
    fn option1_minimal() {
        let task = AmazonWafRequest::new(
            "https://example.com",
            AmazonVisibleCaptcha::new("KEY", "https://x/jsapi.js"),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({"type": "AmazonTask", "websiteURL": "https://example.com",
                   "websiteKey": "KEY", "captchaScript": "https://x/jsapi.js"})
        );
    }

    #[test]
    fn option1_full() {
        let task = AmazonWafRequest::new(
            "https://example.com",
            AmazonVisibleCaptcha::new("KEY", "https://x/jsapi.js").with_user_agent("UA"),
        )
        .with_proxy(ProxyInfo::new(ProxyType::Http, "1.2.3.4", 80))
        .with_cookie_solution(true)
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({"type": "AmazonTask", "websiteURL": "https://example.com",
                   "websiteKey": "KEY", "captchaScript": "https://x/jsapi.js",
                   "userAgent": "UA", "cookieSolution": true,
                   "proxyType": "http", "proxyAddress": "1.2.3.4", "proxyPort": 80})
        );
    }

    #[test]
    fn option2_minimal_and_full() {
        let minimal = AmazonWafRequest::new(
            "https://example.com",
            AmazonChallengeAndCaptcha::new("https://x/challenge.js", "K", "C", "IV"),
        )
        .to_task()
        .unwrap();
        assert_eq!(
            minimal,
            json!({"type": "AmazonTask", "websiteURL": "https://example.com",
                   "challengeScript": "https://x/challenge.js", "websiteKey": "K",
                   "context": "C", "iv": "IV"})
        );

        let full = AmazonWafRequest::new(
            "https://example.com",
            AmazonChallengeAndCaptcha::new("https://x/challenge.js", "K", "C", "IV")
                .with_captcha_script("https://x/captcha.js"),
        )
        .with_cookie_solution(false)
        .to_task()
        .unwrap();
        assert_eq!(
            full,
            json!({"type": "AmazonTask", "websiteURL": "https://example.com",
                   "challengeScript": "https://x/challenge.js", "websiteKey": "K",
                   "context": "C", "iv": "IV", "captchaScript": "https://x/captcha.js",
                   "cookieSolution": false})
        );
    }

    #[test]
    fn option3_sends_empty_context_and_iv() {
        let task = AmazonWafRequest::new(
            "https://example.com",
            AmazonChallengeOnly::new("https://x/challenge.js"),
        )
        .with_cookie_solution(true)
        .to_task()
        .unwrap();
        assert_eq!(
            task,
            json!({"type": "AmazonTask", "websiteURL": "https://example.com",
                   "challengeScript": "https://x/challenge.js", "context": "", "iv": "",
                   "cookieSolution": true})
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = AmazonWafRequest::new("", AmazonChallengeOnly::new("https://x/challenge.js"))
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = AmazonWafRequest::new("https://example.com", AmazonVisibleCaptcha::new("", "S"))
            .to_task()
            .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
        let err = AmazonWafRequest::new("https://example.com", AmazonVisibleCaptcha::new("K", ""))
            .to_task()
            .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("captchaScript"))
        );
        let err = AmazonWafRequest::new(
            "https://example.com",
            AmazonChallengeAndCaptcha::new("", "K", "C", "IV"),
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("challengeScript"))
        );
        let err = AmazonWafRequest::new(
            "https://example.com",
            AmazonChallengeAndCaptcha::new("S", "", "C", "IV"),
        )
        .to_task()
        .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteKey")));
        let err = AmazonWafRequest::new(
            "https://example.com",
            AmazonChallengeAndCaptcha::new("S", "K", "", "IV"),
        )
        .to_task()
        .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("context")));
        let err = AmazonWafRequest::new(
            "https://example.com",
            AmazonChallengeAndCaptcha::new("S", "K", "C", ""),
        )
        .to_task()
        .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("iv")));
        let err = AmazonWafRequest::new("https://example.com", AmazonChallengeOnly::new(""))
            .to_task()
            .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("challengeScript"))
        );
    }
}
