use egui::plot::{CoordinatesFormatter, PlotBounds, PlotPoint, PlotUi};
use egui::*;
use egui_notify::Toasts;
use plot::{Corner, Line, LineStyle, Plot};

use crate::splines;

const POINT_RADIUS: f32 = 5.0;
const CONTROL_POINT_RADIUS: f32 = 3.0;
const CIRCLE_CLICK_RADIUS: f32 = 0.03;
const BOUNDS_OVERSHOOT: f64 = 0.2;
const TANGENT_LENGTH: f32 = 0.04;

const CURVE_COLOR: Color32 = Color32::LIGHT_BLUE;
const POINT_COLOR: Color32 = Color32::LIGHT_GREEN;
const CONTROL_POINT_COLOR: Color32 = Color32::DARK_GREEN;
const CONTROL_POINT_UNLOCKED_COLOR: Color32 = Color32::GREEN;
const CONTROL_POINT_LINE_COLOR: Color32 = Color32::LIGHT_GREEN;
const HOVERED_KEY_STROKE_COLOR: Color32 = Color32::LIGHT_RED;

#[derive(Default, PartialEq, Clone)]
pub struct AnimationKey {
    pub pos: Vec2,
    pub tangent_in: Vec2,
    pub tangent_out: Vec2,
    pub tangent_locked: bool,
}

impl AnimationKey {
    fn new(pos: Vec2) -> AnimationKey {
        AnimationKey {
            pos,
            tangent_in: vec2(-TANGENT_LENGTH, 0.0),
            tangent_out: vec2(TANGENT_LENGTH, 0.0),
            tangent_locked: true,
        }
    }

    fn tangent_in_screen(&self) -> Pos2 {
        (self.pos + self.tangent_in).to_pos2()
    }

    fn tangent_out_screen(&self) -> Pos2 {
        (self.pos + self.tangent_out).to_pos2()
    }

    fn intersects(&self, ptr_coord: Pos2) -> Option<AnimationKeyPointField> {
        let mut nearest_dist = f32::MAX;
        let mut nearest = None;

        let mut dist = self.pos.to_pos2().distance(ptr_coord);
        if dist < CIRCLE_CLICK_RADIUS && dist < nearest_dist {
            nearest_dist = dist;
            nearest = Some(AnimationKeyPointField::Pos);
        }

        dist = self.tangent_in_screen().distance(ptr_coord);
        if dist < CIRCLE_CLICK_RADIUS && dist < nearest_dist {
            nearest_dist = dist;
            nearest = Some(AnimationKeyPointField::TanIn);
        }

        dist = self.tangent_out_screen().distance(ptr_coord);
        if dist < CIRCLE_CLICK_RADIUS && dist < nearest_dist {
            nearest = Some(AnimationKeyPointField::TanOut);
        }
        return nearest;
    }

    fn translate(&mut self, key_field: &AnimationKeyPointField, delta: Vec2) {
        match key_field {
            AnimationKeyPointField::Pos => self.pos += delta,
            AnimationKeyPointField::TanIn => {
                self.tangent_in += delta;
                if self.tangent_locked {
                    self.tangent_out = -self.tangent_in;
                }
            }
            AnimationKeyPointField::TanOut => {
                self.tangent_out += delta;
                if self.tangent_locked {
                    self.tangent_in = -self.tangent_out;
                }
            }
        }
    }

    fn toggle_tangent(&mut self, key_field: AnimationKeyPointField) {
        if !self.tangent_locked {
            match key_field {
                AnimationKeyPointField::TanIn => self.tangent_out = -self.tangent_in,
                AnimationKeyPointField::TanOut => self.tangent_in = -self.tangent_out,
                _ => {}
            }
        }
        self.tangent_locked = !self.tangent_locked;
    }
}

impl Into<AnimationKeyPoint> for &AnimationKey {
    fn into(self) -> AnimationKeyPoint {
        AnimationKeyPoint {
            pos: self.pos,
            tangent_in: self.tangent_in,
            tangent_out: self.tangent_out,
        }
    }
}

