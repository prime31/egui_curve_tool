use std::f64::consts::TAU;

use eframe::emath::RectTransform;
use egui::plot::{CoordinatesFormatter, PlotBounds, PlotUi};
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
            tangent_in: Vec2::new(-0.01, 0.0),
            tangent_out: Vec2::new(0.01, 0.0),
            ..Default::default()
        }
    }
}

#[derive(PartialEq)]
struct LineDemo {
    circle_radius: f64,
    constrain_to_01: bool,
    dragging_my_circle: bool,
    my_circle_pos: Pos2,
    interact_pos: Option<Pos2>,
    points: Vec<AnimationKey>,
}

impl Default for LineDemo {
    fn default() -> Self {
        Self {
            circle_radius: 0.05,
            constrain_to_01: false,
            dragging_my_circle: false,
            my_circle_pos: Pos2::new(0., 0.),
            interact_pos: None,
            points: vec![
                AnimationKey::new(Vec2::new(0.0, 0.0)),
                AnimationKey::new(Vec2::new(1.0, 1.0)),
            ],
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

    fn my_circle(&self, trans: RectTransform) -> Line {
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

        let mut plot = Plot::new("lines_demo")
            // .allow_drag(false)
            // .allow_scroll(false)
            // .allow_zoom(false)
            .allow_boxed_zoom(false)
            .show_x(false)
            .show_y(false)
            .min_size(Vec2::new(128., 128.))
            .data_aspect(1.0)
            // .view_aspect(1.0)
            .height(ui.available_height() - 151.); // magic number for 3 lines of logs on the bottom

        if self.dragging_my_circle {
            plot = plot.coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
        }

        let InnerResponse {
            mut response,
            inner: (drag_delta, ptr_coord, bounds),
        } = plot.show(ui, |plot_ui| {
            // if self.constrain_to_01 {
            //     plot_ui.set_plot_bounds(PlotBounds::from_min_max([-0.1, -0.1], [1.1, 1.1]));
            // } else {
            //     plot_ui.set_plot_bounds(PlotBounds::from_min_max([-0.1, -1.1], [1.1, 1.1]));
            // }

            let min_pt = plot_ui.screen_from_plot(plot::PlotPoint::new(
                plot_ui.plot_bounds().min()[0],
                plot_ui.plot_bounds().min()[1],
            ));
            let max_pt = plot_ui.screen_from_plot(plot::PlotPoint::new(
                plot_ui.plot_bounds().max()[0],
                plot_ui.plot_bounds().max()[1],
            ));

            let actul_aspect = (max_pt.x - min_pt.x) / (min_pt.y - max_pt.y);
            let plot_aspect_ratio = plot_ui.plot_bounds().width() / plot_ui.plot_bounds().height();

            println!(
                "aspect: {:.2?}, min: {:?}, max: {:?}, aspect: {}",
                plot_aspect_ratio, min_pt, max_pt, actul_aspect
            );

            let min = plot_ui.plot_bounds().min().map(|i| i as f32);
            let max = plot_ui.plot_bounds().max().map(|i| i as f32);
            let from = Rect::from_min_max(min.into(), max.into());
            let to_screen = emath::RectTransform::from_to(from, Rect::from_min_max(min_pt, max_pt));

            plot_ui.line(self.my_circle(to_screen));

            plot_ui.ctx().input(|i| {
                if i.pointer.primary_clicked() {
                    self.interact_pos = i.pointer.interact_pos()
                }
                if i.pointer.primary_released() {
                    self.interact_pos = None;
                }
            });
            (
                plot_ui.pointer_coordinate_drag_delta(),
                plot_ui.pointer_coordinate(),
                plot_ui.plot_bounds(),
            )
        });

        if self.interact_pos.is_some() {
            if self.my_circle_pos.distance(ptr_coord.unwrap().to_pos2()) < self.circle_radius as f32 {
                self.dragging_my_circle = true;
                response = response.on_hover_cursor(CursorIcon::Move);
            }
        }
        if response.drag_released() {
            self.dragging_my_circle = false;
        }

        if self.dragging_my_circle {
            self.my_circle_pos += drag_delta;
        }

        if let Some(ptr_coord) = ptr_coord {
            ui.label(format!(
                "in circle {:?}, dragging circle: {}",
                self.my_circle_pos.distance(ptr_coord.to_pos2()) < self.circle_radius as f32,
                self.dragging_my_circle
            ));

            ui.label(format!(
                "interact pos {:.2},{:.2}, delta: {:.2},{:.2}, ptr coord: {:.2},{:.2}",
                self.interact_pos.unwrap_or(Pos2::ZERO).x,
                self.interact_pos.unwrap_or(Pos2::ZERO).y,
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
