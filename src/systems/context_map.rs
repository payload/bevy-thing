///
/// inspiration by Game Endeavor https://www.youtube.com/watch?v=6BrZryMz-ac
///
/// Paper by Andrew Frey "Context Steering" http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter18_Context_Steering_Behavior-Driven_Steering_at_the_Macro_Scale.pdf
use bevy::{prelude::*, render::camera::Camera};
use bevy_prototype_lyon::prelude::*;
use std::{f32::consts::PI, fmt::Display};

use crate::bevy_rapier_utils::na;

pub type ContextMapV = na::VectorN<f32, na::base::U16>;

#[derive(Default, Debug, Clone)]
pub struct ContextMap {
    pub weights: ContextMapV,
}

impl Display for ContextMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.weights.iter()).finish()
    }
}

impl ContextMap {
    pub fn new(weights: ContextMapV) -> Self {
        Self { weights }
    }

    fn angle_to_index(&self, angle: f32) -> usize {
        let res = self.weights.len() as isize;
        let i = res / 2 + (angle * 0.5 * res as f32 / PI).round() as isize;
        if 0 <= i && i < res {
            i as usize
        } else {
            ((if i < 0 { i + res } else { i }) % res) as usize
        }
    }

    fn vec2_to_index(&self, vec: Vec2) -> usize {
        self.angle_to_index(vec.y.atan2(vec.x))
    }

    pub fn add(&mut self, vec: Vec2) {
        let index = self.vec2_to_index(vec);
        self.weights[index] = vec.length();
    }

    pub fn add_map(&mut self, vec: Vec2, map: impl Fn(f32) -> f32) {
        let res = self.weights.len();
        for i in 0..res {
            let slot = self.index_to_norm_vec2(i);
            let w = vec.dot(slot);
            self.weights[i] = self.weights[i].max(map(w));
        }
    }

    fn max_index(&self) -> usize {
        let res = self.weights.len();
        let mut index = 0;
        let mut mag = 0.0;
        for i in 0..res {
            if self.weights[i] >= mag {
                mag = self.weights[i];
                index = i;
            }
        }
        index
    }

    fn get_angle(&self, index: usize) -> f32 {
        index as f32 * 2.0 * PI / self.weights.len() as f32
    }

    pub fn max_as_vec2(&self) -> Vec2 {
        self.index_to_vec2(self.max_index())
    }

    pub fn index_to_norm_vec2(&self, index: usize) -> Vec2 {
        let angle = self.get_angle(index);
        Vec2::new(angle.cos(), angle.sin())
    }

    pub fn index_to_vec2(&self, index: usize) -> Vec2 {
        self.index_to_norm_vec2(index) * self.weights[index]
    }

    pub fn index_to_vec2_muladd(&self, index: usize, mul: f32, add: f32) -> Vec2 {
        let mag = self.weights[index];
        self.index_to_norm_vec2(index) * (add + mul * mag)
    }

    pub fn direction(&self) -> Vec2 {
        let mut dir = Vec2::zero();

        for i in 0..self.weights.len() {
            dir += self.index_to_vec2(i);
        }

        if dir == Vec2::zero() {
            dir
        } else {
            dir.normalize()
        }
    }
}

pub fn spawn_context_map_gizmo(
    context_map: &ContextMap,
    gizmo: &Gizmo,
    cmds: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let res = context_map.weights.len();
    let pointv = |v: Vec2| point(v.x, v.y);
    let map_point = |i, r| pointv(context_map.index_to_vec2_muladd(i, r, gizmo.multiply));
    let mut points = Vec::with_capacity(res * 3);
    for i in 0..res {
        points.push(map_point(i, 0.0));
        points.push(map_point(i, gizmo.inner_radius));
        points.push(map_point(i, 0.0));
    }

    cmds.spawn(primitive(
        materials.add(gizmo.color.into()),
        meshes,
        ShapeType::Polyline {
            points,
            closed: true,
        },
        TessellationMode::Stroke(&StrokeOptions::default()),
        Vec3::zero(),
    ))
    .current_entity()
    .unwrap()
}

pub fn spawn_context_map_ai_gizmo(
    ai: &ContextMapAI,
    gizmo: &Gizmo,
    cmds: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let interests_len = ai.interests.weights.len();
    let dangers_len = ai.dangers.weights.len();
    let pointv = |v: Vec2| point(v.x, v.y);

    let mut polyline = |color, points| {
        primitive(
            color,
            meshes,
            ShapeType::Polyline {
                points,
                closed: true,
            },
            TessellationMode::Stroke(&Default::default()),
            Vec3::zero(),
        )
    };

    let mut interests = Vec::with_capacity(interests_len * 3);
    for (i, &w) in ai.interests.weights.iter().enumerate() {
        let vec = ai.interests.index_to_norm_vec2(i);
        interests.push(pointv(vec * gizmo.inner_radius));
        interests.push(pointv(vec * gizmo.inner_radius + vec * w * gizmo.multiply));
        interests.push(pointv(vec * gizmo.inner_radius));
    }

    let mut dangers = Vec::with_capacity(dangers_len * 3);
    for (i, &w) in ai.dangers.weights.iter().enumerate() {
        let vec = ai.dangers.index_to_norm_vec2(i);
        dangers.push(pointv(vec * gizmo.inner_radius));
        dangers.push(pointv(vec * gizmo.inner_radius + vec * w * gizmo.multiply));
        dangers.push(pointv(vec * gizmo.inner_radius));
    }

    let mut ring = Vec::with_capacity(interests_len);
    for i in 0..interests_len {
        let vec = ai.interests.index_to_norm_vec2(i);
        ring.push(pointv(vec * gizmo.inner_radius));
    }

    let green = materials.add(Color::LIME_GREEN.into());
    let red = materials.add(Color::RED.into());
    let white = materials.add(Color::WHITE.into());

    cmds.spawn((Transform::default(), GlobalTransform::default()))
        .with_children(|cmds| {
            cmds.spawn(polyline(green, interests))
                .spawn(polyline(red, dangers))
                .spawn(polyline(white, ring));
        })
        .current_entity()
        .unwrap()
}