#[derive(Default, PartialEq)]
struct AnimationKeyPoint {
    pos: Vec2,
    tangent_in: Vec2,
    tangent_out: Vec2,
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum AnimationKeyPointField {
    Pos,
    TanIn,
    TanOut,
}

#[derive(PartialEq)]
pub struct CurveEditor {
    constrain_to_01: bool,
    curve_resolution: usize,
    dragged_object: Option<(usize, AnimationKeyPointField)>,
    hovered_object: Option<(usize, AnimationKeyPointField)>,
    right_click_pos: Option<Pos2>,
    points: Vec<AnimationKey>,
    points_for_drawing: Vec<AnimationKeyPoint>,
}

impl Default for CurveEditor {
    fn default() -> Self {
        Self {
            constrain_to_01: false,
            curve_resolution: 50,
            dragged_object: None,
            hovered_object: None,
            right_click_pos: None,
            points: vec![
                AnimationKey::new(vec2(0.0, 0.0)),
                AnimationKey::new(vec2(0.5, 0.5)),
                AnimationKey::new(vec2(1.0, 1.0)),
            ],
            points_for_drawing: vec![],
        }
    }
}

impl super::Demo for CurveEditor {
    fn name(&self) -> &'static str {
        "🗠 Plot"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool, toasts: &mut Toasts) {
        Window::new(self.name())
            .open(open)
            .default_size(vec2(400.0, 400.0))
            .min_width(200.)
            .min_height(300.)
            // .fixed_size(vec2(400.0, 400.0))
            // .vscroll(true) // todo: temp fix for debug data on bottom
            .show(ctx, |ui| self.ui(ui, toasts));
    }
}

impl CurveEditor {
    fn ensure_drawing_points_capacity(&mut self) {
        if self.points.len() != self.points_for_drawing.len() {
            self.points_for_drawing = self.points.iter().map(|p| p.into()).collect();
        }
    }

    fn intersected_key(&self, ptr_coord: Pos2) -> Option<(usize, AnimationKeyPointField)> {
        for (i, pt) in self.points.iter().enumerate() {
            if let Some(key_point_field) = pt.intersects(ptr_coord) {
                // filter out first/last tangent on first/last element
                if (i == 0 && key_point_field == AnimationKeyPointField::TanIn)
                    || (i == self.points.len() - 1 && key_point_field == AnimationKeyPointField::TanOut)
                {
                    continue;
                }

                return Some((i, key_point_field));
            }
        }
        None
    }

    fn add_key(&mut self, pos: Vec2) {
        let y_min = if self.constrain_to_01 { 0. } else { -1. };
        let new_pos = pos.clamp(vec2(0., y_min), vec2(1., 1.));

        self.points.push(AnimationKey::new(new_pos));
        self.points_for_drawing.push(self.points.last().unwrap().into());
        self.points.sort_by(|a, b| a.pos.x.partial_cmp(&b.pos.x).unwrap());
    }

    fn update_dragged_object(&mut self, drag_delta: Vec2) {
        if let Some(ref mut dragged) = self.dragged_object {
            self.points[dragged.0].translate(&dragged.1, drag_delta);

            if dragged.1 == AnimationKeyPointField::Pos {
                let key = self.points[dragged.0].clone();
                self.points.sort_by(|a, b| a.pos.x.partial_cmp(&b.pos.x).unwrap());
                dragged.0 = self.points.iter().position(|e| *e == key).unwrap();
            }
        }
    }

    fn draw_curve(&self) -> Line {
        let mut pts: Vec<_> = self.points.iter().map(|f| [f.pos.x as f64, f.pos.y as f64]).collect();

        pts.clear();
        for i in 0..=self.curve_resolution {
            let t = i as f32 / self.curve_resolution as f32;
            pts.push([t as f64, splines::evaluate(&self.points, t) as f64]);
        }
        Line::new(pts).color(CURVE_COLOR).style(LineStyle::Solid)
    }

