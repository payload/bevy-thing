use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};

use crate::bevy_rapier_utils::*;

fn get_color(
    body: &RigidBody,
    collider: &Collider,
    debug_color: Option<&RapierRenderColor>,
) -> Color {
    let base = if collider.is_sensor() { 0.6 } else { 0.4 };
    let light = base + 0.2 * rand::random::<f32>();
    let default_color = match body.body_status {
        BodyStatus::Static => Color::rgb(0.2 + light, light, light),
        BodyStatus::Dynamic => Color::rgb(light, 0.2 + light, light),
        BodyStatus::Kinematic => Color::rgb(light, light, 0.2 + light),
    };

    let mut color = debug_color
        .map(|c| Color::rgb(c.0, c.1, c.2))
        .unwrap_or(default_color);
    color.set_a(0.5);
    color
}

fn _sync_transform(pos: &Isometry<f32>, scale: f32, transform: &mut Transform) {
    // Do not touch the 'z' part of the translation, used in Bevy for 2d layering
    transform.translation.x = pos.translation.vector.x * scale;
    transform.translation.y = pos.translation.vector.y * scale;

    let rot = na::UnitQuaternion::new(na::Vector3::z() * pos.rotation.angle());
    transform.rotation = Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w);
}

/// System responsible for attaching a PbrBundle to each entity having a collider.
pub fn rapier_debug_render(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    configuration: Res<RapierConfiguration>,
    bodies: Res<RigidBodySet>,
    colliders: ResMut<ColliderSet>,
    query: Query<
        (Entity, &ColliderHandleComponent, Option<&RapierRenderColor>),
        Without<Handle<Mesh>>,
    >,
) {
    for (entity, collider, debug_color) in &mut query.iter() {
        if let Some(collider) = colliders.get(collider.handle()) {
            if let Some(body) = bodies.get(collider.parent()) {
                let shape = collider.shape();
                let mesh = match shape.shape_type() {
                    #[cfg(feature = "dim3")]
                    ShapeType::Cuboid => Mesh::from(shape::Cube { size: 2.0 }),
                    ShapeType::Cuboid => Mesh::from(shape::Quad {
                        size: Vec2::new(2.0, 2.0),
                        flip: false,
                    }),
                    ShapeType::Ball => Mesh::from(shape::Icosphere {
                        subdivisions: 1,
                        radius: 1.0,
                    }),

                    ShapeType::Trimesh => {
                        let mut mesh =
                            Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
                        let trimesh = shape.as_trimesh().unwrap();
                        mesh.set_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            VertexAttributeValues::from(
                                trimesh
                                    .vertices()
                                    .iter()
                                    .map(|vertice| [vertice.x, vertice.y])
                                    .collect::<Vec<_>>(),
                            ),
                        );
                        mesh.set_indices(Some(Indices::U32(
                            trimesh
                                .indices()
                                .iter()
                                .flat_map(|triangle| triangle.iter())
                                .cloned()
                                .collect(),
                        )));
                        mesh
                    }
                    _ => unimplemented!(),
                };

                let scale = match shape.shape_type() {
                    ShapeType::Cuboid => {
                        let c = shape.as_cuboid().unwrap();
                        Vec3::new(c.half_extents.x, c.half_extents.y, 1.0)
                    }
                    #[cfg(feature = "dim3")]
                    ShapeType::Cuboid => {
                        let c = shape.as_cuboid().unwrap();
                        Vec3::from_slice_unaligned(c.half_extents.as_slice())
                    }
                    ShapeType::Ball => {
                        let b = shape.as_ball().unwrap();
                        Vec3::new(b.radius, b.radius, b.radius)
                    }
                    ShapeType::Trimesh => Vec3::one(),
                    _ => unimplemented!(),
                } * configuration.scale;

                let mut transform = Transform::from_scale(scale);
                _sync_transform(
                    collider.position_wrt_parent(),
                    configuration.scale,
                    &mut transform,
                );

                if let ShapeType::Ball = shape.shape_type() {
                    transform.translation.z -= scale.x;
                }

                let ground_pbr = PbrBundle {
                    transform,
                    mesh: meshes.add(mesh),
                    visible: Visible {
                        is_transparent: true,
                        is_visible: true,
                    },
                    material: materials.add(StandardMaterial {
                        albedo: get_color(body, collider, debug_color),
                        albedo_texture: None,
                        shaded: false,
                    }),
                    ..Default::default()
                };

                commands.insert(entity, ground_pbr);
            }
        }
    }
}
