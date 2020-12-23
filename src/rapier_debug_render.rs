use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::{
    pbr::render_graph::FORWARD_PIPELINE_HANDLE,
    prelude::*,
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
};
use bevy_rapier2d::{
    na::{UnitQuaternion, Vector3},
    physics::{ColliderHandleComponent, RapierConfiguration},
    rapier::{
        dynamics::{BodyStatus, RigidBody, RigidBodySet},
        geometry::{ColliderSet, ShapeType},
        math::Isometry,
    },
    render::RapierRenderColor,
};

fn get_color(body: &RigidBody, debug_color: Option<&RapierRenderColor>) -> Color {
    let light = 0.6 + 0.2 * rand::random::<f32>();
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
                    ShapeType::Cuboid => Mesh::from(shape::Quad {
                        size: {
                            let hsize = shape.as_cuboid().unwrap().half_extents;
                            Vec2::new(hsize.x, hsize.y) * configuration.scale * 2.0
                        },
                        flip: false,
                    }),
                    ShapeType::Ball => Mesh::from(shape::Icosphere {
                        subdivisions: 2,
                        radius: shape.as_ball().unwrap().radius * configuration.scale,
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

                let pbr = (
                    RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        FORWARD_PIPELINE_HANDLE.typed(),
                    )]),
                    meshes.add(mesh),
                    Visible {
                        is_transparent: true,
                        is_visible: true,
                    },
                    materials.add(StandardMaterial {
                        albedo: get_color(&body, debug_color),
                        albedo_texture: None,
                        shaded: false,
                    }),
                    MainPass::default(),
                    Draw::default(),
                );

                commands.insert(entity, pbr);
            }
        }
    }
}

fn sync_transform(pos: &Isometry<f32>, scale: f32, transform: &mut Transform) {
    // Do not touch the 'z' part of the translation, used in Bevy for 2d layering
    transform.translation.x = pos.translation.vector.x * scale;
    transform.translation.y = pos.translation.vector.y * scale;

    let rot = UnitQuaternion::new(Vector3::z() * pos.rotation.angle());
    transform.rotation = Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w);
}
