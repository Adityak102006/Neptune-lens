// Neptune Lens - Indexing Panel UI
// Folder selection, indexing progress, and folder management

use crate::egui;
use crate::app::AppState;
use std::path::PathBuf;

pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("📂 Image Indexing");
    ui.add_space(8.0);

    // Add Folder button
    ui.horizontal(|ui| {
        if ui
            .button("➕ Add Folder")
            .on_hover_text("Select a folder to index for images")
            .clicked()
        {
            if let Some(folder) = rfd::FileDialog::new()
                .set_title("Select Image Folder")
                .pick_folder()
            {
                state.pending_folders.push(folder);
            }
        }

        if !state.is_indexing && !state.pending_folders.is_empty() {
            if ui.button("🔍 Start Indexing").clicked() {
                state.start_indexing = true;
            }
        }
    });

    ui.add_space(12.0);

    // Pending folders to index
    if !state.pending_folders.is_empty() {
        ui.colored_label(egui::Color32::from_rgb(100, 180, 255), "Folders to index:");
        let mut to_remove = Vec::new();
        for (i, folder) in state.pending_folders.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("📁 {}", folder.display()));
                if ui.small_button("✖").clicked() {
                    to_remove.push(i);
                }
            });
        }
        for i in to_remove.into_iter().rev() {
            state.pending_folders.remove(i);
        }
        ui.add_space(8.0);
    }

    // Indexing progress
    if state.is_indexing {
        ui.separator();
        ui.add_space(4.0);

        let total = state.progress.get_total();
        let completed = state.progress.get_completed();
        let failed = state.progress.get_failed();

        let progress_frac = if total > 0 {
            completed as f32 / total as f32
        } else {
            0.0
        };

        ui.colored_label(
            egui::Color32::from_rgb(255, 200, 80),
            "⏳ Indexing in progress...",
        );

        ui.add(
            egui::ProgressBar::new(progress_frac)
                .text(format!("{} / {} images", completed, total))
                .animate(true),
        );

        if failed > 0 {
            ui.colored_label(
                egui::Color32::from_rgb(255, 130, 80),
                format!("⚠ {} images failed", failed),
            );
        }

        ui.add_space(4.0);
        if ui.button("⏹ Cancel").clicked() {
            state.progress.cancel();
        }

        ui.ctx().request_repaint();
    }

    // Show last indexing result
    if let Some(ref msg) = state.last_index_message {
        ui.add_space(8.0);
        ui.colored_label(egui::Color32::from_rgb(100, 220, 100), msg.as_str());
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // Indexed folders list
    ui.strong("Indexed Folders");
    ui.add_space(4.0);

    let folder_counts = state.metadata.folder_image_counts();
    let folders: Vec<PathBuf> = state.metadata.indexed_folders().to_vec();

    if folders.is_empty() {
        ui.weak("No folders indexed yet. Add a folder above to get started.");
    } else {
        let mut folder_to_remove = None;
        for folder in &folders {
            ui.horizontal(|ui| {
                let count = folder_counts.get(folder).unwrap_or(&0);
                ui.label(format!("📁 {} ({} images)", folder.display(), count));
                if !state.is_indexing {
                    if ui
                        .small_button("🗑")
                        .on_hover_text("Remove this folder from index")
                        .clicked()
                    {
                        folder_to_remove = Some(folder.clone());
                    }
                }
            });
        }

        if let Some(folder) = folder_to_remove {
            state.metadata.remove_folder(&folder);
            state.needs_save = true;
        }
    }

    // Total stats
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(4.0);
    ui.label(format!(
        "📊 Total indexed: {} images | {} vectors",
        state.metadata.len(),
        state.vector_store_len,
    ));
}
