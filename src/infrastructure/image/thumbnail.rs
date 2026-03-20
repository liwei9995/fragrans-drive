use image::ImageFormat;
use std::io::Cursor;

pub fn generate_thumbnail(data: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let img = image::load_from_memory(data)?;
    let thumb = img.thumbnail(200, 200);

    let mut buffer = Vec::new();
    // Default to WebP or JPEG? Legacy used image-thumbnail which usually outputs same format or JPEG.
    // Let's use JPEG for compatibility.
    thumb.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Jpeg)?;

    Ok(buffer)
}
