use crate::gemini;
use eframe::egui::{self, Button};

pub struct Weiner {
    route: String,
    status: String,  //todo remove
    content: String, //todo parse into styled/rich text
    history: Vec<String>,
    offset: usize,
    fetch_requested: bool,
    fetch_promise: Option<poll_promise::Promise<Option<Vec<u8>>>>,
}

impl Default for Weiner {
    fn default() -> Self {
        Self {
            route: "gemini://geminiprotocol.net/".to_owned(),
            status: Default::default(),
            content: Default::default(),
            history: vec!["gemini://geminiprotocol.net/".to_owned()],
            offset: 1,
            fetch_requested: true,
            fetch_promise: None,
        }
    }
}

impl eframe::App for Weiner {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        //web shit
        if self.fetch_requested {
            let url_str = self.route.clone();
            self.fetch_promise = Some(poll_promise::Promise::spawn_thread(
                //Is there an issue if new thread spawned to overwrite an active thread?
                "gemini::fetch",
                move || gemini::fetch(&url_str),
            ));
            self.fetch_requested = false;
        }
        if self.fetch_promise.is_some() {
            if let Some(ready) = self.fetch_promise.as_ref().unwrap().ready() {
                match ready {
                    Some(response) => {
                        let response = String::from_utf8_lossy(response).into_owned();
                        (self.status, self.content) = match response.split_once("\r\n") {
                            Some((s, c)) => (s.to_owned(), c.to_owned()),
                            None => (Default::default(), Default::default()),
                        };
                        if self.route != self.history[self.history.len() - self.offset] {
                            self.history.push(self.route.clone()); //successful request
                        }
                        println!("{:?}", self.history);
                    }
                    None => {
                        self.route = self.history[self.history.len() - self.offset].clone();
                        //failed for some reason, reset to current page
                        println!("{:?}", self.history);
                    }
                }
                self.fetch_promise = None;
            }
        }

        //ui
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_theme_preference_switch(ui);

                if ui.button("🏠").clicked() {
                    self.route = Weiner::default().route;
                    self.fetch_requested = true;
                }

                let linkedit = ui.text_edit_singleline(&mut self.route);
                if linkedit.lost_focus() {
                    self.offset = 1;
                    self.fetch_requested = true;
                }
                if ui.button("⟳").clicked() {
                    self.fetch_requested = true;
                }

                if ui
                    .add_enabled(self.offset < self.history.len(), Button::new("⬅"))
                    .clicked()
                {
                    self.offset = (self.offset + 1).clamp(1, self.history.len());
                    self.route = self.history[self.history.len() - self.offset].clone();
                    self.fetch_requested = true;
                }

                if ui.add_enabled(self.offset > 1, Button::new("➡")).clicked() {
                    self.offset = (self.offset - 1).clamp(1, self.history.len());
                    self.route = self.history[self.history.len() - self.offset].clone();
                    self.fetch_requested = true;
                }
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
