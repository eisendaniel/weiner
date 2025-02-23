use eframe::egui::{self};
use native_tls::TlsConnector;
use std::io::{Read, Write};
use std::net::TcpStream;

const DEFAULT_LINK: &str = "gemini://geminiprotocol.net/";

fn get_gemini_content(link: &url::Url) -> Vec<u8> {
    
    let host = link.host_str().unwrap();
    let url = format!("{}:1965", host);
    
    
    let mut builder = TlsConnector::builder();
    builder.danger_accept_invalid_certs(true);
    let connector = builder.build().unwrap();
    
    let stream = match TcpStream::connect(&url){
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let mut mstream = connector.connect(host, stream).unwrap();

    let request = format!("{}\r\n", link);

    mstream.write_all(request.as_bytes()).unwrap();
    let mut response = vec![];
    mstream.read_to_end(&mut response).unwrap();

    response
}

fn main() -> eframe::Result {
    //-----------------------------------------
    let mut url_str = DEFAULT_LINK.to_string();

    let response = String::from_utf8_lossy(&get_gemini_content(
        &url::Url::parse(&url_str).expect("Invalid URL"),
    ))
    .to_string();

    let (mut status, mut content) = match response.split_once("\r\n") {
        Some((s, c)) => (s.to_owned(), c.to_owned()),
        None => todo!(),
    };
    //-----------------------------------------
    let options = eframe::NativeOptions::default();
    let mut page_promise: poll_promise::Promise<Vec<u8>> =
        poll_promise::Promise::spawn_thread("page_fetch", || (vec![0])); //0 init
    let mut loading = false;

    eframe::run_simple_native("GeminiClient", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_theme_preference_switch(ui);

                let link_label = ui.label("URL: ");
                let linkedit = ui
                    .text_edit_singleline(&mut url_str)
                    .labelled_by(link_label.id);
                let link = url::Url::parse(&url_str).expect("Invalid URL");

                if linkedit.lost_focus() {
                    loading = true;
                    page_promise = poll_promise::Promise::spawn_thread("page_fetch", move || {
                        get_gemini_content(&link)
                    });
                }
                if loading {
                    if let Some(response) = page_promise.ready() {
                        let response = String::from_utf8_lossy(&response).to_string();

                        (status, content) = match response.split_once("\r\n") {
                            Some((s, c)) => (s.to_owned(), c.to_owned()),
                            None => todo!(),
                        };
                        loading = false;
                    } else {
                        ui.spinner();
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.label(&status);
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
                        |ui| {
                            // ui.label(&content);
                            ui.label(egui::RichText::new(&content));
                        },
                    )
                });
            });
        });
    })
}
