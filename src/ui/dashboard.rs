// Neptune Lens - Dashboard Panel UI
// Welcome screen with stats, quick launch, getting started, and recent collections

#[allow(deprecated)]
use crate::egui;
use crate::app::{AppState, Tab, BG_CARD, ACCENT_BLUE, ACCENT_GREEN, ACCENT_PURPLE, ACCENT_YELLOW, TEXT_PRIMARY, TEXT_SECONDARY};

#[allow(deprecated)]
pub fn show(ui: &mut egui::Ui, state: &mut AppState, model_ok: bool) {
    let avail = ui.available_width();

    // ── Header row ──────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.colored_label(
                TEXT_PRIMARY,
                egui::RichText::new("Welcome to Neptune")
                    .strong()
                    .size(26.0),
            );
            ui.colored_label(
                TEXT_SECONDARY,
                "Your intelligent local image search engine powered by CLIP.",
            );
        });

        // Right-aligned AI status badge
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            let (status_text, status_color) = if model_ok {
                ("✅ Model Connected", ACCENT_GREEN)
            } else {
                ("⚠ Model Offline", egui::Color32::from_rgb(255, 130, 80))
            };
            egui::Frame::default()
                .fill(BG_CARD)
                .rounding(8)
                .inner_margin(egui::Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.colored_label(
                        TEXT_SECONDARY,
                        egui::RichText::new("AI STATUS").size(9.0),
                    );
                    ui.colored_label(
                        status_color,
                        egui::RichText::new(status_text).strong().size(12.0),
                    );
                });
        });
    });

    ui.add_space(20.0);

    // ── Stat cards row ──────────────────────────────────────────────
    let spacing = 12.0_f32;
    let card_w = (avail - spacing * 3.0) / 4.0;

    let stats: Vec<(&str, &str, String, egui::Color32)> = vec![
        ("📂", "Collections", format!("{}", state.metadata.indexed_folders().len()), ACCENT_BLUE),
        ("🖼", "Indexed Images", format!("{}", state.metadata.len()), ACCENT_GREEN),
        ("⚡", "Search Speed", "~15ms".to_string(), ACCENT_YELLOW),
        ("🔭", "Engine Status",
            if model_ok { "Ready" } else { "Offline" }.to_string(),
            if model_ok { ACCENT_GREEN } else { egui::Color32::from_rgb(255, 130, 80) }),
    ];

    ui.horizontal(|ui| {
        for (i, (icon, label, value, accent)) in stats.iter().enumerate() {
            if i > 0 {
                ui.add_space(spacing);
            }
            ui.allocate_ui_with_layout(
                egui::vec2(card_w, 60.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    stat_card(ui, card_w, icon, label, value, *accent);
                },
            );
        }
    });

    ui.add_space(24.0);

    // ── Two-column layout ───────────────────────────────────────────
    let left_w = (avail * 0.58).max(300.0);
    let right_w = avail - left_w - 16.0;

    ui.horizontal(|ui| {
        // ── Left column ─────────────────────────────────────────
        ui.vertical(|ui| {
            ui.set_width(left_w);

            // Quick Launch heading
            ui.horizontal(|ui| {
                ui.colored_label(TEXT_PRIMARY,
                    egui::RichText::new("Quick Launch").strong().size(18.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.link(egui::RichText::new("Full Search Engine →")
                        .color(ACCENT_BLUE).size(12.0)).clicked() {
                        state.switch_tab = Some(Tab::VisualSearch);
                    }
                });
            });
            ui.add_space(8.0);

            // Blue banner
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(20, 50, 100))
                .rounding(12)
                .inner_margin(20)
                .show(ui, |ui| {
                    ui.set_width(left_w - 40.0);
                    ui.colored_label(TEXT_PRIMARY,
                        egui::RichText::new("Find what you're looking for")
                            .strong().size(18.0));
                    ui.add_space(6.0);
                    ui.colored_label(TEXT_SECONDARY,
                        egui::RichText::new(
                            "Enter a text description or upload a reference image\nto find visually similar photos instantly."
                        ).size(13.0));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if styled_button(ui, "Start Visual Query  →", ACCENT_BLUE, TEXT_PRIMARY) {
                            state.switch_tab = Some(Tab::VisualSearch);
                        }
                        ui.add_space(8.0);
                        if styled_button(ui, "Index New Images", BG_CARD, TEXT_PRIMARY) {
                            state.switch_tab = Some(Tab::IndexNew);
                        }
                    });
                });

            ui.add_space(20.0);

            // Recent Collections
            ui.colored_label(TEXT_PRIMARY,
                egui::RichText::new("Recent Collections").strong().size(18.0));
            ui.add_space(8.0);

            egui::Frame::default()
                .fill(BG_CARD)
                .rounding(12)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 50, 60)))
                .inner_margin(egui::Margin::symmetric(16, 24))
                .show(ui, |ui| {
                    ui.set_width(left_w - 32.0);
                    let folders = state.metadata.indexed_folders();
                    if folders.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            ui.colored_label(TEXT_SECONDARY, "No collections yet.");
                            ui.add_space(12.0);
                            if styled_button(ui, "Index your first library", BG_CARD, ACCENT_BLUE) {
                                state.switch_tab = Some(Tab::IndexNew);
                            }
                            ui.add_space(8.0);
                        });
                    } else {
                        for (i, folder) in folders.iter().take(5).enumerate() {
                            if i > 0 { ui.add_space(4.0); }
                            let count = state.metadata.folder_image_counts();
                            let n = count.get(folder).unwrap_or(&0);
                            ui.horizontal(|ui| {
                                ui.colored_label(ACCENT_BLUE, "📁");
                                let name = folder.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| folder.display().to_string());
                                ui.colored_label(TEXT_PRIMARY, &name);
                                ui.colored_label(TEXT_SECONDARY, format!("({} images)", n));
                            });
                        }
                    }
                });
        });

        ui.add_space(16.0);

        // ── Right column ────────────────────────────────────────
        ui.vertical(|ui| {
            ui.set_width(right_w);

            ui.colored_label(TEXT_PRIMARY,
                egui::RichText::new("Getting Started").strong().size(18.0));
            ui.add_space(12.0);

            getting_started_step(ui, "1", ACCENT_BLUE,
                "Create a Collection",
                "Upload a folder of images. Our AI will process\nthem to create unique visual fingerprints.");
            ui.add_space(10.0);
            getting_started_step(ui, "2", ACCENT_GREEN,
                "Wait for Indexing",
                "High-dimensional embeddings are generated\nand stored in your local persistent database.");
            ui.add_space(10.0);
            getting_started_step(ui, "3", ACCENT_PURPLE,
                "Query by Image or Text",
                "Describe what you want or upload a reference\nphoto to find matching aesthetics instantly.");

            ui.add_space(20.0);

            // Pro Tip
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(30, 25, 50))
                .rounding(12)
                .inner_margin(egui::Margin::symmetric(16, 14))
                .show(ui, |ui| {
                    ui.set_width(right_w - 32.0);
                    ui.colored_label(ACCENT_PURPLE,
                        egui::RichText::new("✨ Pro Tip").strong().size(14.0));
                    ui.add_space(6.0);
                    ui.colored_label(TEXT_SECONDARY,
                        egui::RichText::new(
                            "\"Semantic search works best for finding patterns,\ncolors, and textures. It's like having a super-\npowered eye for aesthetics.\""
                        ).italics().size(12.0));
                });
        });
    });
}