    fn draw_tangent_lines(&self, plot_ui: &mut PlotUi) {
        for (i, pt) in self.points.iter().enumerate() {
            let mut pts = Vec::with_capacity(3);

            let pos = [pt.pos.x as f64, pt.pos.y as f64];
            if i > 0 {
                pts.push([pos[0] + pt.tangent_in.x as f64, pos[1] + pt.tangent_in.y as f64]);
            }
            pts.push(pos);

            if i < self.points.len() - 1 {
                pts.push([pos[0] + pt.tangent_out.x as f64, pos[1] + pt.tangent_out.y as f64]);
            }

            plot_ui.line(Line::new(pts).color(CONTROL_POINT_LINE_COLOR));
        }
    }

    fn draw_keys(&self, painter: Painter) {
        for (i, pt) in self.points_for_drawing.iter().enumerate() {
            let ctrl_pt_color = if self.points[i].tangent_locked {
                CONTROL_POINT_COLOR
            } else {
                CONTROL_POINT_UNLOCKED_COLOR
            };

            // dont draw both tangents for first or last keys
            if i > 0 {
                painter.circle_filled(pt.tangent_in.to_pos2(), CONTROL_POINT_RADIUS, ctrl_pt_color);
            }

            painter.circle_filled(pt.pos.to_pos2(), POINT_RADIUS, POINT_COLOR);

            if i < self.points_for_drawing.len() - 1 {
                painter.circle_filled(pt.tangent_out.to_pos2(), CONTROL_POINT_RADIUS, ctrl_pt_color);
            }

            if let Some(hovered) = self.hovered_object {
                if hovered.0 == i {
                    let stroke = Stroke::new(2.0, HOVERED_KEY_STROKE_COLOR);
                    match hovered.1 {
                        AnimationKeyPointField::Pos => painter.circle_stroke(pt.pos.to_pos2(), POINT_RADIUS, stroke),
                        AnimationKeyPointField::TanIn => {
                            painter.circle_stroke(pt.tangent_in.to_pos2(), CONTROL_POINT_RADIUS, stroke)
                        }
                        AnimationKeyPointField::TanOut => {
                            painter.circle_stroke(pt.tangent_out.to_pos2(), CONTROL_POINT_RADIUS, stroke)
                        }
                    }
                }
            }
        }
    }
}

