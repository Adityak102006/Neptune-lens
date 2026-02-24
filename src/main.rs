// Neptune Lens - Local Image Search Engine
// Entry point: launches the egui/eframe desktop application

// Re-export egui for use across the crate
pub use eframe::egui;

mod app;
mod db;
mod engine;
mod ui;

fn main() -> eframe::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    log::info!("Starting Neptune Lens v0.2.0");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("Neptune Lens — Image Search Engine"),
        ..Default::default()
    };

    eframe::run_native(
        "Neptune Lens",
        native_options,
        Box::new(|cc| Ok(Box::new(app::NeptuneLensApp::new(cc)))),
    )
}
