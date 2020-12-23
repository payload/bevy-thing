use bevy::{ecs::DynamicBundle, prelude::*};
use bevy_rapier2d::rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder};

pub fn sprite_bundle(
    texture_atlas: Handle<TextureAtlas>,
    index: u32,
    color: Color,
) -> impl DynamicBundle {
    SpriteSheetBundle {
        texture_atlas,
        sprite: TextureAtlasSprite { index, color },
        ..Default::default()
    }
}

pub fn static_tile_physics_bundle(entity: Entity, transform: Transform) -> impl DynamicBundle {
    (
        RigidBodyBuilder::new_static()
            .translation(transform.translation.x, transform.translation.y)
            .user_data(entity.to_bits() as u128),
        ColliderBuilder::cuboid(8.0, 8.0),
    )
}