#[derive(Default, Debug)]
pub struct ContextMapAI {
    pub interests: ContextMap,
    pub dangers: ContextMap,
}

impl ContextMapAI {
    pub fn new_random() -> Self {
        Self {
            interests: ContextMap::new(ContextMapV::new_random()),
            dangers: ContextMap::new(ContextMapV::new_random()),
        }
    }
}

#[derive(Default, Debug)]
pub struct Gizmo {
    pub color: Color,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub multiply: f32,
    pub gizmo_entity: Option<Entity>,
}

impl Gizmo {
    pub fn new(color: Color, inner_radius: f32, outer_radius: f32) -> Self {
        Self {
            color,
            inner_radius,
            outer_radius,
            multiply: outer_radius - inner_radius,
            gizmo_entity: None,
        }
    }
}

pub fn context_map_ai_gizmo_system(
    cmds: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // TODO use filter Changed<ContextMap>, at the time of writing it did not work
    mut query: Query<(Entity, Mut<Gizmo>, &ContextMapAI)>,
) {
    for (entity, mut gizmo, ai) in query.iter_mut() {
        for child in gizmo.gizmo_entity {
            cmds.despawn_recursive(child);
        }

        let child = spawn_context_map_ai_gizmo(&ai, &gizmo, cmds, &mut materials, &mut meshes);

        cmds.push_children(entity, &[child]);
        gizmo.gizmo_entity = Some(child);
    }
}

pub fn context_map_gizmo_system(
    cmds: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // TODO use filter Changed<ContextMap>, at the time of writing it did not work
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

pub fn example() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(example_setup.system())
        .add_system(context_map_ai_gizmo_system.system())
        .add_system(example_update.system())
        .add_system(update_ai_mouse.system())
        .run();
}

fn update_ai_mouse(
    mut this_query: Query<(&Transform, Mut<ContextMapAI>)>,
    camera_query: Query<&Transform, With<Camera>>,
    windows: Res<Windows>,
) {
    for window in windows.get_primary() {
        for cursor in window.cursor_position() {
            let camera_transform = camera_query.iter().next().unwrap();
            let size = Vec2::new(window.width(), window.height());
            let p = cursor - size * 0.5;
            let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
            let cursor = pos_wld.truncate().truncate();

            for (trans, mut ai) in this_query.iter_mut() {
                ai.interests.weights *= 0.0;
                ai.dangers.weights *= 0.0;

                let vec = (cursor - trans.translation.truncate()).normalize();
                // ai.interests.add_map(vec, |w| 1.0 - (-0.7 - w).abs());
                // ai.dangers.add_map(vec, |w| 0.5 * w);
                // ai.add(vec, |w| w);
                // ai.add(vec, |w| (w + (1.0 - w * w)));
                // ai.interests.add_map(vec, |w| 1.0 - w * w);
                ai.interests.add_map(vec, |w| (1.0 + w) / 2.0);
                ai.interests
                    .add_map(Vec2::new(-0.5, 0.0), |w| (1.0 + w) / 2.0);
                ai.interests
                    .add_map(Vec2::new(0.5, 0.0), |w| -(1.0 + w) / 2.0);
                ai.dangers.add_map(vec, |w| 0.5 * w);
            }
        }
    }
}

fn example_setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn({
        let mut bundle = Camera2dBundle::default();
        bundle.transform.scale = Vec3::new(0.5, 0.5, 1.0);
        bundle
    });

    let gizmo_parent = commands
        .spawn((
            Transform::from_translation(Vec3::new(-100.0, -100.0, 0.0)),
            GlobalTransform::default(),
        ))
        .current_entity()
        .unwrap();
    let gizmo = spawn_context_map_ai_gizmo(
        &ContextMapAI::new_random(),
        &Gizmo::new(Color::WHITE, 0.0, 1.0),
        commands,
        &mut materials,
        &mut meshes,
    );

    commands.push_children(gizmo_parent, &[gizmo]);

    commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        ContextMapAI::new_random(),
        Gizmo {
            color: Color::ORANGE_RED,
            multiply: 30.0,
            inner_radius: 0.0,
            outer_radius: 30.0,
            gizmo_entity: None,
        },
    ));
}

fn example_update(mut query: Query<Mut<ContextMap>>) {
    for mut context_map in query.iter_mut() {
        context_map.weights = ContextMapV::new_random() - ContextMapV::new_random();
    }
}