// ─── Helper: stat card ──────────────────────────────────────────────
#[allow(deprecated)]
fn stat_card(ui: &mut egui::Ui, width: f32, icon: &str, label: &str, value: &str, accent: egui::Color32) {
    egui::Frame::default()
        .fill(BG_CARD)
        .rounding(12)
        .inner_margin(14)
        .show(ui, |ui| {
            let inner_w = width - 28.0;
            ui.set_min_width(inner_w);
            ui.set_width(inner_w);
            ui.horizontal(|ui| {
                ui.colored_label(accent, egui::RichText::new(icon).size(22.0));
                ui.vertical(|ui| {
                    ui.colored_label(TEXT_SECONDARY, egui::RichText::new(label).size(11.0));
                    ui.colored_label(TEXT_PRIMARY, egui::RichText::new(value).strong().size(20.0));
                });
            });
        });
}

// ─── Helper: getting started step ───────────────────────────────────
fn getting_started_step(ui: &mut egui::Ui, number: &str, color: egui::Color32, title: &str, description: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 14.0, color);
        ui.painter().text(
            rect.center(), egui::Align2::CENTER_CENTER,
            number, egui::FontId::proportional(13.0), egui::Color32::WHITE);
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.colored_label(TEXT_PRIMARY, egui::RichText::new(title).strong().size(14.0));
            ui.colored_label(TEXT_SECONDARY, egui::RichText::new(description).size(11.5));
        });
    });
}

// ─── Helper: styled button ──────────────────────────────────────────
#[allow(deprecated)]
fn styled_button(ui: &mut egui::Ui, text: &str, bg: egui::Color32, text_color: egui::Color32) -> bool {
    let btn = egui::Button::new(egui::RichText::new(text).color(text_color).size(13.0))
        .fill(bg)
        .rounding(8);
    ui.add(btn).clicked()
}
