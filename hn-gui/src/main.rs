use std::sync::{Arc};
use chrono::{DateTime, Utc};
use eframe::{App, egui, Frame};
use eframe::egui::{Align, CentralPanel, Color32, Context, FontId, Layout, ScrollArea, TopBottomPanel, Ui, Visuals};
use eframe::egui::style::Margin;
use egui_extras::{RetainedImage};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{Mutex};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StoryItem {
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub text: Option<String>,
    #[serde(default)]
    pub by: String,
    #[serde(default)]
    pub score: i64,
    #[serde(default)]
    pub descendants: u64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub time: DateTime<Utc>,
    #[serde(default)]
    pub kids: Vec<u64>,
    pub r#type: String,
}

struct AppState {
    url: String,
    request_content: Arc<Mutex<Vec<StoryItem>>>,
    runtime: Runtime,
    // hn_logo: Option<RetainedImage>,
    hn_logo: RetainedImage,
}

fn fetch_story_items(state: &mut AppState) {
    let url = state.url.clone();
    let request_content = state.request_content.clone();
    state.runtime.spawn(async move {
        let ids: Vec<u64> =
            reqwest::get(url)
                .await?
                .json::<Vec<u64>>()
                .await?;
        let elements: Vec<StoryItem> =
            futures::future::join_all(ids[0..20].iter()
                .map(|id| async {
                    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", *id);
                    reqwest::get(url).await?.json::<StoryItem>().await
                }))
                .await
                .into_iter()
                .filter_map(|res| res.ok())
                .collect();
        *request_content.lock().await = elements;
        Ok::<(), reqwest::Error>(())
    });
}

fn populate_story_items(state: &AppState, ui: &mut Ui) {
    for story_item in &mut *state.request_content.blocking_lock() {
        ui.add_space(10.0);
        ui.label(&story_item.title);
        ui.label(format!("posted by {}", &story_item.by));
        if let Some(text) = &story_item.text {
            if ui.link(text).clicked() {
                open::that(text).unwrap();
            }
        };
        if ui.link(format!("Comments {}", &story_item.kids.len())).clicked() {
            let link = format!("https://news.ycombinator.com/item?id={}", &story_item.id);
            open::that(link).unwrap();
        }
        ui.add_space(10.0);
    }
}

fn load_fonts(ctx: &egui::Context) {
    use egui::FontFamily::Proportional;
    use egui::TextStyle::*;
    // let mut fonts = egui::paint::fonts::FontDefinitions::default();
    // let val = Cow::Borrowed(include_bytes!("../../fira.ttf"));
    // fonts.fam.insert(egui::TextStyle::Body, val);

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (Heading, FontId::new(40.0, Proportional)),
        (Name("Heading2".into()), FontId::new(30.0, Proportional)),
        (Name("Context".into()), FontId::new(28.0, Proportional)),
        (Body, FontId::new(25.0, Proportional)),
        (Monospace, FontId::new(25.0, Proportional)),
        (Button, FontId::new(25.0, Proportional)),
        (Small, FontId::new(25.0, Proportional)),
    ].into();
    ctx.set_style(style);
}

impl App for AppState {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_visuals(Visuals::light());
        let frame = egui::containers::Frame {
            inner_margin: Default::default(),
            outer_margin: Default::default(),
            rounding: egui::Rounding { nw: 1.0, ne: 1.0, sw: 1.0, se: 1.0 },
            shadow: eframe::epaint::Shadow::default(),
            stroke: egui::Stroke::none(),
            fill: Color32::from_rgb(246, 246, 239),
        };
        let frame2 = egui::containers::Frame {
            inner_margin: Margin::from(7.0),
            outer_margin: Default::default(),
            rounding: egui::Rounding { nw: 1.0, ne: 1.0, sw: 1.0, se: 1.0 },
            shadow: eframe::epaint::Shadow::default(),
            stroke: egui::Stroke::none(),
            fill: Color32::from_rgb(255, 102, 0),
        };
        // ff6600
        TopBottomPanel::top("top_panel").frame(frame2).show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                self.hn_logo.show(ui);
                ui.colored_label(Color32::from_rgb(32, 32, 32), "Hacker News");
            });
        });
        CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Url: ");
                ui.text_edit_singleline(&mut self.url);
            });
            if ui.button("Load content").clicked() {
                fetch_story_items(self);
            }
            ScrollArea::vertical().show(ui, |mut ui| {
                populate_story_items(&self, &mut ui);
            });
        });
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let options = eframe::NativeOptions::default();
    let state = AppState {
        url: "https://hacker-news.firebaseio.com/v0/topstories.json".to_owned(),
        request_content: Arc::new(Mutex::new(Vec::new())),
        hn_logo: std::fs::read("hn-gui/y18.png")
            .map_err(|e| e.to_string())
            .and_then(|bytes|
                RetainedImage::from_image_bytes("HN Logo", &bytes[..]))
            .unwrap(),
        runtime,
    };

    eframe::run_native(
        "First egui window",
        options,
        Box::new(|cc| {
            load_fonts(&cc.egui_ctx);
            Box::new(state)
        }),
    );
}