impl CurveEditor {
    fn ui(&mut self, ui: &mut Ui, toasts: &mut Toasts) -> Response {
        ui.horizontal(|ui| {
            ui.collapsing("Instructions", |ui| {
                ui.label("Command/Ctrl click tangent to toggle tangent lock (or right-click for menu).");
                ui.label("Alt click key to delete (or right click for menu).");
                ui.label("Alt click empty space to add a key (or right click for menu).");
            });
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.style_mut().wrap = Some(false);
            if ui
                .toggle_value(&mut self.constrain_to_01, "Constrain to 0 - 1 Range")
                .changed()
            {
                if self.constrain_to_01 {
                    for pt in &mut self.points {
                        (*pt).pos.y = pt.pos.y.clamp(0., 1.);
                    }
                }
            }

            ui.add(
                egui::Slider::new(&mut self.curve_resolution, 5..=500)
                    .logarithmic(true)
                    .text("Curve Resolution"),
            );

            egui::reset_button(ui, self);
        });

        self.ensure_drawing_points_capacity();

        let mut plot = Plot::new("lines_demo")
            .allow_drag(false && self.dragged_object.is_none())
            .allow_scroll(false && self.dragged_object.is_none())
            .allow_zoom(false && self.dragged_object.is_none())
            .allow_boxed_zoom(false && self.dragged_object.is_none())
            .allow_double_click_reset(false)
            .show_x(false)
            .show_y(false)
            .min_size(vec2(128., 128.))
            // .data_aspect(1.0)
            // .view_aspect(1.0)
            .height(ui.available_height() - 151.); // magic number for 3 lines of logs on the bottom

        if self.dragged_object.is_some() {
            plot = plot.coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
        }

        let InnerResponse {
            mut response,
            inner: (left_click_pos, drag_delta, ptr_coord, _ptr_coord_screen, bounds),
        } = plot.show(ui, |plot_ui| {
            // draw the curve
            plot_ui.line(self.draw_curve());
            self.draw_tangent_lines(plot_ui);

            let y_min = if self.constrain_to_01 { 0. } else { -1. };
            let min_bounds = [0. - BOUNDS_OVERSHOOT, y_min - BOUNDS_OVERSHOOT];
            let max_bounds = [1. + BOUNDS_OVERSHOOT, 1. + BOUNDS_OVERSHOOT];
            plot_ui.set_plot_bounds(PlotBounds::from_min_max(min_bounds, max_bounds));

            let left_click_pos = plot_ui.ctx().input(|i| {
                if i.pointer.primary_clicked() {
                    return i.pointer.interact_pos();
                }
                None
            });

            // convert to screen space for drawing
            for (i, pt) in self.points.iter().enumerate() {
                self.points_for_drawing[i].pos = plot_ui.screen_from_plot(PlotPoint::new(pt.pos.x, pt.pos.y)).to_vec2();
                self.points_for_drawing[i].tangent_in = plot_ui
                    .screen_from_plot(PlotPoint::new(pt.pos.x + pt.tangent_in.x, pt.pos.y + pt.tangent_in.y))
                    .to_vec2();
                self.points_for_drawing[i].tangent_out = plot_ui
                    .screen_from_plot(PlotPoint::new(pt.pos.x + pt.tangent_out.x, pt.pos.y + pt.tangent_out.y))
                    .to_vec2();
            }

            (
                left_click_pos,
                plot_ui.pointer_coordinate_drag_delta(),
                plot_ui.pointer_coordinate(),
                if let Some(pt) = plot_ui.pointer_coordinate() {
                    Some(plot_ui.screen_from_plot(pt))
                } else {
                    None
                },
                plot_ui.plot_bounds(),
            )
        });

        // check for click/drag
        if left_click_pos.is_some() && ptr_coord.is_some() {
            let ptr_coord = ptr_coord.unwrap().to_pos2();
            self.dragged_object = self.intersected_key(ptr_coord);
        }

        if response.drag_released() {
            if let Some(dragged) = &self.dragged_object {
                if dragged.1 == AnimationKeyPointField::Pos {
                    let y_min = if self.constrain_to_01 { 0. } else { -1. };
                    self.points[dragged.0].pos = self.points[dragged.0].pos.clamp(vec2(0., y_min), vec2(1., 1.));
                }
                self.dragged_object = None;
            }
        }

        // handle dragging keys
        self.update_dragged_object(drag_delta);

        // hover cursor if ptr is in plot rect
        if let Some(mut ptr_coord) = ptr_coord {
            if let Some(right_click_pos) = self.right_click_pos {
                ptr_coord = PlotPoint::new(right_click_pos.x as f64, right_click_pos.y as f64);
            }

            if let (None, Some(hovered)) = (&self.dragged_object, self.intersected_key(ptr_coord.to_pos2())) {
                self.hovered_object = Some(hovered);
                if hovered.1 != AnimationKeyPointField::Pos && ui.input(|i| i.modifiers.command) {
                    response = response.on_hover_cursor(CursorIcon::Crosshair);
                } else if hovered.1 == AnimationKeyPointField::Pos && ui.input(|i| i.modifiers.alt) {
                    response = response.on_hover_cursor(CursorIcon::NoDrop);
                }
            } else if ui.input(|i| i.modifiers.alt) {
                response = response.on_hover_cursor(CursorIcon::Copy);
            }

            ui.label(format!(
                "left click pos {:.2},{:.2}, delta: {:.5},{:.5}, ptr coord: {:.2},{:.2}",
                left_click_pos.unwrap_or(Pos2::ZERO).x,
                left_click_pos.unwrap_or(Pos2::ZERO).y,
                drag_delta.x,
                drag_delta.y,
                ptr_coord.x,
                ptr_coord.y
            ));
        }

        // draw the keys
        self.draw_keys(ui.painter_at(response.rect));

        if self.hovered_object.is_some() {
            let mut showing_contex_menu = false;
            response = response.context_menu(|ui| {
                showing_contex_menu = true;

                if let (None, Some(ptr_pos)) = (self.right_click_pos, ptr_coord) {
                    self.right_click_pos = Some(Pos2::new(ptr_pos.x as f32, ptr_pos.y as f32));
                }

                let hovered = self.hovered_object.as_ref().unwrap();
                match hovered.1 {
                    AnimationKeyPointField::Pos => {
                        if ui
                            .add_enabled_ui(self.points.len() > 2, |ui| {
                                if ui.button("Delete Key").clicked() {
                                    ui.close_menu();
                                    return true;
                                }
                                false
                            })
                            .inner
                        {
                            self.points.remove(hovered.0);
                            self.hovered_object = None;
                        }
                    }
                    _ => {
                        let text = if self.points[hovered.0].tangent_locked {
                            "Unlock Tangent"
                        } else {
                            "Lock Tangent"
                        };
                        if ui.button(text).clicked() {
                            self.points[hovered.0].toggle_tangent(hovered.1);
                            toasts.info("tangent lock toggled");
                            ui.close_menu();
                        }
                    }
                }
                ui.separator();
                if ui.button("Close").clicked() {
                    ui.close_menu();
                    self.hovered_object = None;
                }
            });

            // if the context menu is closed unset the hovered object and handle clicks
            if !showing_contex_menu {
                if response.clicked() {
                    let hovered = self.hovered_object.unwrap();
                    if hovered.1 != AnimationKeyPointField::Pos && ui.input(|i| i.modifiers.command) {
                        self.points[hovered.0].toggle_tangent(hovered.1);
                        toasts.info("tangent lock toggled");
                    } else if hovered.1 == AnimationKeyPointField::Pos && ui.input(|i| i.modifiers.alt) {
                        if self.points.len() > 2 {
                            self.points.remove(hovered.0);
                            toasts.info("key removed");
                        } else {
                            toasts.error("cannot remove key");
                        }
                    }
                }

                // self.hovered_object = None;
            }
        } else {
            let mut showing_contex_menu = false;
            response = response.context_menu(|ui| {
                showing_contex_menu = true;

                if let (None, Some(ptr_pos)) = (self.right_click_pos, ptr_coord) {
                    self.right_click_pos = Some(Pos2::new(ptr_pos.x as f32, ptr_pos.y as f32));
                }

                if ui.button("Add Key Here").clicked() {
                    self.add_key(self.right_click_pos.unwrap().to_vec2());
                    toasts.info("key added");
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Close").clicked() {
                    ui.close_menu();
                }
            });

            if !showing_contex_menu {
                self.right_click_pos = None;

                // alt click to add point
                if response.clicked() && ui.input(|i| i.modifiers.alt) {
                    self.add_key(ptr_coord.unwrap().to_vec2());
                    toasts.info("key added");
                }
            }
        }

        let largest_range = {
            let x = bounds.max()[0] - bounds.min()[0];
            let y = bounds.max()[1] - bounds.min()[1];
            x.max(y)
        };

        ui.label(format!(
            "plot bounds: min: {:.02?}, max: {:.02?}, largest_range: {:0.2?}, hovered: {:?}, right click: {:?}",
            bounds.min(),
            bounds.max(),
            largest_range,
            self.hovered_object,
            self.right_click_pos
        ));
        self.hovered_object = None;
        response
    }
}
