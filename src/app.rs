// Neptune Lens - Main application state and egui orchestration
// Modern UI with sidebar navigation and dashboard

use crate::egui;
use crate::db::metadata::ImageMetadata;
use crate::db::vector_store::{SearchResult, VectorStore};
use crate::engine::embedder::ClipEmbedder;
use crate::engine::indexer::{self, IndexProgress};

use std::path::PathBuf;
use std::sync::mpsc;

/// Data directory for persistent storage
fn data_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("data")
}

/// Models directory
fn models_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("models")
}

/// Active tab in the UI
#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Collections,
    VisualSearch,
    IndexNew,
    Settings,
}

/// Messages from background threads to the main app
pub enum BackgroundMessage {
    IndexingComplete {
        images: Vec<(usize, PathBuf, Vec<f32>)>,
    },
    IndexingError(String),
    SearchComplete(Vec<SearchResult>),
    SearchError(String),
}

/// Shared application state accessible from UI panels
pub struct AppState {
    pub metadata: ImageMetadata,
    pub vector_store_len: usize,

    // Indexing
    pub pending_folders: Vec<PathBuf>,
    pub is_indexing: bool,
    pub start_indexing: bool,
    pub progress: IndexProgress,
    pub last_index_message: Option<String>,

    // Search
    pub query_image_path: Option<PathBuf>,
    pub search_results: Vec<SearchResult>,
    pub is_searching: bool,
    pub start_search: bool,

    // Settings
    pub top_n: usize,

    // Control flags
    pub needs_save: bool,
    pub clear_index: bool,

    // Navigation
    pub switch_tab: Option<Tab>,

    // Model status
    pub model_loaded: bool,
}

/// Main application struct
pub struct NeptuneLensApp {
    pub state: AppState,
    active_tab: Tab,
    embedder: Option<ClipEmbedder>,
    vector_store: VectorStore,
    rx: mpsc::Receiver<BackgroundMessage>,
    tx: mpsc::Sender<BackgroundMessage>,
    model_error: Option<String>,
}

// ─── Color palette ──────────────────────────────────────────────────
pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(11, 13, 18);
pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(16, 18, 24);
pub const BG_CARD: egui::Color32 = egui::Color32::from_rgb(22, 25, 33);
const BG_CARD_HOVER: egui::Color32 = egui::Color32::from_rgb(28, 31, 40);
pub const ACCENT_BLUE: egui::Color32 = egui::Color32::from_rgb(56, 120, 240);
pub const ACCENT_GREEN: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
pub const ACCENT_PURPLE: egui::Color32 = egui::Color32::from_rgb(139, 92, 246);
pub const ACCENT_YELLOW: egui::Color32 = egui::Color32::from_rgb(250, 204, 21);
pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(230, 235, 245);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(148, 158, 178);
const SIDEBAR_BG: egui::Color32 = egui::Color32::from_rgb(14, 16, 22);
const SIDEBAR_ACTIVE: egui::Color32 = egui::Color32::from_rgb(56, 120, 240);

impl NeptuneLensApp {
    #[allow(deprecated)]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders for egui
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Set up custom dark mode visuals
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG_DARK;
        visuals.window_fill = BG_CARD;
        visuals.widgets.noninteractive.bg_fill = BG_CARD;
        visuals.widgets.inactive.bg_fill = BG_CARD;
        visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
        visuals.widgets.active.bg_fill = ACCENT_BLUE;
        visuals.widgets.noninteractive.fg_stroke =
            egui::Stroke::new(1.0, TEXT_SECONDARY);
        visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(1.0, TEXT_SECONDARY);
        visuals.widgets.hovered.fg_stroke =
            egui::Stroke::new(1.0, TEXT_PRIMARY);
        visuals.widgets.active.fg_stroke =
            egui::Stroke::new(1.0, TEXT_PRIMARY);
        visuals.selection.bg_fill = ACCENT_BLUE;
        visuals.extreme_bg_color = BG_PANEL;
        visuals.faint_bg_color = BG_CARD;
        visuals.window_corner_radius = egui::CornerRadius::same(12);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
        cc.egui_ctx.set_visuals(visuals);

        let (tx, rx) = mpsc::channel();

        // Load persistent data
        let data = data_dir();
        let metadata = ImageMetadata::load(&data.join("metadata.json")).unwrap_or_default();
        let vector_store = VectorStore::load(&data).unwrap_or_else(|e| {
            log::warn!("Failed to load vector store: {}", e);
            VectorStore::new()
        });
        let vector_store_len = vector_store.len();

