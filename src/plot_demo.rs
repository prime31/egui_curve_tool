use std::f64::consts::TAU;

use egui::plot::{CoordinatesFormatter, PlotBounds, PlotPoint, PlotUi};
use egui::*;
use plot::{Corner, Line, LineStyle, Plot, PlotPoints};

// ----------------------------------------------------------------------------

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
                ui.label("Pan by dragging, or scroll (+ shift = horizontal).");
                ui.label("Box zooming: Right click to zoom in and zoom out using a selection.");
                if cfg!(target_arch = "wasm32") {
                    ui.label("Zoom with ctrl / ⌘ + pointer wheel, or with pinch gesture.");
                } else if cfg!(target_os = "macos") {
                    ui.label("Zoom with ctrl / ⌘ + scroll.");
                } else {
                    ui.label("Zoom with ctrl + scroll.");
                }
            });
        });
        ui.separator();

        self.line_demo.ui(ui);
    }
}

#[derive(Default, PartialEq)]
enum AnimationKeyInterpolation {
    #[default]
    None,
    Step,
}

#[derive(Default, PartialEq)]
struct AnimationKey {
    pos: Vec2,
    tangent_in: Vec2,
    tangent_out: Vec2,
    interopolation: AnimationKeyInterpolation,
    tangent_locked: bool,
}

