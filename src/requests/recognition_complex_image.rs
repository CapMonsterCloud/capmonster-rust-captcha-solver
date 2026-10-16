use serde::Serialize;

use super::complex_image::timeouts;
use super::CaptchaRequest;
use crate::error::{Error, Result};
use crate::timeouts::GetResultTimeouts;

/// Kind of data sent in `imagesBase64` (`PayloadType` inside `metadata`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PayloadType {
    /// The array holds base64-encoded audio (used by `bills_audio`).
    Audio,
}

/// `metadata` of a recognition task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecognitionMetadata {
    /// Name of the recognition module. Modules documented today: `shein`, `MathSum`, `dli`,
    /// `bls_3x3` (needs `task_argument`), `baidu`, `betpunch_3x3_rotate`, `bills_audio` (needs
    /// `payload_type = Audio`), `oocl_rotate_new`, `oocl_rotate_double_new`,
    /// `portugal_text_find_icon` (needs `task_argument`).
    #[serde(rename = "Task")]
    pub task: String,
    /// Module argument, e.g. the number to find for `bls_3x3` (`"123"`) or the instruction text
    /// for `portugal_text_find_icon` (non-ASCII characters escaped as `ã`).
    #[serde(rename = "TaskArgument", skip_serializing_if = "Option::is_none")]
    pub task_argument: Option<String>,
    /// Type of the payload; only needed for audio modules.
    #[serde(rename = "PayloadType", skip_serializing_if = "Option::is_none")]
    pub payload_type: Option<PayloadType>,
}

impl RecognitionMetadata {
    pub fn new(task: impl Into<String>) -> Self {
        Self {
            task: task.into(),
            task_argument: None,
            payload_type: None,
        }
    }

    pub fn with_task_argument(mut self, value: impl Into<String>) -> Self {
        self.task_argument = Some(value.into());
        self
    }

    pub fn with_payload_type(mut self, value: PayloadType) -> Self {
        self.payload_type = Some(value);
        self
    }
}

/// Recognition modules of `ComplexImageTask` (class `recognition`), no proxy needed.
///
/// The concrete module is chosen by `metadata.Task`. Modules documented today, with their
/// answer kind: `shein` (click coordinates), `MathSum` (text), `dli` (text), `bls_3x3`
/// (grid, needs `TaskArgument`), `baidu`, `oocl_rotate_new`, `oocl_rotate_double_new`,
/// `betpunch_3x3_rotate` (rotation angles), `portugal_text_find_icon` (needs `TaskArgument`),
/// `bills_audio` (audio, `PayloadType = Audio`). Most modules take exactly one image.
#[derive(Debug, Clone, Serialize)]
pub struct RecognitionComplexImageTaskRequest {
    /// Base64-encoded images (without a `data:...;base64,` prefix); at least one.
    #[serde(rename = "imagesBase64")]
    pub images_base64: Vec<String>,
    /// Module name and arguments.
    pub metadata: RecognitionMetadata,
    /// User-Agent (optional).
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

impl RecognitionComplexImageTaskRequest {
    pub fn new(images_base64: Vec<String>, metadata: RecognitionMetadata) -> Self {
        Self {
            images_base64,
            metadata,
            user_agent: None,
        }
    }

    /// Shortcut for the common single-image case.
    pub fn single(image_base64: impl Into<String>, metadata: RecognitionMetadata) -> Self {
        Self::new(vec![image_base64.into()], metadata)
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }
}

impl CaptchaRequest for RecognitionComplexImageTaskRequest {
    const TASK_TYPE: &'static str = "ComplexImageTask";
    const CLASS: Option<&'static str> = Some("recognition");

    fn timeouts(&self) -> GetResultTimeouts {
        timeouts()
    }

    fn validate(&self) -> Result<()> {
        if self.metadata.task.is_empty() {
            return Err(Error::Validation("metadata: Task must be defined".into()));
        }
        if self.images_base64.is_empty() {
            return Err(Error::Validation(
                "images_base64: at least one image expected".into(),
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
    fn minimal_task() {
        let task =
            RecognitionComplexImageTaskRequest::single("AAA", RecognitionMetadata::new("shein"))
                .to_task()
                .unwrap();
        assert_eq!(
            task,
            json!({"type": "ComplexImageTask", "class": "recognition",
                   "imagesBase64": ["AAA"], "metadata": {"Task": "shein"}})
        );
    }

    #[test]
    fn full_task() {
        let meta = RecognitionMetadata::new("bills_audio")
            .with_task_argument("123")
            .with_payload_type(PayloadType::Audio);
        let task = RecognitionComplexImageTaskRequest::new(vec!["A".into(), "B".into()], meta)
            .with_user_agent("UA")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "ComplexImageTask", "class": "recognition",
                   "imagesBase64": ["A", "B"],
                   "metadata": {"Task": "bills_audio", "TaskArgument": "123", "PayloadType": "Audio"},
                   "userAgent": "UA"})
        );
    }

    #[test]
    fn validation_errors() {
        let no_images =
            RecognitionComplexImageTaskRequest::new(vec![], RecognitionMetadata::new("shein"));
        assert!(matches!(no_images.to_task(), Err(Error::Validation(_))));
        let no_task = RecognitionComplexImageTaskRequest::single("A", RecognitionMetadata::new(""));
        assert!(matches!(no_task.to_task(), Err(Error::Validation(_))));
    }
}
