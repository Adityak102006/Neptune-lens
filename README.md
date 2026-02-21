# 🔭 Neptune Lens

A **local Google Lens–style image search engine** built entirely in Rust. Index your local image folders, generate CLIP embeddings, and find visually similar images instantly — all offline.

![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Platform](https://img.shields.io/badge/Platform-Windows-0078D4?logo=windows)

## ✨ Features

- **🔍 Visual Similarity Search** — Select a query image and find the most visually similar images in your indexed folders
- **📂 Recursive Folder Indexing** — Supports JPG, PNG, WEBP, BMP formats
- **🧠 CLIP ViT-B-32 Embeddings** — State-of-the-art visual embeddings via pure Rust ONNX inference
- **⚡ Fast Search** — Cosine similarity on 512-dim vectors, handles 50k+ images
- **💾 Persistent Index** — Saves between sessions, no re-indexing needed
- **🖥️ Native Desktop UI** — Built with egui, instant and responsive
- **🔒 Fully Offline** — No cloud, no API keys, everything runs locally

## 🚀 Quick Start

### Download
1. Go to [**Releases**](https://github.com/Adityak102006/Neptune-lens/releases)
2. Download `neptune-lens.exe`
3. Download the CLIP model (`clip-vit-b32-vision.onnx`) from the release assets
4. Place the model in a `models/` folder next to the exe:
   ```
   📁 your-folder/
   ├── neptune-lens.exe
   └── models/
       └── clip-vit-b32-vision.onnx
   ```
5. Run `neptune-lens.exe`

### Build from Source
```bash
# Clone the repo
git clone https://github.com/Adityak102006/Neptune-lens.git
cd Neptune-lens

# Build (requires Rust toolchain)
cargo build --release

# Download the CLIP model
# Place clip-vit-b32-vision.onnx in target/release/models/

# Run
cargo run --release
```

## 📖 How to Use

1. **Index tab** → Click "Add Folder" → Select your image folder → "Start Indexing"
2. Wait for indexing to complete (progress bar shows status)
3. **Search tab** → Click "Select Query Image" → Pick any image → "Search"
4. View results as thumbnail cards with similarity percentages
5. Click "Open" on any result to view it in your default image viewer

## 🛠️ Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust |
| GUI | egui / eframe |
| ML Inference | tract-onnx (pure Rust) |
| CLIP Model | ViT-B-32 vision encoder |
| Image Processing | image crate |
| Parallelism | rayon |
| Persistence | bincode + serde |

## 📂 Project Structure

```
src/
├── main.rs              # Entry point
├── app.rs               # Application state & orchestration
├── engine/
│   ├── preprocessor.rs  # Image resize & CLIP normalization
│   ├── embedder.rs      # ONNX model loading & inference
│   └── indexer.rs       # Parallel folder scanning & embedding
├── db/
│   ├── vector_store.rs  # Cosine similarity search & persistence
│   └── metadata.rs      # File path ↔ vector ID mapping
└── ui/
    ├── indexing_panel.rs # Folder management & progress
    ├── search_panel.rs   # Query image selection
    ├── results_grid.rs   # Thumbnail results display
    └── settings_panel.rs # Configuration & stats
```

## 📝 License

MIT License — feel free to use, modify, and distribute.
