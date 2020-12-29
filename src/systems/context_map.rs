///
/// inspiration by Game Endeavor https://www.youtube.com/watch?v=6BrZryMz-ac
///
/// Paper by Andrew Frey "Context Steering" http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter18_Context_Steering_Behavior-Driven_Steering_at_the_Macro_Scale.pdf
use bevy::prelude::*;
use std::{cmp::Ordering, f32::consts::PI, fmt::Display};

use crate::bevy_rapier_utils::na;

type Vector8 = na::VectorN<f32, na::base::U8>;

#[derive(Default, Debug, Clone)]
pub struct ContextMap {
    pub weights: Vector8,
}

impl Display for ContextMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.weights.iter()).finish()
    }
}

const ANGLES: [f32; 8] = [
    0.0 / 4.0 * PI,
    1.0 / 4.0 * PI,
    2.0 / 4.0 * PI,
    3.0 / 4.0 * PI,
    4.0 / 4.0 * PI,
    5.0 / 4.0 * PI,
    6.0 / 4.0 * PI,
    7.0 / 4.0 * PI,
];

impl ContextMap {
    pub fn new(weights: Vector8) -> Self {
        Self { weights }
    }

    fn angle_to_index(angle: f32) -> usize {
        let i = 4 + (angle * 4.0 / PI).round() as usize;
        if 0 <= i && i <= 7 {
            i as usize
        } else {
            ((if i < 0 { i + 8 } else { i }) % 8) as usize
        }
    }

    fn vec2_to_index(vec: Vec2) -> usize {
        Self::angle_to_index(vec.y.atan2(vec.x))
    }

    pub fn add_interest(&mut self, vec: Vec2, length_func: impl FnOnce(f32) -> f32) {
        self.weights[Self::vec2_to_index(vec)] = length_func(vec.length_squared());
    }

    pub fn add_desinterest(&mut self, vec: Vec2, length_func: impl FnOnce(f32) -> f32) {
        self.weights[Self::vec2_to_index(vec)] = -length_func(vec.length_squared());
    }

    fn max_index(&self) -> usize {
        self.weights
            .iter()
            .enumerate()
            .max_by(|a: &(usize, &f32), b: &(usize, &f32)| {
                a.1.partial_cmp(&b.1).unwrap_or(Ordering::Less)
            })
            .map(|(i, _)| i as usize)
            .unwrap()
    }

    pub fn max_as_vec2(&self) -> Vec2 {
        let index = self.max_index();
        let mag = self.weights[index];
        let angle = ANGLES[index];
        Vec2::new(angle.cos(), angle.sin()) * mag
    }
}

pub fn example() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(example_setup.system())
        .run();
}

fn example_setup(cmds: &mut Commands) {
    cmds.spawn({
        let mut bundle = Camera2dBundle::default();
        bundle.transform.scale = Vec3::new(0.5, 0.5, 1.0);
        bundle
    });

    let v: Vector8 = Vector8::zeros();
    let mut c = ContextMap::new(v.clone());
    println!("{:.1}", &c);
    c.add_interest(Vec2::new(0.0, 1.0), |len| len * 2.0);
    println!("{:.1}", &c);
    c.add_interest(Vec2::new(-0.5, -0.5), |len| len * 2.0);
    println!("{:.1}", &c);
    println!("{:.1}", &c);
}
