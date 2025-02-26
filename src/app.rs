use crate::gemini;
use eframe::egui::{self, Button, TextEdit};

pub struct Weiner {
    route: String,
    content: String, //todo parse into styled/rich text
    history: Vec<String>,
    offset: usize,
    fetch_requested: bool,
    fetch_promise: Option<poll_promise::Promise<Option<(String, Vec<u8>)>>>,
}

impl Default for Weiner {
    fn default() -> Self {
        Self {
            route: "gemini://geminiprotocol.net/".to_owned(),
            content: Default::default(),
            history: vec!["gemini://geminiprotocol.net/".to_owned()],
            offset: 1,
            fetch_requested: true,
            fetch_promise: None,
        }
    }
}

impl Weiner {
    fn process_gemini(&mut self) {
        //web shit
        if self.fetch_requested {
            let url_str = self.route.clone();
            self.fetch_promise = Some(poll_promise::Promise::spawn_thread(
                //Is there an issue if new thread spawned to overwrite an active thread? Does it die or get orphaned?
                "gemini::fetch",
                move || gemini::fetch(&url_str),
            ));
            self.fetch_requested = false;
        }

        if self.fetch_promise.is_some() {
            //display fetch gemini response and update history
            if let Some(ready) = self.fetch_promise.as_ref().unwrap().ready() {
                match ready {
                    Some((uri, response)) => {
                        self.content = String::from_utf8_lossy(response).into_owned();
                        self.route = uri.clone(); //update with endpoint uri (redirection etc)
                        if self.route != self.history[self.history.len() - self.offset] {
                            self.history
                                .insert(self.history.len() - self.offset + 1, self.route.clone());
                        }
                    }
                    None => {
                        self.route = self.history[self.history.len() - self.offset].clone();
                        //failed for some reason, reset to current page
                    }
                }
                self.fetch_promise = None; //make request complete
            }
        }
    }
}

impl eframe::App for Weiner {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| {
            i.key_pressed(egui::Key::Escape) | (i.modifiers.ctrl && i.key_pressed(egui::Key::Q))
        }) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        self.process_gemini();

        //ui
        egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_theme_preference_switch(ui);

                if ui.button("🏠").clicked() {
                    self.route = Weiner::default().route;
                    self.fetch_requested = true;
                }

                let searchbar = ui.add_enabled(
                    self.fetch_promise.is_none(),
                    TextEdit::singleline(&mut self.route)
                        .desired_width(ui.available_width() * 0.75),
                );

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
                    //indicating loading page/request etc
                    ui.spinner();
                }

                //layout input
                self.fetch_requested |=
                    searchbar.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.input(|i| i.key_pressed(egui::Key::Slash)) {
                    searchbar.request_focus();
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.collapsing("History", |ui| {
                let mut i = self.history.len();
                for item in &self.history {
                    if self.offset == i {
                        ui.label(format!("> {}", item));
                    } else {
                        ui.label(item);
                    }
                    i -= 1;
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(egui::RichText::new(&self.content));
            });
        });
    }
}
