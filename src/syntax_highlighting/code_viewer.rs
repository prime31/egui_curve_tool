use egui::{vec2, Context, Response, Ui, Window};
use egui_notify::Toasts;

#[derive(PartialEq)]
pub struct CodeViewer {}

impl Default for CodeViewer {
    fn default() -> Self {
        Self {}
    }
}

impl super::super::Demo for CodeViewer {
    fn name(&self) -> &'static str {
        "🗠 Code Viewer"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool, toasts: &mut Toasts) {
        Window::new(self.name())
            .open(open)
            .default_size(vec2(400.0, 400.0))
            // .fixed_size(vec2(400.0, 400.0))
            .vscroll(true)
            .show(ctx, |ui| self.ui(ui, toasts));
    }
}

impl CodeViewer {
    fn ui(&mut self, ui: &mut Ui, _toasts: &mut Toasts) -> Response {
        show_code(
            ui,
            r#"
  if ui.button("Save").clicked() {
      my_state.save();
  }
  "#,
        );

        show_code(
            ui,
            r#"
            // Putting things on the same line using ui.horizontal:
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            "#,
        );

        show_code(
            ui,
            r#"
            // Putting things on the same line using ui.horizontal:
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });"#,
        )
    }
}

fn show_code(ui: &mut egui::Ui, code: &str) -> Response {
    let code = remove_leading_indentation(code.trim_start_matches('\n'));
    crate::syntax_highlighting::code_view_ui(ui, &code)
}

fn remove_leading_indentation(code: &str) -> String {
    fn is_indent(c: &u8) -> bool {
        matches!(*c, b' ' | b'\t')
    }

    let first_line_indent = code.bytes().take_while(is_indent).count();

    let mut out = String::new();

    let mut code = code;
    while !code.is_empty() {
        let indent = code.bytes().take_while(is_indent).count();
        let start = first_line_indent.min(indent);
        let end = code.find('\n').map_or_else(|| code.len(), |endline| endline + 1);
        out += &code[start..end];
        code = &code[end..];
    }
    out
}
