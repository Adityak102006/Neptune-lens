// Neptune Lens - Recursive folder indexer with parallel embedding generation
// Scans folders for images, generates embeddings in parallel with Rayon

use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use super::embedder::ClipEmbedder;
use super::preprocessor::is_supported_image;

/// Progress state for indexing operations
#[derive(Clone)]
pub struct IndexProgress {
    pub total: Arc<AtomicUsize>,
    pub completed: Arc<AtomicUsize>,
    pub failed: Arc<AtomicUsize>,
    pub cancelled: Arc<AtomicBool>,
}

impl Default for IndexProgress {
    fn default() -> Self {
        Self {
            total: Arc::new(AtomicUsize::new(0)),
            completed: Arc::new(AtomicUsize::new(0)),
            failed: Arc::new(AtomicUsize::new(0)),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl IndexProgress {
    pub fn reset(&self) {
        self.total.store(0, Ordering::Relaxed);
        self.completed.store(0, Ordering::Relaxed);
        self.failed.store(0, Ordering::Relaxed);
        self.cancelled.store(false, Ordering::Relaxed);
    }

    pub fn get_total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }

    pub fn get_completed(&self) -> usize {
        self.completed.load(Ordering::Relaxed)
    }

    pub fn get_failed(&self) -> usize {
        self.failed.load(Ordering::Relaxed)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

/// Result from indexing a single image
pub struct IndexedImage {
    pub path: PathBuf,
    pub embedding: Vec<f32>,
}

/// Scan a directory recursively for supported image files.
pub fn scan_folder(folder: &Path) -> Vec<PathBuf> {
    WalkDir::new(folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_supported_image(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Index a list of image paths in parallel using Rayon.
/// Returns successfully indexed images with their embeddings.
pub fn index_images_parallel(
    image_paths: &[PathBuf],
    embedder: &ClipEmbedder,
    progress: &IndexProgress,
    already_indexed: &std::collections::HashSet<PathBuf>,
) -> Vec<IndexedImage> {
    // Filter out already indexed files
    let new_paths: Vec<&PathBuf> = image_paths
        .iter()
        .filter(|p| !already_indexed.contains(*p))
        .collect();

    progress.total.store(new_paths.len(), Ordering::Relaxed);
    progress.completed.store(0, Ordering::Relaxed);
    progress.failed.store(0, Ordering::Relaxed);

    if new_paths.is_empty() {
        return Vec::new();
    }

    log::info!("Indexing {} new images...", new_paths.len());

    let results: Vec<Option<IndexedImage>> = new_paths
        .par_iter()
        .map(|path| {
            if progress.is_cancelled() {
                return None;
            }

            match embedder.embed_image(path) {
                Ok(embedding) => {
                    progress.completed.fetch_add(1, Ordering::Relaxed);
                    Some(IndexedImage {
                        path: path.to_path_buf(),
                        embedding,
                    })
                }
                Err(e) => {
                    log::warn!("Failed to embed {}: {}", path.display(), e);
                    progress.failed.fetch_add(1, Ordering::Relaxed);
                    progress.completed.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        })
        .collect();

    results.into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_index_progress() {
        let progress = IndexProgress::default();
        assert_eq!(progress.get_total(), 0);
        assert_eq!(progress.get_completed(), 0);
        assert!(!progress.is_cancelled());

        progress.total.store(100, Ordering::Relaxed);
        progress.completed.store(50, Ordering::Relaxed);
        assert_eq!(progress.get_total(), 100);
        assert_eq!(progress.get_completed(), 50);

        progress.cancel();
        assert!(progress.is_cancelled());

        progress.reset();
        assert_eq!(progress.get_total(), 0);
        assert!(!progress.is_cancelled());
    }
}
