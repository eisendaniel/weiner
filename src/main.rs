fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Weiner::Client",
        options,
        Box::new(|_| Ok(Box::new(weiner::Weiner::default()))),
    )
}
