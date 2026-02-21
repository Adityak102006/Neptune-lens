// Neptune Lens - Image preprocessing for CLIP model
// Resize images to 224x224, normalize with CLIP mean/std, output NCHW tensor

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, imageops::FilterType};
use ndarray::Array4;
use std::path::Path;

// CLIP normalization constants
const CLIP_MEAN: [f32; 3] = [0.48145466, 0.4578275, 0.40821073];
const CLIP_STD: [f32; 3] = [0.26862954, 0.26130258, 0.27577711];
const INPUT_SIZE: u32 = 224;

/// Load an image from disk, resize to 224x224, and convert to a normalized NCHW tensor.
pub fn preprocess_image(path: &Path) -> Result<Array4<f32>> {
    let img = image::open(path)
        .with_context(|| format!("Failed to open image: {}", path.display()))?;

    preprocess_dynamic_image(&img)
}

/// Preprocess a DynamicImage into a CLIP-compatible tensor.
pub fn preprocess_dynamic_image(img: &DynamicImage) -> Result<Array4<f32>> {
    // Resize to 224x224 using Lanczos3 for quality
    let resized = img.resize_exact(INPUT_SIZE, INPUT_SIZE, FilterType::Lanczos3);
    let rgb = resized.to_rgb8();

    // Create NCHW tensor [1, 3, 224, 224]
    let mut tensor = Array4::<f32>::zeros((1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize));

    for (x, y, pixel) in rgb.enumerate_pixels() {
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;

        // Normalize with CLIP mean/std
        tensor[[0, 0, y as usize, x as usize]] = (r - CLIP_MEAN[0]) / CLIP_STD[0];
        tensor[[0, 1, y as usize, x as usize]] = (g - CLIP_MEAN[1]) / CLIP_STD[1];
        tensor[[0, 2, y as usize, x as usize]] = (b - CLIP_MEAN[2]) / CLIP_STD[2];
    }

    Ok(tensor)
}

/// Load an image and return it as a DynamicImage.
pub fn load_image(path: &Path) -> Result<DynamicImage> {
    image::open(path).with_context(|| format!("Failed to open image: {}", path.display()))
}

/// Check if a file extension is a supported image format.
pub fn is_supported_image(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "webp" | "bmp"
        ),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_image(Path::new("test.jpg")));
        assert!(is_supported_image(Path::new("test.JPEG")));
        assert!(is_supported_image(Path::new("test.png")));
        assert!(is_supported_image(Path::new("test.webp")));
        assert!(is_supported_image(Path::new("test.bmp")));
        assert!(!is_supported_image(Path::new("test.gif")));
        assert!(!is_supported_image(Path::new("test.txt")));
        assert!(!is_supported_image(Path::new("no_extension")));
    }
}
