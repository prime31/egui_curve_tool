use egui::plot::{CoordinatesFormatter, PlotBounds, PlotPoint, PlotUi};
use egui::*;
use plot::{Corner, Line, LineStyle, Plot};

const POINT_RADIUS: f32 = 5.0;
const CONTROL_POINT_RADIUS: f32 = 3.0;
const CIRCLE_CLICK_RADIUS: f32 = 0.02;
const BOUNDS_OVERSHOOT: f64 = 0.2;

const CURVE_COLOR: Color32 = Color32::DARK_GRAY;
const POINT_COLOR: Color32 = Color32::LIGHT_GREEN;
const CONTROL_POINT_COLOR: Color32 = Color32::DARK_GREEN;
const CONTROL_POINT_LINE_COLOR: Color32 = Color32::LIGHT_GREEN;

#[derive(PartialEq, Default)]
pub struct PlotDemo {
    line_demo: LineDemo,
}

impl super::Demo for PlotDemo {
    fn name(&self) -> &'static str {
        "🗠 Plot"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        use super::View as _;
        Window::new(self.name())
            .open(open)
            .default_size(vec2(400.0, 400.0))
            .min_width(200.)
            .min_height(300.)
            // .fixed_size(vec2(400.0, 400.0))
            .vscroll(true) // todo: temp fix for debug data on bottom
            .show(ctx, |ui| self.ui(ui));
    }
}

impl super::View for PlotDemo {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            egui::reset_button(ui, self);
            ui.collapsing("Instructions", |ui| {
                ui.label("Command/Ctrl click tangent to toggle tangent lock (or right-click for menu).");
                ui.label("Alt click key to delete (or right click for menu).");
                ui.label("Alt click empty space to add a key (or right click for menu).");
            });
        });
        ui.separator();

        self.line_demo.ui(ui);
    }
}

#[derive(Default, PartialEq, Clone)]
struct AnimationKey {
    pos: Vec2,
    tangent_in: Vec2,
    tangent_out: Vec2,
    tangent_locked: bool,
}

