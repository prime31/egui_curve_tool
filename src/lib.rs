#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TemplateApp;

// windows
mod bezier_curve_editor;
mod context_menu;
mod curve_editor;
mod paint_bezier;
mod plot_demo;

/// Something to view in the demo windows
pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Something to view
pub trait Demo {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}
