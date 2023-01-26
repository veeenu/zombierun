use eframe::NativeOptions;
use egui::Vec2;

fn main() {
    tracing_subscriber::fmt::init();
    eframe::run_native(
        "john's zombie runs",
        NativeOptions {
            initial_window_size: Some(Vec2::new(400., 320.)),
            min_window_size: Some(Vec2::new(400., 320.)),
            max_window_size: Some(Vec2::new(400., 320.)),
            ..Default::default()
        },
        Box::new(|cc| Box::new(zombierun::App::new(cc))),
    );
}