        // Try to load CLIP model
        let model_path = models_dir().join("clip-vit-b32-vision.onnx");
        let (embedder, model_error) = if model_path.exists() {
            match ClipEmbedder::new(&model_path) {
                Ok(e) => (Some(e), None),
                Err(e) => (None, Some(format!("Failed to load CLIP model: {}", e))),
            }
        } else {
            (
                None,
                Some(format!(
                    "CLIP model not found at: {}. Please download clip-vit-b32-vision.onnx from Hugging Face.",
                    model_path.display()
                )),
            )
        };

        let model_loaded = embedder.is_some();

        Self {
            state: AppState {
                metadata,
                vector_store_len,
                pending_folders: Vec::new(),
                is_indexing: false,
                start_indexing: false,
                progress: IndexProgress::default(),
                last_index_message: None,
                query_image_path: None,
                search_results: Vec::new(),
                is_searching: false,
                start_search: false,
                top_n: 20,
                needs_save: false,
                clear_index: false,
                switch_tab: None,
                model_loaded,
            },
            active_tab: Tab::Dashboard,
            embedder,
            vector_store,
            rx,
            tx,
            model_error,
        }
    }

    fn handle_background_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                BackgroundMessage::IndexingComplete { images } => {
                    let count = images.len();
                    for (id, path, embedding) in &images {
                        self.vector_store.insert(*id, embedding);
                        self.state.metadata.insert(*id, path.clone());
                    }
                    self.state.vector_store_len = self.vector_store.len();
                    self.state.is_indexing = false;
                    self.state.last_index_message =
                        Some(format!("✅ Successfully indexed {} images", count));
                    self.state.needs_save = true;
                }
                BackgroundMessage::IndexingError(err) => {
                    self.state.is_indexing = false;
                    self.state.last_index_message = Some(format!("❌ Error: {}", err));
                }
                BackgroundMessage::SearchComplete(results) => {
                    self.state.search_results = results;
                    self.state.is_searching = false;
                }
                BackgroundMessage::SearchError(err) => {
                    self.state.is_searching = false;
                    log::error!("Search error: {}", err);
                }
            }
        }
    }

    fn start_indexing(&mut self) {
        if self.embedder.is_none() {
            self.state.last_index_message =
                Some("❌ CLIP model not loaded. Check Settings.".to_string());
            return;
        }

        let folders: Vec<PathBuf> = self.state.pending_folders.drain(..).collect();
        if folders.is_empty() {
            return;
        }

        self.state.is_indexing = true;
        self.state.last_index_message = None;
        self.state.progress.reset();

        let embedder = self.embedder.clone().unwrap();
        let tx = self.tx.clone();
        let progress = self.state.progress.clone();
        let already_indexed = self.state.metadata.indexed_paths();
        let mut next_id = self.vector_store.next_id();

        let folder_list = folders.clone();
        for f in &folder_list {
            self.state.metadata.add_folder(f.clone());
        }

        std::thread::spawn(move || {
            let mut all_images = Vec::new();
            for folder in &folders {
                let images = indexer::scan_folder(folder);
                log::info!("Found {} images in {}", images.len(), folder.display());
                all_images.extend(images);
            }

            let indexed = indexer::index_images_parallel(
                &all_images,
                &embedder,
                &progress,
                &already_indexed,
            );

            let mut images_with_ids = Vec::new();
            for item in indexed {
                images_with_ids.push((next_id, item.path, item.embedding));
                next_id += 1;
            }

            let _ = tx.send(BackgroundMessage::IndexingComplete {
                images: images_with_ids,
            });
        });
    }

    fn start_search(&mut self) {
        let query_path = match &self.state.query_image_path {
            Some(p) => p.clone(),
            None => return,
        };

        if self.embedder.is_none() {
            return;
        }

        self.state.is_searching = true;
        self.state.search_results.clear();

        let embedder = self.embedder.clone().unwrap();
        let tx = self.tx.clone();
        let metadata = self.state.metadata.clone();
        let top_n = self.state.top_n;
        let store_clone = self.vector_store.clone();

        std::thread::spawn(move || {
            match embedder.embed_image(&query_path) {
                Ok(query_embedding) => {
                    let results = store_clone.search(&query_embedding, top_n, &metadata);
                    let _ = tx.send(BackgroundMessage::SearchComplete(results));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMessage::SearchError(format!(
                        "Failed to embed query image: {}",
                        e
                    )));
                }
            }
        });
    }

    fn save_data(&mut self) {
        let data = data_dir();
        if let Err(e) = self.state.metadata.save(&data.join("metadata.json")) {
            log::error!("Failed to save metadata: {}", e);
        }
        if let Err(e) = self.vector_store.save(&data) {
            log::error!("Failed to save vector store: {}", e);
        }
    }

    fn clear_index(&mut self) {
        self.state.metadata.clear();
        self.vector_store.clear();
        self.state.vector_store_len = 0;
        self.state.search_results.clear();
        self.state.last_index_message = Some("🗑 Index cleared".to_string());
        self.state.needs_save = true;
    }

    // ─── Sidebar rendering ──────────────────────────────────────────
    #[allow(deprecated)]
    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(180.0)
            .frame(
                egui::Frame::default()
                    .fill(SIDEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(10, 16)),
            )
            .show(ctx, |ui| {
                // ── Logo / Brand ────────────────────────────────
                ui.horizontal(|ui| {
                    ui.colored_label(
                        ACCENT_BLUE,
                        egui::RichText::new("🔭").size(22.0),
                    );
                    ui.vertical(|ui| {
                        ui.colored_label(
                            TEXT_PRIMARY,
                            egui::RichText::new("Neptune").strong().size(16.0),
                        );
                        ui.colored_label(
                            TEXT_SECONDARY,
                            egui::RichText::new("VISUAL ENGINE").size(9.0),
                        );
                    });
                });

                ui.add_space(24.0);

                // ── Nav items ───────────────────────────────────
                self.sidebar_nav_item(ui, "🏠  Dashboard", Tab::Dashboard);
                self.sidebar_nav_item(ui, "📁  Collections", Tab::Collections);
                self.sidebar_nav_item(ui, "🔍  Visual Search", Tab::VisualSearch);
                self.sidebar_nav_item(ui, "➕  Index New", Tab::IndexNew);
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                self.sidebar_nav_item(ui, "⚙  Settings", Tab::Settings);

                // ── AI Processing box pinned to bottom ──────────
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(4.0);
                    egui::Frame::default()
                        .fill(BG_CARD)
                        .rounding(8)
                        .inner_margin(10)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    ACCENT_GREEN,
                                    egui::RichText::new("⚡ AI PROCESSING").size(10.0).strong(),
                                );
                            });
                            ui.add_space(4.0);
                            ui.colored_label(
                                TEXT_SECONDARY,
                                egui::RichText::new(
                                    "Powered by CLIP vision\nmodels. Indexing is\nperformed on-device."
                                ).size(10.0),
                            );
                        });
                });
            });
    }

    #[allow(deprecated)]
    fn sidebar_nav_item(&mut self, ui: &mut egui::Ui, label: &str, tab: Tab) {
        let is_active = self.active_tab == tab;
        let fill = if is_active { SIDEBAR_ACTIVE } else { egui::Color32::TRANSPARENT };
        let text_col = if is_active { TEXT_PRIMARY } else { TEXT_SECONDARY };

        let btn = egui::Frame::default()
            .fill(fill)
            .rounding(8)
            .inner_margin(egui::Margin::symmetric(10, 7))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.colored_label(
                    text_col,
                    egui::RichText::new(label).size(13.0),
                );
            });

        if btn.response.interact(egui::Sense::click()).clicked() {
            self.active_tab = tab;
        }

        ui.add_space(2.0);
    }
}

