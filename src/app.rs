use crate::gemini;
use eframe::egui;

pub struct Weiner {
    current: String,
    route: String,
    status: String, //todo remove
    content: String, //todo parse into styled/rich text

    fetch_requested: bool,
    fetch_promise: Option<poll_promise::Promise<Option<Vec<u8>>>>,
}

impl Default for Weiner {
    fn default() -> Self {
        Self {
            current: "gemini://geminiprotocol.net/".to_owned(),
            route: "gemini://geminiprotocol.net/".to_owned(),
            status: Default::default(),
            content: Default::default(),
            fetch_requested: true,
            fetch_promise: None,
        }
    }
}

impl eframe::App for Weiner {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.fetch_requested {
            let url_str = self.route.clone();
            self.fetch_promise = Some(poll_promise::Promise::spawn_thread(
                "gemini::fetch",
                move || gemini::fetch(&url_str),
            ));
            self.fetch_requested = false;
        }

        if self.fetch_promise.is_some() {
            if let Some(ready) = self.fetch_promise.as_ref().unwrap().ready() {
                match ready {
                    Some(response) => {
                        let response = String::from_utf8_lossy(response).to_owned();
                        (self.status, self.content) = match response.split_once("\r\n") {
                            Some((s, c)) => (s.to_owned(), c.to_owned()),
                            None => (Default::default(), Default::default()),
                        };
                        self.current = self.route.clone(); //successful request
                    }
                    None => {
                        self.route = self.current.clone() //failed for some reason, reset to current page
                    }
                }
                self.fetch_promise = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_theme_preference_switch(ui);

                if ui.button("🏠").clicked() {
                    self.route = Weiner::default().route;
                    self.fetch_requested = true;
                }
                
                let linkedit = ui.text_edit_singleline(&mut self.route);
                if linkedit.lost_focus() && (self.route != self.current) {
                    self.fetch_requested = true;
                }
                if ui.button("⟳").clicked() {
                    self.fetch_requested = true;
                }
                let _ = ui.button("⬅");
                let _ = ui.button("➡");
                if self.fetch_promise.is_some() {
                    ui.spinner();
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.label(&self.status);
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                        |ui| {
                            // ui.label(&content);
                            ui.label(egui::RichText::new(&self.content));
                        },
                    )
                });
            });
        });
    }
}