impl AnimationKey {
    fn new(pos: Vec2) -> AnimationKey {
        AnimationKey {
            pos,
            tangent_in: Vec2::new(-0.03, 0.0),
            tangent_out: Vec2::new(0.03, 0.0),
            ..Default::default()
        }
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

#[derive(Default, PartialEq)]
enum AnimationKeyPointField {
    #[default]
    Pos,
    TanIn,
    TanOut,
}

#[derive(PartialEq)]
struct LineDemo {
    circle_radius: f64,
    constrain_to_01: bool,
    dragging_my_circle: bool,
    dragged_object: Option<(usize, AnimationKeyPointField)>,
    my_circle_pos: Pos2,
    left_click_pos: Option<Pos2>,
    points: Vec<AnimationKey>,
    points_for_drawing: Vec<AnimationKeyPoint>,
}

impl Default for LineDemo {
    fn default() -> Self {
        Self {
            circle_radius: 0.05,
            constrain_to_01: false,
            dragging_my_circle: false,
            dragged_object: None,
            my_circle_pos: Pos2::new(0., 0.),
            left_click_pos: None,
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
            ui.toggle_value(constrain_to_01, "0 - 1 Range");
        });
    }

    fn ensure_drawing_points_capacity(&mut self) {
        if self.points.len() != self.points_for_drawing.len() {
            self.points_for_drawing = self.points.iter().map(|p| p.into()).collect();
        }
    }

    fn add_animation_key(&mut self, pos: Vec2) {
        self.points.push(AnimationKey::new(pos));
        self.points_for_drawing.push(self.points.last().unwrap().into());
    }

    fn draw(&self, key: &AnimationKey, plot_ui: &mut PlotUi) {
        plot_ui.line(self.circle(key.pos, self.circle_radius));
        plot_ui.line(self.circle(key.pos + key.tangent_in, 0.1));
        plot_ui.line(self.circle(key.pos + key.tangent_out, 0.1));
    }

    fn circle(&self, pos: Vec2, radius: f64) -> Line {
        let n = 15;
        let circle_points: PlotPoints = (0..=n)
            .map(|i| {
                let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
                [radius * t.cos() + pos.x as f64, radius * t.sin() + pos.y as f64]
            })
            .collect();
        Line::new(circle_points)
            .color(Color32::from_rgb(100, 200, 100))
            .style(LineStyle::Solid)
    }

    fn my_circle(&self) -> Line {
        let n = 15;
        let circle_points: PlotPoints = (0..=n)
            .map(|i| {
                let t = remap(i as f64, 0.0..=(n as f64), 0.0..=TAU);
                let r = self.circle_radius;
                [
                    (r * t.cos() + self.my_circle_pos.x as f64),
                    (r * t.sin() + self.my_circle_pos.y as f64),
                ]
            })
            .collect();

        Line::new(circle_points)
            .color(Color32::from_rgb(100, 200, 100))
            .name("my circle")
    }

    fn thingy(&self) -> Line {
        Line::new(PlotPoints::from_parametric_callback(
            move |t| ((2.0 * t).sin(), (3.0 * t).sin()),
            0.0..=TAU,
            256,
        ))
        .color(Color32::from_rgb(100, 150, 250))
        .name("x = sin(2t), y = sin(3t)")
    }
}

impl LineDemo {
    fn ui(&mut self, ui: &mut Ui) -> Response {
        self.options_ui(ui);

        self.ensure_drawing_points_capacity();

        let mut plot = Plot::new("lines_demo")
            .allow_drag(!self.dragging_my_circle)
            .allow_scroll(!self.dragging_my_circle)
            .allow_zoom(!self.dragging_my_circle)
            .allow_boxed_zoom(!self.dragging_my_circle)
            .show_x(false)
            .show_y(false)
            .min_size(Vec2::new(128., 128.))
            // .data_aspect(1.0)
            // .view_aspect(1.0)
            .height(ui.available_height() - 151.); // magic number for 3 lines of logs on the bottom

        if self.dragging_my_circle {
            plot = plot.coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
        }

        let InnerResponse {
            mut response,
            inner: (drag_delta, ptr_coord, ptr_coord_screen, bounds),
        } = plot.show(ui, |plot_ui| {
            // if self.constrain_to_01 {
            //     plot_ui.set_plot_bounds(PlotBounds::from_min_max([-0.1, -0.1], [1.1, 1.1]));
            // } else {
            //     plot_ui.set_plot_bounds(PlotBounds::from_min_max([-0.1, -1.1], [1.1, 1.1]));
            // }

            // clamp x min of the graph
            let y_min = if self.constrain_to_01 { -0.1 } else { -1.1 };
            let mut min_bounds = plot_ui.plot_bounds().min();
            if min_bounds[0] != -0.1 || min_bounds[1] < y_min {
                min_bounds[0] = -0.1;
                min_bounds[1] = min_bounds[1].clamp(y_min, 100.0);

                let mut max_bounds = plot_ui.plot_bounds().max();
                let bounds = plot_ui.plot_bounds();
                max_bounds[0] -= bounds.min()[0] - min_bounds[0];
                max_bounds[1] -= bounds.min()[1] - min_bounds[1];

                plot_ui.set_plot_bounds(PlotBounds::from_min_max(min_bounds, max_bounds));
            }

            // clamp y max of the graph
            let mut max_bounds = plot_ui.plot_bounds().max();
            if max_bounds[1] > 1.1 {
                max_bounds[1] = max_bounds[1].clamp(y_min, 1.1);
                plot_ui.set_plot_bounds(PlotBounds::from_min_max(min_bounds, max_bounds));
            }

            max_bounds[0] = 1.1;
            plot_ui.set_plot_bounds(PlotBounds::from_min_max(min_bounds, max_bounds));

            let t = plot_ui.ctx().input(|i| {
                if i.pointer.primary_clicked() {
                    self.left_click_pos = i.pointer.interact_pos()
                }
                if i.pointer.primary_released() {
                    self.left_click_pos = None;
                }
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

        if self.left_click_pos.is_some() {
            if self.my_circle_pos.distance(ptr_coord.unwrap().to_pos2()) < self.circle_radius as f32 {
                self.dragging_my_circle = true;
                response = response.on_hover_cursor(CursorIcon::Move);
            }
        }
        if response.drag_released() {
            self.dragging_my_circle = false;
            self.my_circle_pos = self.my_circle_pos.clamp(Pos2::new(0., -1.), Pos2::new(1., 1.))
        }

        if self.dragging_my_circle {
            self.my_circle_pos += drag_delta;
        }

        for pt in &self.points_for_drawing {
            ui.painter().circle_filled(pt.pos.to_pos2(), 5.0, Color32::LIGHT_GREEN);
            ui.painter()
                .circle_filled(pt.tangent_in.to_pos2(), 3.0, Color32::LIGHT_RED);
            ui.painter()
                .circle_filled(pt.tangent_out.to_pos2(), 3.0, Color32::LIGHT_RED);
        }

        if let (Some(ptr_coord), Some(ptr_coord_screen)) = (ptr_coord, ptr_coord_screen) {
            if !self.dragging_my_circle && self.my_circle_pos.distance(ptr_coord.to_pos2()) < self.circle_radius as f32
            {
                response = response.on_hover_cursor(CursorIcon::Grab);
            }

            // ui.painter().circle_filled(ptr_coord_screen, 10.0, Color32::LIGHT_GREEN);

            ui.label(format!(
                "in circle {:?}, dragging circle: {}",
                self.my_circle_pos.distance(ptr_coord.to_pos2()) < self.circle_radius as f32,
                self.dragging_my_circle
            ));

            ui.label(format!(
                "interact pos {:.2},{:.2}, delta: {:.2},{:.2}, ptr coord: {:.2},{:.2}",
                self.left_click_pos.unwrap_or(Pos2::ZERO).x,
                self.left_click_pos.unwrap_or(Pos2::ZERO).y,
                drag_delta.x,
                drag_delta.y,
                ptr_coord.x,
                ptr_coord.y
            ));
        }

        let largest_range = {
            let x = bounds.max()[0] - bounds.min()[0];
            let y = bounds.max()[1] - bounds.min()[1];
            x.max(y)
        };
        // self.circle_radius = largest_range * 0.005;
        // self.plot_scale = largest_range / ui.available_size_before_wrap().x as f64;

        ui.label(format!(
            "plot bounds: min: {:.02?}, max: {:.02?}, largest_range: {:0.2?}",
            bounds.min(),
            bounds.max(),
            largest_range,
        ));

        response
    }
}