impl eframe::App for NeptuneLensApp {
    #[allow(deprecated)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle messages from background threads
        self.handle_background_messages();

        // Handle action triggers
        if self.state.start_indexing {
            self.state.start_indexing = false;
            self.start_indexing();
        }
        if self.state.start_search {
            self.state.start_search = false;
            self.start_search();
        }
        if self.state.clear_index {
            self.state.clear_index = false;
            self.clear_index();
        }
        if self.state.needs_save {
            self.state.needs_save = false;
            self.save_data();
        }

        // Handle tab switch requests from child panels
        if let Some(tab) = self.state.switch_tab.take() {
            self.active_tab = tab;
        }

        // ── Sidebar ─────────────────────────────────────────────────
        self.render_sidebar(ctx);

        // ── Model error banner ──────────────────────────────────────
        if let Some(ref err) = self.model_error {
            egui::TopBottomPanel::top("model_error")
                .frame(
                    egui::Frame::default()
                        .fill(egui::Color32::from_rgb(60, 30, 10))
                        .inner_margin(8),
                )
                .show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 180, 80),
                        format!("⚠ {}", err),
                    );
                });
        }

        // ── Central panel ───────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(BG_DARK)
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.active_tab {
                            Tab::Dashboard => {
                                crate::ui::dashboard::show(ui, &mut self.state, self.model_error.is_none());
                            }
                            Tab::Collections => {
                                crate::ui::collections_panel::show(ui, &mut self.state);
                            }
                            Tab::IndexNew => {
                                crate::ui::indexing_panel::show(ui, &mut self.state);
                            }
                            Tab::VisualSearch => {
                                crate::ui::search_panel::show(ui, &mut self.state);
                                crate::ui::results_grid::show(ui, &mut self.state);
                            }
                            Tab::Settings => {
                                crate::ui::settings_panel::show(ui, &mut self.state);
                            }
                        }
                    });
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_data();
        log::info!("Neptune Lens shutting down, data saved.");
    }
}
