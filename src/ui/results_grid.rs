// Neptune Lens - Results Grid UI
// Display search results as thumbnail cards with similarity scores

use crate::egui;
use crate::app::AppState;
use crate::db::vector_store::SearchResult;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    if state.search_results.is_empty() {
        return;
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    ui.strong(format!("🎯 Top {} Results", state.search_results.len()));
    ui.add_space(8.0);

    // Calculate number of columns based on available width
    let available_width = ui.available_width();
    let card_width = 180.0_f32;
    let spacing = 12.0_f32;
    let cols = ((available_width + spacing) / (card_width + spacing))
        .floor()
        .max(1.0) as usize;

    // Display results in a grid
    let results = state.search_results.clone();
    let chunks: Vec<&[SearchResult]> = results.chunks(cols).collect();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in chunks {
                ui.horizontal(|ui| {
                    for result in row {
                        ui.vertical(|ui| {
                            render_result_card(ui, result, card_width);
                        });
                        ui.add_space(spacing);
                    }
                });
                ui.add_space(spacing);
            }
        });
}

fn render_result_card(ui: &mut egui::Ui, result: &SearchResult, card_width: f32) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(35, 38, 48))
        .rounding(8.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(card_width - 16.0);
            ui.set_min_height(220.0);

            // Image thumbnail — load bytes directly for Windows reliability
            let key = format!("bytes://result/{}", result.path.display());
            if let Ok(bytes) = std::fs::read(&result.path) {
                ui.add(
                    egui::Image::from_bytes(key, bytes)
                        .max_width(card_width - 24.0)
                        .max_height(150.0)
                        .rounding(4.0),
                );
            }

            ui.add_space(4.0);

            // Similarity score with color coding
            let score_pct = (result.similarity * 100.0).min(100.0);
            let score_color = score_to_color(result.similarity);

            ui.colored_label(score_color, format!("{:.1}% match", score_pct));

            // File name (truncated)
            if let Some(name) = result.path.file_name().and_then(|n| n.to_str()) {
                let display_name = if name.len() > 22 {
                    format!("{}...", &name[..19])
                } else {
                    name.to_string()
                };
                ui.weak(display_name);
            }

            // Click to open in system viewer
            if ui
                .small_button("📂 Open")
                .on_hover_text("Open in default image viewer")
                .clicked()
            {
                if let Err(e) = open::that(&result.path) {
                    log::warn!("Failed to open image: {}", e);
                }
            }
        });
}

/// Map similarity score to a color (red → yellow → green)
fn score_to_color(similarity: f32) -> egui::Color32 {
    let s = similarity.clamp(0.0, 1.0);
    if s > 0.8 {
        egui::Color32::from_rgb(80, 220, 100)
    } else if s > 0.6 {
        egui::Color32::from_rgb(200, 220, 80)
    } else if s > 0.4 {
        egui::Color32::from_rgb(255, 200, 80)
    } else {
        egui::Color32::from_rgb(255, 120, 80)
    }
}
