// Neptune Lens - Search Panel UI
// Query image selection and search trigger

use crate::egui;
use crate::app::AppState;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("🔍 Image Search");
    ui.add_space(8.0);

    // Query image selection
    ui.horizontal(|ui| {
        if ui
            .button("🖼 Select Query Image")
            .on_hover_text("Choose an image to search for similar ones")
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Select Query Image")
                .add_filter("Images", &["jpg", "jpeg", "png", "webp", "bmp"])
                .pick_file()
            {
                state.query_image_path = Some(path);
                state.search_results.clear();
            }
        }

        if state.query_image_path.is_some()
            && !state.is_searching
            && state.metadata.len() > 0
        {
            if ui
                .button("🚀 Search")
                .on_hover_text("Find similar images in the index")
                .clicked()
            {
                state.start_search = true;
            }
        }
    });

    ui.add_space(8.0);

    // Show query image preview
    if let Some(ref path) = state.query_image_path {
        ui.weak(format!("Query: {}", path.display()));
        ui.add_space(4.0);

        // Load image bytes directly for reliable Windows support
        let key = format!("bytes://query/{}", path.display());
        if let Ok(bytes) = std::fs::read(path) {
            ui.add(
                egui::Image::from_bytes(key, bytes)
                    .max_width(250.0)
                    .max_height(250.0)
                    .rounding(8),
            );
        } else {
            ui.colored_label(egui::Color32::RED, "Failed to load image");
        }
    } else {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.weak("No query image selected");
            ui.add_space(8.0);
            ui.weak("Select an image above to search for visually similar images");
        });
    }

    // Searching indicator
    if state.is_searching {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Searching...");
        });
        ui.ctx().request_repaint();
    }

    // Search status info
    if state.metadata.is_empty() && state.query_image_path.is_some() {
        ui.add_space(8.0);
        ui.colored_label(
            egui::Color32::from_rgb(255, 180, 80),
            "⚠ No images indexed yet. Go to the Index tab first.",
        );
    }
}