impl AnimationKey {
    fn new(pos: Vec2) -> AnimationKey {
        AnimationKey {
            pos,
            tangent_in: Vec2::new(-0.03, 0.0),
            tangent_out: Vec2::new(0.03, 0.0),
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
        if self.pos.to_pos2().distance(ptr_coord) < CIRCLE_CLICK_RADIUS {
            return Some(AnimationKeyPointField::Pos);
        }

        if self.tangent_in_screen().distance(ptr_coord) < CIRCLE_CLICK_RADIUS {
            return Some(AnimationKeyPointField::TanIn);
        }

        if self.tangent_out_screen().distance(ptr_coord) < CIRCLE_CLICK_RADIUS {
            return Some(AnimationKeyPointField::TanOut);
        }
        return None;
    }

    fn translate(&mut self, key_field: &AnimationKeyPointField, delta: Vec2) {
        match key_field {
            AnimationKeyPointField::Pos => self.pos += delta,
            AnimationKeyPointField::TanIn => {
                self.tangent_in += delta;
                if self.tangent_locked {
                    self.tangent_out -= delta;
                }
            }
            AnimationKeyPointField::TanOut => {
                self.tangent_out += delta;
                if self.tangent_locked {
                    self.tangent_in -= delta;
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
struct LineDemo {
    circle_radius: f64,
    constrain_to_01: bool,
    dragged_object: Option<(usize, AnimationKeyPointField)>,
    hovered_object: Option<(usize, AnimationKeyPointField)>,
    points: Vec<AnimationKey>,
    points_for_drawing: Vec<AnimationKeyPoint>,
}

impl Default for LineDemo {
    fn default() -> Self {
        Self {
            circle_radius: 0.05,
            constrain_to_01: false,
            dragged_object: None,
            hovered_object: None,
            points: vec![
                AnimationKey::new(Vec2::new(0.0, 0.0)),
                AnimationKey::new(Vec2::new(0.5, 0.5)),
                AnimationKey::new(Vec2::new(1.0, 1.0)),
            ],
            points_for_drawing: vec![],
        }
    }
}

impl LineDemo {
    fn options_ui(&mut self, ui: &mut Ui) {
        let Self {
            circle_radius,
            constrain_to_01,
            ..
        } = self;

        ui.horizontal(|ui| {
            ui.style_mut().wrap = Some(false);
            ui.add(egui::Slider::new(circle_radius, 1.0..=1000.0));
            if ui.toggle_value(constrain_to_01, "0 - 1 Range").changed() {
                if *constrain_to_01 {
                    for pt in &mut self.points {
                        (*pt).pos.y = pt.pos.y.clamp(0., 1.);
                    }
                }
            }
        });
    }

    fn ensure_drawing_points_capacity(&mut self) {
        if self.points.len() != self.points_for_drawing.len() {
            self.points_for_drawing = self.points.iter().map(|p| p.into()).collect();
        }
    }

    fn intersected_key(&self, ptr_coord: Pos2) -> Option<(usize, AnimationKeyPointField)> {
        for (i, pt) in self.points.iter().enumerate() {
            if let Some(key_point_field) = pt.intersects(ptr_coord) {
                return Some((i, key_point_field));
            }
        }
        None
    }

    fn add_animation_key(&mut self, pos: Vec2) {
        self.points.push(AnimationKey::new(pos));
        self.points_for_drawing.push(self.points.last().unwrap().into());
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

    fn curve(&self) -> Line {
        let pts: Vec<_> = self.points.iter().map(|f| [f.pos.x as f64, f.pos.y as f64]).collect();
        Line::new(pts).color(CURVE_COLOR).style(LineStyle::Solid)
    }

    fn tangent_lines(&self, plot_ui: &mut PlotUi) {
        for pt in &self.points {
            let pos = [pt.pos.x as f64, pt.pos.y as f64];
            let pts = vec![
                [pos[0] + pt.tangent_in.x as f64, pos[1] + pt.tangent_in.y as f64],
                pos,
                [pos[0] + pt.tangent_out.x as f64, pos[1] + pt.tangent_out.y as f64],
            ];
            plot_ui.line(Line::new(pts).color(CONTROL_POINT_LINE_COLOR));
        }
    }
}

impl LineDemo {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        self.options_ui(ui);

        self.ensure_drawing_points_capacity();

        let mut plot = Plot::new("lines_demo")
            .allow_drag(false && self.dragged_object.is_none())
            .allow_scroll(false && self.dragged_object.is_none())
            .allow_zoom(false && self.dragged_object.is_none())
            .allow_boxed_zoom(false && self.dragged_object.is_none())
            .allow_double_click_reset(false)
            .show_x(false)
            .show_y(false)
            .min_size(Vec2::new(128., 128.))
            // .data_aspect(1.0)
            // .view_aspect(1.0)
            .height(ui.available_height() - 151.); // magic number for 3 lines of logs on the bottom

        if self.dragged_object.is_some() {
            plot = plot.coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
        }

        let InnerResponse {
            mut response,
            inner: (left_click_pos, drag_delta, ptr_coord, ptr_coord_screen, bounds),
        } = plot.show(ui, |plot_ui| {
            // draw the curve
            plot_ui.line(self.curve());
            self.tangent_lines(plot_ui);

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

            // convert to screen spa
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

        let painter = ui.painter_at(response.rect);

        // check for click/drag
        if left_click_pos.is_some() && ptr_coord.is_some() {
            let ptr_coord = ptr_coord.unwrap().to_pos2();
            self.dragged_object = self.intersected_key(ptr_coord);
        }

        if response.drag_released() {
            if let Some(dragged) = &self.dragged_object {
                if dragged.1 == AnimationKeyPointField::Pos {
                    let y_min = if self.constrain_to_01 { 0. } else { -1. };
                    self.points[dragged.0].pos = self.points[dragged.0]
                        .pos
                        .clamp(Vec2::new(0., y_min), Vec2::new(1., 1.));
                }
                self.dragged_object = None;
            }
        }

        // handle dragging keys
        self.update_dragged_object(drag_delta);

        // draw the keys
        for pt in &self.points_for_drawing {
            painter.circle_filled(pt.pos.to_pos2(), POINT_RADIUS, POINT_COLOR);
            painter.circle_filled(pt.tangent_in.to_pos2(), CONTROL_POINT_RADIUS, CONTROL_POINT_COLOR);
            painter.circle_filled(pt.tangent_out.to_pos2(), CONTROL_POINT_RADIUS, CONTROL_POINT_COLOR);
        }

        // hover cursor if ptr is in plot rect
        if let Some(ptr_coord) = ptr_coord {
            if let (None, Some(hovered)) = (&self.dragged_object, self.intersected_key(ptr_coord.to_pos2())) {
                self.hovered_object = Some(hovered);
                if hovered.1 != AnimationKeyPointField::Pos && ui.input(|i| i.modifiers.command) {
                    response = response.on_hover_cursor(CursorIcon::Crosshair);
                } else if hovered.1 == AnimationKeyPointField::Pos && ui.input(|i| i.modifiers.alt) {
                    response = response.on_hover_cursor(CursorIcon::NoDrop);
                } else {
                    response = response.on_hover_cursor(CursorIcon::Grab);
                }
            }

            ui.label(format!(
                "left click pos {:.2},{:.2}, delta: {:.2},{:.2}, ptr coord: {:.2},{:.2}",
                left_click_pos.unwrap_or(Pos2::ZERO).x,
                left_click_pos.unwrap_or(Pos2::ZERO).y,
                drag_delta.x,
                drag_delta.y,
                ptr_coord.x,
                ptr_coord.y
            ));
        }

        if self.hovered_object.is_some() {
            let mut showing_contex_menu = false;
            response = response.context_menu(|ui| {
                showing_contex_menu = true;

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
                    } else if hovered.1 == AnimationKeyPointField::Pos
                        && ui.input(|i| i.modifiers.alt)
                        && self.points.len() > 2
                    {
                        self.points.remove(hovered.0);
                    }
                }

                self.hovered_object = None;
            }
        } else {
            response = response.context_menu(|ui| {
                if ui.button("Add Key Here").clicked() {
                    println!("pos to add: {:?}", ptr_coord);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Close").clicked() {
                    ui.close_menu();
                }
            });
        }

        let largest_range = {
            let x = bounds.max()[0] - bounds.min()[0];
            let y = bounds.max()[1] - bounds.min()[1];
            x.max(y)
        };

        ui.label(format!(
            "plot bounds: min: {:.02?}, max: {:.02?}, largest_range: {:0.2?}, hovered: {:?}",
            bounds.min(),
            bounds.max(),
            largest_range,
            self.hovered_object
        ));

        response
    }
}
