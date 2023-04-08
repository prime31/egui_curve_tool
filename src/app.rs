use egui::{Context, Ui};

use crate::{bezier_curve_editor, context_menu, curve_editor, paint_bezier, plot_demo};
use egui_notify::Toasts;

use super::Demo;
use std::{
    collections::BTreeSet,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    picked_path: Option<String>,
    #[serde(skip)]
    dropped_files: Vec<egui::DroppedFile>,

    value: f32,

    #[serde(skip)]
    demos: Vec<Box<dyn Demo>>,
    #[serde(skip)]
    open: BTreeSet<String>,
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let demos: Vec<Box<dyn Demo>> = vec![
            Box::new(paint_bezier::PaintBezier::default()),
            Box::new(context_menu::ContextMenus::default()),
            Box::new(plot_demo::PlotDemo::default()),
            Box::new(curve_editor::CurveEditor::default()),
            Box::new(bezier_curve_editor::BezierCurveEditor::default()),
        ];
        let mut open = BTreeSet::new();
        open.insert("fart".to_owned());

        Self {
            label: "foobar".into(),
            picked_path: None,
            dropped_files: vec![],
            value: 2.0,
            demos,
            open,
            toasts: Toasts::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn windows(&mut self, ctx: &Context) {
        let Self { demos, open, .. } = self;
        for demo in demos {
            let mut is_open = open.contains(demo.name());
            demo.show(ctx, &mut is_open);
            TemplateApp::set_open(open, demo.name(), is_open);
        }
    }

    pub fn checkboxes(&mut self, ui: &mut Ui) {
        let Self { demos, open, .. } = self;
        for demo in demos {
            let mut is_open = open.contains(demo.name());
            ui.toggle_value(&mut is_open, demo.name());
            TemplateApp::set_open(open, demo.name(), is_open);
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

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs_f32(self.value));

        self.toasts.show(ctx);
        self.windows(ctx);

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                (*self).value += 1.0;
                self.show_toast("fuck you", 3);
            }

            #[cfg(not(target_arch = "wasm32"))]
            ui.label(format!(
                "Repainting the UI each frame. FPS: {:?}",
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
            ));

            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |inner_ui| {
                inner_ui.separator();
                inner_ui.label("Windows");
                self.checkboxes(inner_ui);

                inner_ui.separator();
                if inner_ui.button("Organize windows").clicked() {
                    inner_ui.ctx().memory_mut(|mem| mem.reset_areas());
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Open file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.picked_path = Some(path.display().to_string());
                }
            }
            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }
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

        if true {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
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
