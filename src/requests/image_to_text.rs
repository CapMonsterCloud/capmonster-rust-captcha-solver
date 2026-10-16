use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Serialize, Serializer};

use super::CaptchaRequest;
use crate::error::{Error, Result};
use crate::timeouts::GetResultTimeouts;

/// Recognition modules accepted by `ImageToTextTask` (`capMonsterModule`), as listed in
/// <https://docs.capmonster.cloud/docs/captchas/ImageToText/module-name.txt>.
/// A module missing here (the list grows) can be passed with [`TextModule::Custom`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextModule {
    Amazon,
    Apple,
    Botdetect,
    Facebook,
    Gmx,
    Google,
    Hotmail,
    Mailru,
    Okeng,
    Okrus,
    Partiallyblur,
    Ramblerrus,
    Ramblerrusnew,
    Solvemedia,
    Steam,
    Vk,
    Whatsapp,
    WikimediaPaddleEnsemble,
    Yandex,
    /// Two-word Yandex captcha.
    Yandexnew,
    Yandexwave,
    /// Latin Yandex captcha.
    Yandexwavelatin,
    CaptchaMath,
    Sat,
    BlsText,
    /// All other text captchas.
    Universal,
    /// Any other module name (e.g. `special`).
    Custom(String),
}

impl TextModule {
    /// The module name as sent to the API.
    pub fn as_str(&self) -> &str {
        match self {
            TextModule::Amazon => "amazon",
            TextModule::Apple => "apple",
            TextModule::Botdetect => "botdetect",
            TextModule::Facebook => "facebook",
            TextModule::Gmx => "gmx",
            TextModule::Google => "google",
            TextModule::Hotmail => "hotmail",
            TextModule::Mailru => "mailru",
            TextModule::Okeng => "okeng",
            TextModule::Okrus => "okrus",
            TextModule::Partiallyblur => "partiallyblur",
            TextModule::Ramblerrus => "ramblerrus",
            TextModule::Ramblerrusnew => "ramblerrusnew",
            TextModule::Solvemedia => "solvemedia",
            TextModule::Steam => "steam",
            TextModule::Vk => "vk",
            TextModule::Whatsapp => "whatsapp",
            TextModule::WikimediaPaddleEnsemble => "wikimedia_paddle_ensemble",
            TextModule::Yandex => "yandex",
            TextModule::Yandexnew => "yandexnew",
            TextModule::Yandexwave => "yandexwave",
            TextModule::Yandexwavelatin => "yandexwavelatin",
            TextModule::CaptchaMath => "captcha_math",
            TextModule::Sat => "sat",
            TextModule::BlsText => "bls_text",
            TextModule::Universal => "universal",
            TextModule::Custom(name) => name,
        }
    }
}

impl Serialize for TextModule {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

fn as_base64<S: Serializer>(bytes: &[u8], s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(&STANDARD.encode(bytes))
}

/// Text-on-image captcha (`ImageToTextTask`).
#[derive(Debug, Clone, Serialize)]
pub struct ImageToTextRequest {
    /// Raw image bytes (PNG/JPEG/...); sent base64-encoded as `body`.
    #[serde(rename = "body", serialize_with = "as_base64")]
    pub image_bytes: Vec<u8>,
    #[serde(rename = "capMonsterModule", skip_serializing_if = "Option::is_none")]
    pub module_name: Option<TextModule>,
    /// Minimum recognition confidence, 0..=100.
    #[serde(
        rename = "recognizingThreshold",
        skip_serializing_if = "Option::is_none"
    )]
    pub threshold: Option<u8>,
    /// Case-sensitive text.
    #[serde(rename = "case", skip_serializing_if = "Option::is_none")]
    pub case: Option<bool>,
    /// Only digits (sent as 0/1).
    #[serde(
        serialize_with = "bool_as_int",
        skip_serializing_if = "Option::is_none"
    )]
    pub numeric: Option<bool>,
    /// The image contains an arithmetic expression to calculate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub math: Option<bool>,
}

fn bool_as_int<S: Serializer>(v: &Option<bool>, s: S) -> std::result::Result<S::Ok, S::Error> {
    match v {
        Some(b) => s.serialize_u8(*b as u8),
        None => s.serialize_none(),
    }
}

impl ImageToTextRequest {
    pub fn new(image_bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            image_bytes: image_bytes.into(),
            module_name: None,
            threshold: None,
            case: None,
            numeric: None,
            math: None,
        }
    }

    pub fn with_module_name(mut self, value: TextModule) -> Self {
        self.module_name = Some(value);
        self
    }

    pub fn with_threshold(mut self, value: u8) -> Self {
        self.threshold = Some(value);
        self
    }

    pub fn with_case(mut self, value: bool) -> Self {
        self.case = Some(value);
        self
    }

    pub fn with_numeric(mut self, value: bool) -> Self {
        self.numeric = Some(value);
        self
    }

    pub fn with_math(mut self, value: bool) -> Self {
        self.math = Some(value);
        self
    }
}

impl CaptchaRequest for ImageToTextRequest {
    const TASK_TYPE: &'static str = "ImageToTextTask";

    fn timeouts(&self) -> GetResultTimeouts {
        GetResultTimeouts::secs(0.35, 0.0, 0.2, 10.0)
    }

    fn validate(&self) -> Result<()> {
        if self.image_bytes.is_empty() {
            return Err(Error::Validation("body must not be empty".into()));
        }
        if matches!(self.threshold, Some(t) if t > 100) {
            return Err(Error::Validation(
                "threshold must be between 0 and 100".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn body_is_base64_and_numeric_is_int() {
        let task = ImageToTextRequest::new(b"abc".to_vec())
            .with_module_name(TextModule::Yandexnew)
            .with_threshold(90)
            .with_numeric(true)
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "ImageToTextTask", "body": "YWJj", "capMonsterModule": "yandexnew",
                   "recognizingThreshold": 90, "numeric": 1})
        );
    }

    #[test]
    fn threshold_out_of_range_is_rejected() {
        let err = ImageToTextRequest::new(vec![1])
            .with_threshold(101)
            .to_task()
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn empty_image_bytes_are_rejected() {
        let err = ImageToTextRequest::new(Vec::new()).to_task().unwrap_err();
        assert!(matches!(err, Error::Validation(ref m) if m.contains("body")));
    }
}
