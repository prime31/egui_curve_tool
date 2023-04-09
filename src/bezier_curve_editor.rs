use egui::epaint::{CubicBezierShape, PathShape};
use egui::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct BezierCurveEditor {
    /// The control points. The [`Self::degree`] first of them are used.
    control_points: Vec<Pos2>,

    /// Stroke for Bézier curve.
    stroke: Stroke,

    /// Fill for Bézier curve.
    fill: Color32,

    /// Stroke for auxiliary lines.
    aux_stroke: Stroke,

    bounding_box_stroke: Stroke,
}

impl Default for BezierCurveEditor {
    fn default() -> Self {
        Self {
            control_points: vec![
                pos2(50.0, 50.0),
                pos2(60.0, 250.0),
                pos2(200.0, 200.0),
                pos2(250.0, 150.0),
                pos2(250.0, 150.0),
                pos2(250.0, 50.0),
                pos2(350.0, 200.0),
                pos2(350.0, 50.0),
            ],
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            fill: Color32::TRANSPARENT,
            aux_stroke: Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            bounding_box_stroke: Stroke::new(1.0, Color32::LIGHT_GREEN.linear_multiply(0.25)),
        }
    }
}

impl BezierCurveEditor {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Colors", |ui| {
            ui.horizontal(|ui| {
                ui.label("Fill color:");
                ui.color_edit_button_srgba(&mut self.fill);
            });
            egui::stroke_ui(ui, &mut self.stroke, "Curve Stroke");
            egui::stroke_ui(ui, &mut self.aux_stroke, "Auxiliary Stroke");
            egui::stroke_ui(ui, &mut self.bounding_box_stroke, "Bounding Box Stroke");
        });

        ui.collapsing("Global tessellation options", |ui| {
            let mut tessellation_options = ui.ctx().tessellation_options(|to| *to);
            tessellation_options.ui(ui);
            ui.ctx().tessellation_options_mut(|to| *to = tessellation_options);
        });

        ui.label("Move the points by dragging them.");
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

        let to_screen =
            emath::RectTransform::from_to(Rect::from_min_size(Pos2::ZERO, response.rect.size()), response.rect);

        let control_point_radius = 8.0;

        let control_point_shapes: Vec<Shape> = self
            .control_points
            .iter_mut()
            .enumerate()
            .map(|(i, point)| {
                let size = Vec2::splat(2.0 * control_point_radius);

                let point_in_screen = to_screen.transform_pos(*point);
                let point_rect = Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::drag());

                *point += point_response.drag_delta();
                *point = to_screen.from().clamp(*point);

                let point_in_screen = to_screen.transform_pos(*point);
                let stroke = ui.style().interact(&point_response).fg_stroke;

                Shape::circle_stroke(point_in_screen, control_point_radius, stroke)
            })
            .collect();

        let points_in_screen: Vec<Pos2> = self.control_points.iter().map(|p| to_screen * *p).collect();
        let fourples: Vec<[Pos2; 4]> = points_in_screen
            .windows(4)
            .map(|arr| [arr[0], arr[1], arr[2], arr[3]])
            .collect();

        for cubic_pts in fourples {
            let shape = CubicBezierShape::from_points_stroke(cubic_pts, true, self.fill, self.stroke);
            // painter.add(epaint::RectShape::stroke(
            //     shape.visual_bounding_rect(),
            //     0.0,
            //     self.bounding_box_stroke,
            // ));
            painter.add(shape);
        }

        painter.add(PathShape::line(points_in_screen, self.aux_stroke));
        painter.extend(control_point_shapes);

        response
    }
}

impl super::Demo for BezierCurveEditor {
    fn name(&self) -> &'static str {
        "👌 Bézier Curve Editor"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        use super::View as _;
        Window::new(self.name())
            .open(open)
            .vscroll(false)
            .resizable(true)
            .default_size([300.0, 550.0])
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for BezierCurveEditor {
    fn ui(&mut self, ui: &mut Ui) {
        self.ui_control(ui);

        Frame::canvas(ui.style()).show(ui, |ui| {
            self.ui_content(ui);
        });
    }
}
