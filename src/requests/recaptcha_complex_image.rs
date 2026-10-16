use serde::Serialize;

use super::complex_image::{check_image_list, require_images, timeouts};
use super::CaptchaRequest;
use crate::error::{Error, Result};
use crate::timeouts::GetResultTimeouts;

/// Image grid size of a reCAPTCHA challenge (`Grid` inside `metadata`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RecaptchaGrid {
    #[serde(rename = "3x3")]
    ThreeByThree,
    #[serde(rename = "4x4")]
    FourByFour,
    #[serde(rename = "1x1")]
    OneByOne,
}

/// `metadata` of the reCAPTCHA click task.
///
/// The docs require `Grid` and at least one of `Task` / `TaskDefinition`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecaptchaMetadata {
    /// Image grid size.
    #[serde(rename = "Grid")]
    pub grid: RecaptchaGrid,
    /// Task name in English, e.g. `Click on traffic lights`.
    #[serde(rename = "Task", skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    /// Technical task identifier, e.g. `/m/015qff` (taken from the `/recaptcha/<api>/reload` or
    /// `/userverify` response).
    #[serde(rename = "TaskDefinition", skip_serializing_if = "Option::is_none")]
    pub task_definition: Option<String>,
}

impl RecaptchaMetadata {
    /// Metadata identified by the task name.
    pub fn from_task(grid: RecaptchaGrid, task: impl Into<String>) -> Self {
        Self {
            grid,
            task: Some(task.into()),
            task_definition: None,
        }
    }

    /// Metadata identified by the technical `TaskDefinition` value.
    pub fn from_task_definition(grid: RecaptchaGrid, task_definition: impl Into<String>) -> Self {
        Self {
            grid,
            task: None,
            task_definition: Some(task_definition.into()),
        }
    }

    pub fn with_task(mut self, value: impl Into<String>) -> Self {
        self.task = Some(value.into());
        self
    }

    pub fn with_task_definition(mut self, value: impl Into<String>) -> Self {
        self.task_definition = Some(value.into());
        self
    }
}

/// reCAPTCHA v2 image click task (`ComplexImageTask`, class `recaptcha`). No proxy is needed.
///
/// Pass the full 4x4 / 3x3 image or a new 1x1 tile either by URL or as base64 (one entry max).
#[derive(Debug, Clone, Serialize)]
pub struct RecaptchaComplexImageTaskRequest {
    /// Image links (array of one); required if `images_base64` is not set.
    #[serde(rename = "imageUrls", skip_serializing_if = "Option::is_none")]
    pub image_urls: Option<Vec<String>>,
    /// Base64 images without the `data:image/jpeg;base64,` prefix (array of one); required if
    /// `image_urls` is not set.
    #[serde(rename = "imagesBase64", skip_serializing_if = "Option::is_none")]
    pub images_base64: Option<Vec<String>>,
    /// Grid and task description.
    pub metadata: RecaptchaMetadata,
    /// User-Agent to use when loading images passed as URLs.
    #[serde(rename = "userAgent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// URL of the page where the captcha is solved.
    #[serde(rename = "websiteURL", skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
}

impl RecaptchaComplexImageTaskRequest {
    /// Request with the image given by URL.
    pub fn from_url(image_url: impl Into<String>, metadata: RecaptchaMetadata) -> Self {
        Self {
            image_urls: Some(vec![image_url.into()]),
            images_base64: None,
            metadata,
            user_agent: None,
            website_url: None,
        }
    }

    /// Request with the image given as base64.
    pub fn from_base64(image_base64: impl Into<String>, metadata: RecaptchaMetadata) -> Self {
        Self {
            image_urls: None,
            images_base64: Some(vec![image_base64.into()]),
            metadata,
            user_agent: None,
            website_url: None,
        }
    }

    pub fn with_image_urls(mut self, value: Vec<String>) -> Self {
        self.image_urls = Some(value);
        self
    }

    pub fn with_images_base64(mut self, value: Vec<String>) -> Self {
        self.images_base64 = Some(value);
        self
    }

    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    pub fn with_website_url(mut self, value: impl Into<String>) -> Self {
        self.website_url = Some(value.into());
        self
    }
}

impl CaptchaRequest for RecaptchaComplexImageTaskRequest {
    const TASK_TYPE: &'static str = "ComplexImageTask";
    const CLASS: Option<&'static str> = Some("recaptcha");

    fn timeouts(&self) -> GetResultTimeouts {
        timeouts()
    }

    fn validate(&self) -> Result<()> {
        if self.metadata.task.is_none() && self.metadata.task_definition.is_none() {
            return Err(Error::Validation(
                "metadata: either Task or TaskDefinition must be set".into(),
            ));
        }
        require_images(self.image_urls.as_ref(), self.images_base64.as_ref())?;
        check_image_list("image_urls", self.image_urls.as_ref(), 1)?;
        check_image_list("images_base64", self.images_base64.as_ref(), 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn meta() -> RecaptchaMetadata {
        RecaptchaMetadata::from_task(RecaptchaGrid::ThreeByThree, "Click on traffic lights")
    }

    #[test]
    fn minimal_task() {
        let task = RecaptchaComplexImageTaskRequest::from_base64("AAA", meta())
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "ComplexImageTask", "class": "recaptcha", "imagesBase64": ["AAA"],
                   "metadata": {"Grid": "3x3", "Task": "Click on traffic lights"}})
        );
    }

    #[test]
    fn full_task() {
        let metadata =
            RecaptchaMetadata::from_task_definition(RecaptchaGrid::FourByFour, "/m/015qff")
                .with_task("Click on traffic lights");
        let task = RecaptchaComplexImageTaskRequest::from_url("https://i/x.jpg", metadata)
            .with_images_base64(vec!["B".into()])
            .with_user_agent("UA")
            .with_website_url("https://example.com")
            .to_task()
            .unwrap();
        assert_eq!(
            task,
            json!({"type": "ComplexImageTask", "class": "recaptcha",
                   "imageUrls": ["https://i/x.jpg"], "imagesBase64": ["B"],
                   "metadata": {"Grid": "4x4", "Task": "Click on traffic lights",
                                "TaskDefinition": "/m/015qff"},
                   "userAgent": "UA", "websiteURL": "https://example.com"})
        );
    }

    #[test]
    fn grid_serialization() {
        assert_eq!(
            serde_json::to_value(RecaptchaGrid::OneByOne).unwrap(),
            json!("1x1")
        );
    }

    #[test]
    fn metadata_needs_task_or_definition() {
        let mut m = meta();
        m.task = None;
        let err = RecaptchaComplexImageTaskRequest::from_base64("A", m)
            .to_task()
            .unwrap_err();
        assert!(matches!(err, Error::Validation(_)));
    }

    #[test]
    fn image_checks() {
        let mut r = RecaptchaComplexImageTaskRequest::from_base64("A", meta());
        r.images_base64 = None;
        assert!(r.to_task().is_err(), "no images");
        let r = RecaptchaComplexImageTaskRequest::from_base64("A", meta())
            .with_images_base64(vec!["A".into(), "B".into()]);
        assert!(r.to_task().is_err(), "more than one image");
        let r = RecaptchaComplexImageTaskRequest::from_base64("A", meta()).with_image_urls(vec![]);
        assert!(r.to_task().is_err(), "empty list");
    }
}
