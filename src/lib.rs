#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::App;
use egui_notify::Toasts;

mod curve_editor;
#[allow(dead_code)]
mod syntax_highlighting;

/// Something to view
pub trait Demo {
    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, app: &mut Toasts);
}
