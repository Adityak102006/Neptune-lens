// Neptune Lens - CLIP ONNX embedding generator using tract
// Pure Rust ONNX inference - no native library dependencies

use anyhow::{Context, Result};
use ndarray::Array4;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use tract_onnx::prelude::*;

/// CLIP embedding dimension for ViT-B-32
pub const EMBEDDING_DIM: usize = 512;

/// Type alias for the optimized tract model
type TractModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Thread-safe CLIP embedder wrapping a tract inference model.
pub struct ClipEmbedder {
    model: Arc<Mutex<TractModel>>,
}

impl Clone for ClipEmbedder {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
        }
    }
}

impl ClipEmbedder {
    /// Create a new ClipEmbedder by loading the ONNX model from the given path.
    pub fn new(model_path: &Path) -> Result<Self> {
        log::info!("Loading CLIP model from: {}", model_path.display());

        let model = tract_onnx::onnx()
            // Load the ONNX model
            .model_for_path(model_path)
            .with_context(|| format!("Failed to load ONNX model: {}", model_path.display()))?
            // Set input shape: [1, 3, 224, 224] float32
            .with_input_fact(0, f32::fact([1, 3, 224, 224]).into())
            .with_context(|| "Failed to set input fact")?
            // Optimize the model for inference
            .into_optimized()
            .with_context(|| "Failed to optimize model")?
            // Create a runnable plan
            .into_runnable()
            .with_context(|| "Failed to create runnable plan")?;

        log::info!("CLIP model loaded and optimized successfully");

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    /// Generate a normalized embedding vector for a preprocessed image tensor.
    /// Input: NCHW tensor [1, 3, 224, 224]
    /// Output: L2-normalized 512-dim vector
    pub fn embed(&self, input_tensor: Array4<f32>) -> Result<Vec<f32>> {
        // Convert ndarray to tract tensor
        let input: Tensor = input_tensor.into();

        let model = self.model.lock();
        let outputs = model
            .run(tvec!(input.into()))
            .with_context(|| "CLIP inference failed")?;

        // Extract the output tensor
        let output = &outputs[0];
        let output_view = output
            .to_array_view::<f32>()
            .with_context(|| "Failed to extract output tensor")?;

        let embedding: Vec<f32> = output_view.iter().copied().collect();

        // L2 normalize the embedding
        Ok(l2_normalize(&embedding))
    }

    /// Generate an embedding directly from an image file path
    pub fn embed_image(&self, image_path: &Path) -> Result<Vec<f32>> {
        let tensor = super::preprocessor::preprocess_image(image_path)?;
        self.embed(tensor)
    }
}

/// L2 normalize a vector
fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter().map(|x| x / norm).collect()
    } else {
        v.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_normalize() {
        let v = vec![3.0, 4.0];
        let normalized = l2_normalize(&v);
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);

        let length: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((length - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_l2_normalize_zero() {
        let v = vec![0.0, 0.0, 0.0];
        let normalized = l2_normalize(&v);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }
}
