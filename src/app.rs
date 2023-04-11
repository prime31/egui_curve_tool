use super::Demo;
use crate::curve_editor;
use egui::{Context, Ui};
use egui_notify::Toasts;
use std::{collections::BTreeSet, time::Duration};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    picked_path: Option<String>,
    #[serde(skip)]
    dropped_files: Vec<egui::DroppedFile>,

    #[serde(skip)]
    demos: Vec<Box<dyn Demo>>,
    #[serde(skip)]
    open: BTreeSet<String>,
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for App {
    fn default() -> Self {
        let demos: Vec<Box<dyn Demo>> = vec![Box::new(curve_editor::CurveEditor::default())];
        let mut open = BTreeSet::new();
        open.insert("🗠 Plot".to_owned());

        Self {
            picked_path: None,
            dropped_files: vec![],
            demos,
            open,
            toasts: Toasts::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any). Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn windows(&mut self, ctx: &Context) {
        let Self { demos, open, .. } = self;
        for demo in demos {
            let mut is_open = open.contains(demo.name());
            demo.show(ctx, &mut is_open, &mut self.toasts);
            App::set_open(open, demo.name(), is_open);
        }
    }

    pub fn checkboxes(&mut self, ui: &mut Ui) {
        let Self { demos, open, .. } = self;
        for demo in demos {
            let mut is_open = open.contains(demo.name());
            ui.toggle_value(&mut is_open, demo.name());
            App::set_open(open, demo.name(), is_open);
        }
    }

    fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
        if is_open {
            if !open.contains(key) {
                open.insert(key.to_owned());
            }
        } else {
            open.remove(key);
        }
    }

    pub fn show_toast(&mut self, caption: impl Into<String>, duration: u64) {
        self.toasts
            .info(caption)
            .set_duration(Duration::from_secs(duration).into());
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs_f32(10.0));

        self.toasts.show(ctx);
        self.windows(ctx);

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Open...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.picked_path = Some(path.display().to_string());
                        }
                        ui.close_menu();
                    }
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Organize Windows").clicked() {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |inner_ui| {
                inner_ui.separator();
                inner_ui.heading("Open Windows");
                self.checkboxes(inner_ui);
            });
        });

        // The central panel the region left after adding TopPanel's and SidePanel's
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };
                        if let Some(bytes) = &file.bytes {
                            use std::fmt::Write as _;
                            write!(info, " ({} bytes)", bytes.len()).ok();
                        }
                        ui.label(info);
                    }
                });
            }
            preview_files_being_dropped(ctx);

            // Collect dropped files:
            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    self.dropped_files = i.raw.dropped_files.clone();
                }
            });
        });
    }
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}
