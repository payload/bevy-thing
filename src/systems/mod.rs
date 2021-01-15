pub mod context_map;
pub mod jabber;
pub mod steering;
pub mod texture_atlas_utils;

///
/// inspiration by Game Endeavor https://www.youtube.com/watch?v=6BrZryMz-ac
///
/// Paper by Andrew Frey "Context Steering" http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter18_Context_Steering_Behavior-Driven_Steering_at_the_Macro_Scale.pdf
use bevy::prelude::*;

use self::context_map::{ContextMap, ContextMapV};

pub fn steer_along_path(my_trans: &Transform) -> Vec2 {
    let get_direction = |pos| circle_nearest_tangent(Vec2::zero(), pos);
    let interest = interest(my_trans, Some(&get_direction));
    let danger = danger(my_trans, &[]);

    let _c = ContextMap::new(
        interest
            .weights
            .map_with_location(|r, _c, v| if danger.weights[r] > 0.0 { 0.0 } else { v })
    );

    interest.direction()
}

fn interest(my_trans: &Transform, get_direction: Option<&dyn Fn(Vec2) -> Vec2>) -> ContextMap {
    if let Some(get_direction) = get_direction {
        interest_in_direction(get_direction(pos(my_trans)))
    } else {
        interest_in_direction(forward(my_trans))
    }
}

fn danger(my_trans: &Transform, dangers: &[Vec2]) -> ContextMap {
    let mut c = ContextMap::default();
    let pos = pos(my_trans);

    for danger in dangers {
        c.add_map(*danger - pos, |w| w.max(0.0));
    }

    // TODO replace with calculating just the nearest angle/index of danger
    let mut max_i = 0;
    let mut max_v = 0.0;
    for i in 0..c.weights.len() {
        if c.weights[i] > max_v {
            max_i = i;
            max_v = c.weights[i];
        }
    }

    let mut c = ContextMap::new(ContextMapV::default());
    c.weights[max_i] = max_v;
    c
}

fn circle_nearest_tangent(center: Vec2, pos: Vec2) -> Vec2 {
    (pos - center).normalize().perp()
}

fn forward(trans: &Transform) -> Vec2 {
    angle_to_vec2(trans.rotation.z)
}

fn pos(trans: &Transform) -> Vec2 {
    trans.translation.truncate()
}

fn interest_in_direction(direction: Vec2) -> ContextMap {
    let mut c = ContextMap::default();
    c.add_map(direction, |w| w.max(0.0));
    c
}

fn angle_to_vec2(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin())
}
