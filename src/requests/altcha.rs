use serde::Serialize;

use super::{require_non_empty, CaptchaRequest};
use crate::error::Result;
use crate::proxy::ProxyInfo;
use crate::timeouts::GetResultTimeouts;

/// Altcha challenge parameters taken from the page (all four are required by the API).
#[derive(Debug, Clone, Serialize)]
pub struct AltchaMetadata {
    pub challenge: String,
    pub iterations: String,
    pub salt: String,
    pub signature: String,
}

/// Altcha proof-of-work captcha (`CustomTask`, class `altcha`).
#[derive(Debug, Clone, Serialize)]
pub struct AltchaCustomTaskRequest {
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Site key; an empty string is allowed.
    #[serde(rename = "websiteKey")]
    pub website_key: String,
    pub metadata: AltchaMetadata,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyInfo>,
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl AltchaCustomTaskRequest {
    pub fn new(
        website_url: impl Into<String>,
        website_key: impl Into<String>,
        metadata: AltchaMetadata,
    ) -> Self {
        Self {
            website_url: website_url.into(),
            website_key: website_key.into(),
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

impl CaptchaRequest for AltchaCustomTaskRequest {
    const TASK_TYPE: &'static str = "CustomTask";
    const CLASS: Option<&'static str> = Some("altcha");

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(1.0, 0.0, 1.0, 50.0)
    }

    fn validate(&self) -> Result<()> {
        require_non_empty("websiteURL", &self.website_url)?;
        require_non_empty("metadata.challenge", &self.metadata.challenge)?;
        require_non_empty("metadata.iterations", &self.metadata.iterations)?;
        require_non_empty("metadata.salt", &self.metadata.salt)?;
        require_non_empty("metadata.signature", &self.metadata.signature)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn task_has_class_and_metadata() {
        let meta = AltchaMetadata {
            challenge: "c".into(),
            iterations: "1".into(),
            salt: "s".into(),
            signature: "g".into(),
        };
        let task = AltchaCustomTaskRequest::new("https://example.com", "", meta)
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({
                "type": "CustomTask", "class": "altcha",
                "websiteURL": "https://example.com", "websiteKey": "",
                "metadata": {"challenge": "c", "iterations": "1", "salt": "s", "signature": "g"}
            })
        );
    }

    #[test]
    fn empty_required_strings_are_rejected() {
        let err = AltchaCustomTaskRequest::new(
            "",
            "",
            AltchaMetadata {
                challenge: "c".into(),
                iterations: "1".into(),
                salt: "s".into(),
                signature: "g".into(),
            },
        )
        .to_task()
        .unwrap_err();
        assert!(matches!(err, crate::error::Error::Validation(ref m) if m.contains("websiteURL")));
        let err = AltchaCustomTaskRequest::new(
            "https://example.com",
            "",
            AltchaMetadata {
                challenge: "".into(),
                iterations: "1".into(),
                salt: "s".into(),
                signature: "g".into(),
            },
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.challenge"))
        );
        let err = AltchaCustomTaskRequest::new(
            "https://example.com",
            "",
            AltchaMetadata {
                challenge: "c".into(),
                iterations: "".into(),
                salt: "s".into(),
                signature: "g".into(),
            },
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.iterations"))
        );
        let err = AltchaCustomTaskRequest::new(
            "https://example.com",
            "",
            AltchaMetadata {
                challenge: "c".into(),
                iterations: "1".into(),
                salt: "".into(),
                signature: "g".into(),
            },
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.salt"))
        );
        let err = AltchaCustomTaskRequest::new(
            "https://example.com",
            "",
            AltchaMetadata {
                challenge: "c".into(),
                iterations: "1".into(),
                salt: "s".into(),
                signature: "".into(),
            },
        )
        .to_task()
        .unwrap_err();
        assert!(
            matches!(err, crate::error::Error::Validation(ref m) if m.contains("metadata.signature"))
        );
    }
}
