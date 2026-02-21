// Neptune Lens - Metadata storage for mapping vector IDs to file paths
// Persistent JSON store with reverse lookup

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Persistent metadata mapping vector IDs to image file paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    /// Map from vector ID to image file path
    id_to_path: HashMap<usize, PathBuf>,
    /// Map from image file path to vector ID (reverse lookup)
    path_to_id: HashMap<PathBuf, usize>,
    /// List of indexed folders
    indexed_folders: Vec<PathBuf>,
}

impl Default for ImageMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageMetadata {
    pub fn new() -> Self {
        Self {
            id_to_path: HashMap::new(),
            path_to_id: HashMap::new(),
            indexed_folders: Vec::new(),
        }
    }

    /// Insert a mapping from ID to path
    pub fn insert(&mut self, id: usize, path: PathBuf) {
        self.path_to_id.insert(path.clone(), id);
        self.id_to_path.insert(id, path);
    }

    /// Get the file path for a vector ID
    pub fn get_path(&self, id: usize) -> Option<PathBuf> {
        self.id_to_path.get(&id).cloned()
    }

    /// Get the vector ID for a file path
    pub fn get_id(&self, path: &Path) -> Option<usize> {
        self.path_to_id.get(path).copied()
    }

    /// Check if a file is already indexed
    pub fn is_indexed(&self, path: &Path) -> bool {
        self.path_to_id.contains_key(path)
    }

    /// Get the set of all indexed file paths
    pub fn indexed_paths(&self) -> HashSet<PathBuf> {
        self.path_to_id.keys().cloned().collect()
    }

    /// Get the total number of indexed images
    pub fn len(&self) -> usize {
        self.id_to_path.len()
    }

    /// Check if metadata is empty
    pub fn is_empty(&self) -> bool {
        self.id_to_path.is_empty()
    }

    /// Add a folder to the indexed folders list
    pub fn add_folder(&mut self, folder: PathBuf) {
        if !self.indexed_folders.contains(&folder) {
            self.indexed_folders.push(folder);
        }
    }

    /// Remove a folder and all its images from the index
    pub fn remove_folder(&mut self, folder: &Path) {
        // Remove all images that belong to this folder
        let ids_to_remove: Vec<usize> = self
            .id_to_path
            .iter()
            .filter(|(_, path)| path.starts_with(folder))
            .map(|(id, _)| *id)
            .collect();

        for id in ids_to_remove {
            if let Some(path) = self.id_to_path.remove(&id) {
                self.path_to_id.remove(&path);
            }
        }

        self.indexed_folders.retain(|f| f != folder);
    }

    /// Get the list of indexed folders
    pub fn indexed_folders(&self) -> &[PathBuf] {
        &self.indexed_folders
    }

    /// Get image count per folder
    pub fn folder_image_counts(&self) -> HashMap<PathBuf, usize> {
        let mut counts = HashMap::new();
        for folder in &self.indexed_folders {
            let count = self
                .id_to_path
                .values()
                .filter(|p| p.starts_with(folder))
                .count();
            counts.insert(folder.clone(), count);
        }
        counts
    }

    /// Save metadata to a JSON file
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| "Failed to create metadata directory")?;
        }

        let json = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize metadata")?;

        std::fs::write(path, json).with_context(|| "Failed to write metadata file")?;

        log::info!("Metadata saved ({} entries)", self.len());
        Ok(())
    }

    /// Load metadata from a JSON file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            log::info!("No metadata file found, creating new one");
            return Ok(Self::new());
        }

        let json =
            std::fs::read_to_string(path).with_context(|| "Failed to read metadata file")?;

        let metadata: Self =
            serde_json::from_str(&json).with_context(|| "Failed to deserialize metadata")?;

        log::info!("Loaded metadata with {} entries", metadata.len());
        Ok(metadata)
    }

    /// Clear all metadata
    pub fn clear(&mut self) {
        self.id_to_path.clear();
        self.path_to_id.clear();
        self.indexed_folders.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_insert_and_lookup() {
        let mut meta = ImageMetadata::new();
        let path = PathBuf::from("C:\\test\\image.jpg");

        meta.insert(0, path.clone());

        assert_eq!(meta.get_path(0), Some(path.clone()));
        assert_eq!(meta.get_id(&path), Some(0));
        assert!(meta.is_indexed(&path));
        assert_eq!(meta.len(), 1);
    }

    #[test]
    fn test_metadata_remove_folder() {
        let mut meta = ImageMetadata::new();
        let folder = PathBuf::from("C:\\photos");
        meta.add_folder(folder.clone());

        meta.insert(0, PathBuf::from("C:\\photos\\a.jpg"));
        meta.insert(1, PathBuf::from("C:\\photos\\b.jpg"));
        meta.insert(2, PathBuf::from("C:\\other\\c.jpg"));

        meta.remove_folder(&folder);

        assert_eq!(meta.len(), 1);
        assert!(meta.get_path(2).is_some());
        assert!(meta.get_path(0).is_none());
    }
}
