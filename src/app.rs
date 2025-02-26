use crate::gemini;
use eframe::egui::{
    self, Button, CentralPanel, Key, RichText, ScrollArea, TextEdit, TopBottomPanel,
    ViewportCommand,
};
use poll_promise::Promise;

pub struct Weiner {
    route: String,
    content: String, //todo parse into styled/rich text
    history: Vec<String>,
    offset: usize,
    fetch_requested: bool,
    fetch_promise: Option<Promise<gemini::Response>>,
}

impl Default for Weiner {
    fn default() -> Self {
        let home = "gemini://geminiprotocol.net/";
        Self {
            route: home.to_owned(),
            content: Default::default(),
            history: vec![home.to_owned()],
            offset: 1,
            fetch_requested: true,
            fetch_promise: None,
        }
    }
}

impl eframe::App for Weiner {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(Key::Escape) | (i.modifiers.ctrl && i.key_pressed(Key::Q))) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        self.process_gemini();
        self.draw_toolbar(ctx);
        self.draw_content(ctx);
    }
}

impl Weiner {
    fn process_gemini(&mut self) {
        //web shit
        if self.fetch_requested {
            let url_str = self.route.clone();
            self.fetch_promise = Some(Promise::spawn_thread(
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

    fn draw_toolbar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_theme_preference_switch(ui);

                let back = ui.add_enabled(self.offset < self.history.len(), Button::new("⬅"));
                let forward = ui.add_enabled(self.offset > 1, Button::new("➡"));

                if self.fetch_promise.is_some() {
                    if ui.button("🗙").clicked() {
                        self.fetch_promise = None;
                        self.route = self.history[self.history.len() - self.offset].clone();
                    }
                } else {
                    self.fetch_requested |=
                        ui.button("⟳").clicked() || ui.input(|i| i.key_pressed(Key::F5));
                }

                let searchbar = ui.add_enabled(
                    self.fetch_promise.is_none(),
                    TextEdit::singleline(&mut self.route).desired_width(ui.available_width()),
                );

                if self.fetch_promise.is_some() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(10.0);
                        ui.spinner();
                    });
                };

                //layout input
                self.fetch_requested |=
                    searchbar.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                if ui.input(|i| i.key_pressed(Key::Slash)) {
                    searchbar.request_focus();
                }
                if back.clicked()
                    || (back.enabled()
                        && ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowLeft)))
                {
                    self.offset = (self.offset + 1).clamp(1, self.history.len());
                    self.route = self.history[self.history.len() - self.offset].clone();
                    self.fetch_requested = true;
                }
                if forward.clicked()
                    || (forward.enabled()
                        && ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowRight)))
                {
                    self.offset = (self.offset - 1).clamp(1, self.history.len());
                    self.route = self.history[self.history.len() - self.offset].clone();
                    self.fetch_requested = true;
                }
            });
        });
    }

    fn draw_content(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
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
            ScrollArea::vertical().show(ui, |ui| {
                ui.label(RichText::new(&self.content));
            });
        });
    }
}
