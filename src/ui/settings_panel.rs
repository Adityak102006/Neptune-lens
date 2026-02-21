// Neptune Lens - Settings Panel UI
// Configuration, statistics, and index management

use crate::egui;
use crate::app::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("⚙ Settings");
    ui.add_space(12.0);

    // Search parameters
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(35, 38, 48))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.strong("Search Parameters");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Top-N Results:");
                ui.add(egui::Slider::new(&mut state.top_n, 5..=100).text("results"));
            });
        });

    ui.add_space(16.0);

    // Index statistics
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(35, 38, 48))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.strong("Index Statistics");
            ui.add_space(8.0);

            ui.label(format!("📊 Total images indexed: {}", state.metadata.len()));
            ui.label(format!(
                "📁 Indexed folders: {}",
                state.metadata.indexed_folders().len()
            ));
            ui.label(format!("🔢 Vector store size: {}", state.vector_store_len));
        });

    ui.add_space(16.0);

    // Index management
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(35, 38, 48))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.strong("Index Management");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .button("🗑 Clear Index")
                    .on_hover_text("Remove all indexed data")
                    .clicked()
                {
                    state.clear_index = true;
                }

                if ui
                    .button("💾 Save Index")
                    .on_hover_text("Manually save the current index to disk")
                    .clicked()
                {
                    state.needs_save = true;
                }
            });
        });

    ui.add_space(16.0);

    // About section
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(35, 38, 48))
        .rounding(8.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.strong("About Neptune Lens");
            ui.add_space(4.0);
            ui.label("A local Google Lens–style image search engine.");
            ui.label("Built with Rust, CLIP (ViT-B-32), and cosine similarity.");
            ui.add_space(4.0);
            ui.weak("v0.1.0");
        });
}
