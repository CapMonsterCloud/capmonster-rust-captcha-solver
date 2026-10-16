//! Parts shared by the `ComplexImageTask` request types (classes `recaptcha`, `recognition`).

use crate::error::{Error, Result};
use crate::timeouts::GetResultTimeouts;

/// Polling schedule used for every `ComplexImageTask` class (same as Python).
pub(crate) fn timeouts() -> GetResultTimeouts {
    GetResultTimeouts::secs(0.35, 0.0, 0.2, 10.0)
}

/// Checks one optional image array: when given it must hold `1..=max` entries.
pub(crate) fn check_image_list(field: &str, list: Option<&Vec<String>>, max: usize) -> Result<()> {
    match list {
        Some(l) if l.is_empty() => Err(Error::Validation(format!(
            "{field}: at least one entry expected, got 0"
        ))),
        Some(l) if l.len() > max => Err(Error::Validation(format!(
            "{field}: at most {max} entries allowed, got {}",
            l.len()
        ))),
        _ => Ok(()),
    }
}

/// At least one of the two image arrays must be filled.
pub(crate) fn require_images(
    urls: Option<&Vec<String>>,
    base64: Option<&Vec<String>>,
) -> Result<()> {
    if urls.is_none() && base64.is_none() {
        return Err(Error::Validation(
            "at least one of image_urls / images_base64 must be set".into(),
        ));
    }
    Ok(())
}
