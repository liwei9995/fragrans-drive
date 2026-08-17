use crate::api::error::AppError;
use image::{ImageFormat, ImageReader};
use std::io::Cursor;

pub fn generate_thumbnail(data: &[u8]) -> Result<Vec<u8>, AppError> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| AppError::BadRequest(format!("Invalid image format: {}", e)))?;

    let dimensions = reader
        .into_dimensions()
        .map_err(|e| AppError::BadRequest(format!("Cannot read dimensions: {}", e)))?;
    if dimensions.0 > 20000 || dimensions.1 > 20000 {
        return Err(AppError::BadRequest(
            "Image dimensions exceed 20000 pixels".into(),
        ));
    }
    if (dimensions.0 as u64) * (dimensions.1 as u64) > 100_000_000 {
        return Err(AppError::BadRequest(
            "Image exceeds 100,000,000 pixels".into(),
        ));
    }

    let img = image::load_from_memory(data)
        .map_err(|e| AppError::BadRequest(format!("Image load failed: {}", e)))?;
    let thumb = img.thumbnail(200, 200);

    let mut buffer = Vec::new();
    // Default to WebP or JPEG? Legacy used image-thumbnail which usually outputs same format or JPEG.
    // Let's use JPEG for compatibility.
    thumb
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Jpeg)
        .map_err(|e| AppError::InternalError(format!("Image encode failed: {}", e)))?;

    Ok(buffer)
}
