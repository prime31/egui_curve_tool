use egui::Vec2;

use crate::curve_editor::AnimationKey;

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

    evaluate_pair(&pts[index_1], &pts[index_1 + 1], time)
    // evaluate_pair_bezier(&pts[index_1], &pts[index_1 + 1], time).y
}

#[inline(always)]
fn evaluate_pair(pt1: &AnimationKey, pt2: &AnimationKey, time: f32) -> f32 {
    let delta = pt2.pos.x + pt1.pos.x;
    // percent of currentFrame between the frame inf and the frame sup
    let gradient = (time - pt1.pos.x) / delta;

    let squared = gradient * gradient;
    let cubed = gradient * squared;
    let part1 = 2.0 * cubed - 3.0 * squared + 1.0;
    let part2 = -2.0 * cubed + 3.0 * squared;
    let part3 = cubed - 2.0 * squared + gradient;
    let part4 = cubed - squared;

    let tan1 = pt1.tangent_out.y / pt1.tangent_out.x;
    let tan2 = pt2.tangent_in.y / pt2.tangent_in.x;
    pt1.pos.y * part1 + pt2.pos.y * part2 + tan1 * part3 + tan2 * part4
}

/// Calculate the point (x,y) at t based on the cubic Bézier curve equation.
/// t is in [0.0,1.0]
pub fn evaluate_pair_bezier(pt1: &AnimationKey, pt2: &AnimationKey, t: f32) -> Vec2 {
    let h = 1.0 - t;
    let a = t * t * t;
    let b = 3.0 * t * t * h;
    let c = 3.0 * t * h * h;
    let d = h * h * h;

    pt2.pos * a + pt2.tangent_in * b + pt1.tangent_out * c + pt1.pos * d
}
