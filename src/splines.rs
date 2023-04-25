use eframe::epaint::{CubicBezierShape, QuadraticBezierShape};
use egui::{Color32, Stroke, Vec2};

use crate::curve_editor::AnimationKey;

pub fn get_hermite(points: &Vec<AnimationKey>, curve_resolution: usize) -> Vec<[f64; 2]> {
    let mut pts: Vec<[f64; 2]> = Vec::new();

    for i in 0..=curve_resolution {
        let t = i as f32 / curve_resolution as f32;
        let value = evaluate(&points, t);
        pts.push([t as f64, value as f64]);
    }

    pts
}

pub fn evaluate(pts: &Vec<AnimationKey>, time: f32) -> f32 {
    if time <= pts[0].pos.x {
        return pts[0].pos.y;
    }
    if time >= pts[pts.len() - 1].pos.x {
        return pts[pts.len() - 1].pos.y;
    }

    // find the two pts we are looking at
    let mut index_1 = 0;
    for chunk in pts.windows(2) {
        if chunk[0].pos.x <= time && chunk[1].pos.x >= time {
            break;
        }
        index_1 += 1;
    }

    // translate time to a new t value in a 0-1 range between these 2 points
    let delta = pts[index_1 + 1].pos.x - pts[index_1].pos.x;
    let t = (time - pts[index_1].pos.x) / delta;

    evaluate_pair_bezier(&pts[index_1], &pts[index_1 + 1], t).y
}

/// Calculate the point (x,y) at t based on the cubic hermite curve equation.
/// t is in [0.0,1.0]
#[inline(always)]
fn evaluate_pair_hermite(pt1: &AnimationKey, pt2: &AnimationKey, time: f32) -> f32 {
    let squared = time * time;
    let cubed = time * squared;

    let part1 = 2.0 * cubed - 3.0 * squared + 1.0;
    let part2 = -2.0 * cubed + 3.0 * squared;
    let part3 = cubed - 2.0 * squared + time;
    let part4 = cubed - squared;

    let tan1 = pt1.tangent_out.y / pt1.tangent_out.x;
    let tan2 = pt2.tangent_in.y / pt2.tangent_in.x;
    pt1.pos.y * part1 + pt2.pos.y * part2 + tan1 * part3 + tan2 * part4
}

#[inline(always)]
fn evaluate_pair_hermite_first_derivative(pt1: &AnimationKey, pt2: &AnimationKey, time: f32) -> f32 {
    let t2 = time * time;

    let tan1 = pt1.tangent_out.y / pt1.tangent_out.x;
    let tan2 = pt2.tangent_in.y / pt2.tangent_in.x;
    (t2 - time) * 6. * pt1.pos.y
        + (3. * t2 - 4. * time + 1.) * tan1
        + (-t2 + time) * 6. * pt2.pos.y
        + (3. * t2 - 2. * time) * tan2
}

/// Calculate the point (x,y) at t based on the cubic Bézier curve equation.
/// t is in [0.0,1.0]
pub fn evaluate_pair_bezier(pt1: &AnimationKey, pt2: &AnimationKey, time: f32) -> Vec2 {
    let h = 1.0 - time;
    let a = time * time * time;
    let b = 3.0 * time * time * h;
    let c = 3.0 * time * h * h;
    let d = h * h * h;

    pt2.pos * a + pt2.tangent_in_world() * b + pt1.tangent_out_world() * c + pt1.pos * d
}

/// find a set of points that approximate the quadratic Bézier curve. the number of points is determined by the tolerance.
/// the points may not be evenly distributed in the range [0.0,1.0] (t value)
pub fn flatten(points: &Vec<AnimationKey>, tolerance: Option<f32>) {
    for chunk in points.windows(2) {
        let shape = CubicBezierShape {
            points: [
                chunk[0].pos.to_pos2(),
                chunk[0].pos.to_pos2(),
                chunk[0].pos.to_pos2(),
                chunk[0].pos.to_pos2(),
            ],
            closed: false,
            fill: Color32::RED,
            stroke: Stroke::default(),
        };
    }
}
