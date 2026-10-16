//! One struct per captcha type. Every struct implements [`CaptchaRequest`].
//!
//! Conventions used by all request types (keep them when adding a new one):
//! * `Type::new(<required fields>)` builds a request; optional fields are `Option<_>` public
//!   fields with chainable `with_<field>(value)` setters.
//! * Field names are `snake_case`; `#[serde(rename = "...")]` maps them to the exact API names
//!   (note `websiteURL`, not `websiteUrl`). `None` fields are not sent.
//! * `"type"` (and `"class"` for CustomTask) come from `TASK_TYPE` / `CLASS`, never from a field.
//! * Local checks that the API would reject anyway live in `validate()`.

use serde::Serialize;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::timeouts::GetResultTimeouts;

mod alibaba;
mod altcha;
mod amazon_waf;
mod basilisk;
mod binance;
mod complex_image;
mod datadome;
mod friendly;
mod funcaptcha;
mod geetest;
mod hunt;
mod image_to_text;
mod imperva;
mod mt_captcha;
mod prosopo;
mod recaptcha_complex_image;
mod recaptcha_v2;
mod recaptcha_v2_enterprise;
mod recaptcha_v3;
mod recaptcha_v3_enterprise;
mod recognition_complex_image;
mod ten_di;
mod tspd;
mod turnstile;
mod yidun;

pub use alibaba::{AlibabaCustomTaskRequest, AlibabaMetadata};
pub use altcha::{AltchaCustomTaskRequest, AltchaMetadata};
pub use amazon_waf::{
    AmazonChallengeAndCaptcha, AmazonChallengeOnly, AmazonVisibleCaptcha, AmazonWafRequest,
    AmazonWafVariant,
};
pub use basilisk::BasiliskCustomTaskRequest;
pub use binance::BinanceTaskRequest;
pub use datadome::{DataDomeCustomTaskRequest, DataDomeMetadata, DataDomeVersion};
pub use friendly::{FriendlyCustomTaskRequest, FriendlyMetadata};
pub use funcaptcha::FuncaptchaRequest;
pub use geetest::{GeetestRequest, GeetestVersion};
pub use hunt::{HuntCustomTaskRequest, HuntMetadata, HuntMode};
pub use image_to_text::{ImageToTextRequest, TextModule};
pub use imperva::{ImpervaCustomTaskRequest, ImpervaMetadata};
pub use mt_captcha::MtCaptchaRequest;
pub use prosopo::ProsopoTaskRequest;
pub use recaptcha_complex_image::{
    RecaptchaComplexImageTaskRequest, RecaptchaGrid, RecaptchaMetadata,
};
pub use recaptcha_v2::RecaptchaV2Request;
pub use recaptcha_v2_enterprise::{EnterprisePayload, RecaptchaV2EnterpriseRequest};
pub use recaptcha_v3::RecaptchaV3ProxylessRequest;
pub use recaptcha_v3_enterprise::RecaptchaV3EnterpriseRequest;
pub use recognition_complex_image::{
    PayloadType, RecognitionComplexImageTaskRequest, RecognitionMetadata,
};
pub use ten_di::{TenDiCustomTaskRequest, TenDiMetadata};
pub use tspd::{TspdCustomTaskRequest, TspdMetadata};
pub use turnstile::{
    TurnstileChallengeCookie, TurnstileChallengeToken, TurnstileMode, TurnstileRequest,
    TurnstileStandard, TurnstileWaitRoom,
};
pub use yidun::YidunRequest;

/// Fails with [`Error::Validation`] when a required string field is empty.
pub(crate) fn require_non_empty(field: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::Validation(format!("{field} must not be empty")));
    }
    Ok(())
}

/// A task that can be sent to CapMonster Cloud.
pub trait CaptchaRequest: Serialize {
    /// Value of the `"type"` field of the task.
    const TASK_TYPE: &'static str;
    /// Value of the `"class"` field (only for `CustomTask`).
    const CLASS: Option<&'static str> = None;

    /// Polling schedule recommended for this captcha type.
    fn timeouts(&self) -> GetResultTimeouts;

    /// Extra consistency checks, run before the request is sent.
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// The `task` JSON object for `createTask`.
    fn to_task(&self) -> Result<Value> {
        self.validate()?;
        let mut task = serde_json::to_value(self)?;
        let obj = task
            .as_object_mut()
            .ok_or_else(|| Error::Validation("task must serialize to a JSON object".into()))?;
        obj.insert("type".into(), Self::TASK_TYPE.into());
        if let Some(class) = Self::CLASS {
            obj.insert("class".into(), class.into());
        }
        Ok(task)
    }
}
