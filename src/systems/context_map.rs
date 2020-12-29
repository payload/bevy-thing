///
/// inspiration by Game Endeavor https://www.youtube.com/watch?v=6BrZryMz-ac
///
/// Paper by Andrew Frey "Context Steering" http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter18_Context_Steering_Behavior-Driven_Steering_at_the_Macro_Scale.pdf
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use float_eq::assert_float_eq;
use std::{f32::consts::PI, fmt::Display};

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
        let i = 4 + (angle * 4.0 / PI).round() as isize;
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

    fn max_index(&self) -> usize {
        let mut index = 0;
        let mut mag = 0.0;
        for i in 0..8 {
            if self.weights[i] >= mag {
                mag = self.weights[i];
                index = i;
            }
        }
        index
    }

    pub fn max_as_vec2(&self) -> Vec2 {
        self.index_to_vec2(self.max_index())
    }

    pub fn max_as_norm_vec2(&self) -> Vec2 {
        self.index_to_norm_vec2(self.max_index())
    }

    pub fn index_to_norm_vec2(&self, index: usize) -> Vec2 {
        let angle = ANGLES[index];
        Vec2::new(angle.cos(), angle.sin())
    }

    pub fn index_to_vec2(&self, index: usize) -> Vec2 {
        self.index_to_norm_vec2(index) * self.weights[index]
    }

    pub fn index_to_vec2_muladd(&self, index: usize, mul: f32, add: f32) -> Vec2 {
        let mag = self.weights[index];
        self.index_to_norm_vec2(index) * (add + mul * mag)
    }
}

pub fn example() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(example_setup.system())
        .add_system(context_map_gizmo_system.system())
        .add_system(example_update.system())
        .run();
}

pub fn spawn_context_map_gizmo(
    context_map: &ContextMap,
    gizmo: &Gizmo,
    cmds: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let pointv = |v: Vec2| point(v.x, v.y);
    let map_point = |i, r| pointv(context_map.index_to_vec2_muladd(i, r, gizmo.multiply));
    let mut points = Vec::with_capacity(8 * 3);
    for i in 0..8 {
        points.push(map_point(i, 0.0));
        points.push(map_point(i, gizmo.radius));
        points.push(map_point(i, 0.0));
    }

    cmds.spawn(primitive(
        materials.add(gizmo.color.into()),
        meshes,
        ShapeType::Polyline {
            points,
            closed: true,
        },
        TessellationMode::Stroke(&Default::default()),
        Vec3::zero(),
    ))
    .current_entity()
    .unwrap()
}

#[derive(Default, Debug)]
pub struct Gizmo {
    pub color: Color,
    pub radius: f32,
    pub multiply: f32,
    pub gizmo_entity: Option<Entity>,
}

impl Gizmo {
    pub fn new(color: Color, radius: f32) -> Self {
        Self {
            color,
            radius,
            multiply: radius,
            gizmo_entity: None,
        }
    }
}

pub fn context_map_gizmo_system(
    cmds: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(Entity, Mut<Gizmo>, &ContextMap)>,
) {
    for (entity, mut gizmo, context_map) in query.iter_mut() {
        for child in gizmo.gizmo_entity {
            cmds.despawn(child);
        }

        let child = spawn_context_map_gizmo(context_map, &gizmo, cmds, &mut materials, &mut meshes);

        cmds.push_children(entity, &[child]);
        gizmo.gizmo_entity = Some(child);
    }
}

fn example_setup(
    cmds: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    cmds.spawn({
        let mut bundle = Camera2dBundle::default();
        bundle.transform.scale = Vec3::new(0.5, 0.5, 1.0);
        bundle
    });

    let mut c = ContextMap::new(Vector8::zeros());
    c.add_interest(Vec2::new(2.0, 0.1), |len| len * 2.0);
    c.add_interest(Vec2::new(-3.0, -1.0), |len| len * 2.0);
    c.add_interest(Vec2::new(0.0, 2.0), |len| -len * 2.0);

    assert_float_eq!(c.max_as_norm_vec2().x, 1.0, abs <= 0.0001);
    assert_float_eq!(c.max_as_norm_vec2().y, 0.0, abs <= 0.0001);
    assert!(c.max_as_vec2().length() > 1.0);

    let context_map = ContextMap::new(Vector8::new_random() - Vector8::new_random());
    spawn_context_map_gizmo(
        &context_map,
        &Gizmo::new(Color::WHITE, 20.0),
        cmds,
        &mut materials,
        &mut meshes,
    );

    cmds.spawn((
        Transform::from_translation(Vec3::new(-100.0, -100.0, 0.0)),
        GlobalTransform::default(),
        context_map,
        Gizmo::new(Color::ORANGE_RED, 30.0),
    ));
}

fn example_update(mut query: Query<Mut<ContextMap>>) {
    for mut context_map in query.iter_mut() {
        context_map.weights = Vector8::new_random() - Vector8::new_random();
    }
}
