// Neptune Lens - Collections Panel UI
// Browse and manage indexed image collections (folders)

#[allow(deprecated)]
use crate::egui;
use crate::app::{AppState, Tab, BG_CARD, ACCENT_BLUE, ACCENT_GREEN, ACCENT_PURPLE, TEXT_PRIMARY, TEXT_SECONDARY};

#[allow(deprecated)]
pub fn show(ui: &mut egui::Ui, state: &mut AppState) {
    // ── Header ──────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.colored_label(
            TEXT_PRIMARY,
            egui::RichText::new("📁 Collections").strong().size(24.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("➕ New Collection").color(TEXT_PRIMARY).size(13.0),
                    )
                    .fill(ACCENT_BLUE)
                    .rounding(8),
                )
                .clicked()
            {
                state.switch_tab = Some(Tab::IndexNew);
            }
        });
    });
    ui.add_space(4.0);
    ui.colored_label(
        TEXT_SECONDARY,
        "Manage your indexed image libraries. Each folder you index becomes a collection.",
    );

    ui.add_space(20.0);

    // ── Stats bar ───────────────────────────────────────────────────
    let folders = state.metadata.indexed_folders().to_vec();
    let total_images = state.metadata.len();
    let folder_counts = state.metadata.folder_image_counts();

    ui.horizontal(|ui| {
        mini_stat(ui, "Total Collections", &format!("{}", folders.len()), ACCENT_BLUE);
        ui.add_space(16.0);
        mini_stat(ui, "Total Images", &format!("{}", total_images), ACCENT_GREEN);
        ui.add_space(16.0);
        mini_stat(ui, "Vectors Stored", &format!("{}", state.vector_store_len), ACCENT_PURPLE);
    });

    ui.add_space(24.0);

    // ── Collection cards or empty state ─────────────────────────────
    if folders.is_empty() {
        // Empty state
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.colored_label(
                TEXT_SECONDARY,
                egui::RichText::new("📂").size(48.0),
            );
            ui.add_space(12.0);
            ui.colored_label(
                TEXT_PRIMARY,
                egui::RichText::new("No collections yet").strong().size(20.0),
            );
            ui.add_space(6.0);
            ui.colored_label(
                TEXT_SECONDARY,
                "Index a folder of images to create your first collection.",
            );
            ui.add_space(16.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("➕ Index Your First Folder").color(TEXT_PRIMARY).size(14.0),
                    )
                    .fill(ACCENT_BLUE)
                    .rounding(8),
                )
                .clicked()
            {
                state.switch_tab = Some(Tab::IndexNew);
            }
        });
    } else {
        // Grid of collection cards
        ui.colored_label(
            TEXT_SECONDARY,
            egui::RichText::new("YOUR LIBRARIES").size(10.0).strong(),
        );
        ui.add_space(10.0);

        let avail = ui.available_width();
        let card_w = 280.0_f32;
        let spacing = 12.0_f32;
        let cols = ((avail + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

        let mut folder_to_remove: Option<std::path::PathBuf> = None;

        let chunks: Vec<&[std::path::PathBuf]> = folders.chunks(cols).collect();
        for row in chunks {
            ui.horizontal(|ui| {
                for folder in row {
                    let count = folder_counts.get(folder).copied().unwrap_or(0);
                    let name = folder
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| folder.display().to_string());

                    egui::Frame::default()
                        .fill(BG_CARD)
                        .rounding(12)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 44, 55)))
                        .inner_margin(16)
                        .show(ui, |ui| {
                            ui.set_width(card_w - 32.0);
                            ui.set_min_height(100.0);

                            // Folder icon + name
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    ACCENT_BLUE,
                                    egui::RichText::new("📁").size(24.0),
                                );
                                ui.vertical(|ui| {
                                    ui.colored_label(
                                        TEXT_PRIMARY,
                                        egui::RichText::new(&name).strong().size(15.0),
                                    );
                                    ui.colored_label(
                                        TEXT_SECONDARY,
                                        egui::RichText::new(format!("{} images", count)).size(12.0),
                                    );
                                });
                            });

                            ui.add_space(8.0);

                            // Path (truncated)
                            let path_str = folder.display().to_string();
                            let short_path = if path_str.len() > 45 {
                                format!("...{}", &path_str[path_str.len() - 42..])
                            } else {
                                path_str
                            };
                            ui.colored_label(
                                TEXT_SECONDARY,
                                egui::RichText::new(short_path).size(10.0),
                            );

                            ui.add_space(10.0);

                            // Action buttons
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("📂 Open").size(11.0),
                                        )
                                        .rounding(6),
                                    )
                                    .on_hover_text("Open folder in file explorer")
                                    .clicked()
                                {
                                    let _ = open::that(folder);
                                }

                                ui.add_space(4.0);

                                if !state.is_indexing {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("🗑 Remove")
                                                    .size(11.0)
                                                    .color(egui::Color32::from_rgb(255, 120, 80)),
                                            )
                                            .rounding(6),
                                        )
                                        .on_hover_text("Remove this folder from the index")
                                        .clicked()
                                    {
                                        folder_to_remove = Some(folder.clone());
                                    }
                                }
                            });
                        });

                    ui.add_space(spacing);
                }
            });
            ui.add_space(spacing);
        }

        // Handle folder removal
        if let Some(folder) = folder_to_remove {
            state.metadata.remove_folder(&folder);
            state.needs_save = true;
        }
    }
}

// ─── Helper: mini stat pill ─────────────────────────────────────────
#[allow(deprecated)]
fn mini_stat(ui: &mut egui::Ui, label: &str, value: &str, accent: egui::Color32) {
    egui::Frame::default()
        .fill(BG_CARD)
        .rounding(8)
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(accent, egui::RichText::new(value).strong().size(16.0));
                ui.colored_label(TEXT_SECONDARY, egui::RichText::new(label).size(11.0));
            });
        });
}
