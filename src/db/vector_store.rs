// Neptune Lens - Vector store with brute-force cosine similarity search
// Persistent storage via bincode serialization
// For 50k images with 512-dim vectors, brute-force search takes ~20-50ms

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::metadata::ImageMetadata;

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: usize,
    pub path: PathBuf,
    pub distance: f32,
    pub similarity: f32,
}

/// Stored vector entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorEntry {
    id: usize,
    embedding: Vec<f32>,
}

/// Vector store using brute-force cosine similarity search
/// with bincode persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStore {
    entries: Vec<VectorEntry>,
    next_id: usize,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore {
    /// Create a new empty vector store
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }

    /// Insert a single embedding with its ID
    pub fn insert(&mut self, id: usize, embedding: &[f32]) {
        self.entries.push(VectorEntry {
            id,
            embedding: embedding.to_vec(),
        });
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }

    /// Get the next available ID
    pub fn next_id(&self) -> usize {
        self.next_id
    }

    /// Search for the top-N most similar embeddings using cosine similarity
    pub fn search(
        &self,
        query: &[f32],
        top_n: usize,
        metadata: &ImageMetadata,
    ) -> Vec<SearchResult> {
        if self.entries.is_empty() {
            return Vec::new();
        }

        // Compute cosine similarity in parallel
        let mut scored: Vec<(usize, f32)> = self
            .entries
            .par_iter()
            .map(|entry| {
                let sim = cosine_similarity(query, &entry.embedding);
                (entry.id, sim)
            })
            .collect();

        // Sort by similarity descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top-N and resolve paths
        scored
            .into_iter()
            .take(top_n)
            .filter_map(|(id, similarity)| {
                metadata.get_path(id).map(|path| SearchResult {
                    id,
                    path,
                    distance: 1.0 - similarity,
                    similarity,
                })
            })
            .collect()
    }

    /// Save the vector store to a binary file
    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;

        let path = dir.join("vectors.bin");
        let data = bincode::serialize(self)
            .with_context(|| "Failed to serialize vector store")?;

        std::fs::write(&path, data)
            .with_context(|| "Failed to write vector store")?;

        log::info!("Vector store saved ({} vectors)", self.entries.len());
        Ok(())
    }

    /// Load a vector store from a binary file
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join("vectors.bin");

        if !path.exists() {
            log::info!("No vector store found, creating new one");
            return Ok(Self::new());
        }

        let data = std::fs::read(&path)
            .with_context(|| "Failed to read vector store")?;

        let store: Self = bincode::deserialize(&data)
            .with_context(|| "Failed to deserialize vector store")?;

        log::info!("Loaded vector store with {} vectors", store.entries.len());
        Ok(store)
    }

    /// Get the number of stored vectors
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all vectors
    pub fn clear(&mut self) {
        self.entries.clear();
        self.next_id = 0;
    }
}

/// Compute cosine similarity between two vectors
/// Both vectors should be L2-normalized, so this is just a dot product
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_store_search() {
        let mut store = VectorStore::new();
        let mut meta = ImageMetadata::new();

        // Insert some test vectors
        store.insert(0, &[1.0, 0.0, 0.0]);
        meta.insert(0, PathBuf::from("a.jpg"));

        store.insert(1, &[0.9, 0.1, 0.0]);
        meta.insert(1, PathBuf::from("b.jpg"));

        store.insert(2, &[0.0, 1.0, 0.0]);
        meta.insert(2, PathBuf::from("c.jpg"));

        let results = store.search(&[1.0, 0.0, 0.0], 2, &meta);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, 0); // Most similar
    }
}
