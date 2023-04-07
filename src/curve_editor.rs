fn sin(x: f64) -> f64 {
    x.sin() + 0.0001
}

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CurveEditor {
    show_axes: [bool; 2],
    allow_drag: bool,
    allow_zoom: bool,
    allow_scroll: bool,
    center_x_axis: bool,
    center_y_axis: bool,
    width: f32,
    height: f32,
}

impl Default for CurveEditor {
    fn default() -> Self {
        Self {
            show_axes: [true, true],
            allow_drag: true,
            allow_zoom: true,
            allow_scroll: true,
            center_x_axis: false,
            center_y_axis: false,
            width: 400.0,
            height: 200.0,
        }
    }
}

impl super::Demo for CurveEditor {
    fn name(&self) -> &'static str {
        "✋ Curve Editor"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use super::View;
        egui::Window::new(self.name())
            .vscroll(false)
            .resizable(false)
            .open(open)
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for CurveEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.example_plot(ui).context_menu(|ui| {
                egui::Grid::new("button_grid").show(ui, |ui| {
                    ui.add(egui::DragValue::new(&mut self.width).speed(1.0).prefix("Width:"));
                    ui.add(egui::DragValue::new(&mut self.height).speed(1.0).prefix("Height:"));
                    ui.end_row();
                    ui.checkbox(&mut self.show_axes[0], "x-Axis");
                    ui.checkbox(&mut self.show_axes[1], "y-Axis");
                    ui.end_row();
                    if ui.checkbox(&mut self.allow_drag, "Drag").changed()
                        || ui.checkbox(&mut self.allow_zoom, "Zoom").changed()
                        || ui.checkbox(&mut self.allow_scroll, "Scroll").changed()
                    {
                        ui.close_menu();
                    }
                });
            });
        });
    }
}

impl CurveEditor {
    fn example_plot(&self, ui: &mut egui::Ui) -> egui::Response {
        use egui::plot::{Line, PlotPoints};
        let n = 128;
        let line = Line::new(
            (0..=n)
                .map(|i| {
                    use std::f64::consts::TAU;
                    let x = egui::remap(i as f64, 0.0..=n as f64, -TAU..=TAU);
                    [x, sin(x)]
                })
                .collect::<PlotPoints>(),
        );
        egui::plot::Plot::new("example_plot")
            .show_axes(self.show_axes)
            .allow_drag(self.allow_drag)
            .allow_zoom(self.allow_zoom)
            .allow_scroll(self.allow_scroll)
            .center_x_axis(self.center_x_axis)
            .center_x_axis(self.center_y_axis)
            .width(self.width)
            .height(self.height)
            .data_aspect(1.0)
            .show(ui, |plot_ui| plot_ui.line(line))
            .response
    }
}
