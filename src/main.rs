fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder {
            inner_size: Some([600.0, 800.0].into()),
            min_inner_size: Some([240.0, 100.0].into()),
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "Weiner::Client",
        options,
        Box::new(|_| Ok(Box::new(weiner::Weiner::default()))),
    )
}
