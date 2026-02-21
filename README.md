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
3. Download the CLIP model (`clip-vit-b32-vision.onnx`) and place it in a `models/` folder next to the executable.

   **Option A: Manual Download**
   Download from [Hugging Face](https://huggingface.co/Xenova/clip-vit-base-patch32/tree/main/onnx) and place it directly in the `models/` folder.
   Alternatively, use `curl` (for tech-savvy users):
   ```bash
   mkdir -p models
   curl -L -o models/clip-vit-b32-vision.onnx https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/model.onnx
   ```

   **Option B: PowerShell (Quick Install)**
   Open PowerShell in the folder where `neptune-lens.exe` is located and run:
   ```powershell
   New-Item -ItemType Directory -Force -Path "models"; Invoke-WebRequest -Uri "https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/model.onnx" -OutFile "models\clip-vit-b32-vision.onnx"
   ```

   Your folder structure should look like this:
   ```text
   📁 your-folder/
   ├── neptune-lens.exe
   └── models/
       └── clip-vit-b32-vision.onnx
   ```
4. Run `neptune-lens.exe`

### Build from Source
```bash
# Clone the repo
git clone https://github.com/Adityak102006/Neptune-lens.git
cd Neptune-lens

# Build (requires Rust toolchain)
cargo build --release

# Download the CLIP model
mkdir -p target/release/models
curl -L -o target/release/models/clip-vit-b32-vision.onnx https://huggingface.co/Xenova/clip-vit-base-patch32/resolve/main/onnx/model.onnx

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
