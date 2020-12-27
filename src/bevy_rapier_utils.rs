use bevy::prelude::*;

pub use bevy_rapier2d::{
    na::{self, Vector2},
    physics::{
        ColliderHandleComponent, EventQueue, RapierConfiguration, RapierPhysicsPlugin,
        RigidBodyHandleComponent,
    },
    rapier::{
        dynamics::{BodyStatus, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
        geometry::{
            Collider, ColliderBuilder, ColliderHandle, ColliderSet, Proximity, ProximityEvent,
            ShapeType,
        },
        math::Isometry,
    },
    render::{RapierRenderColor, RapierRenderPlugin},
};

pub trait IntoVector2 {
    fn into_vector2(self) -> Vector2<f32>;
}

impl IntoVector2 for Vec3 {
    fn into_vector2(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}

impl IntoVector2 for Vec2 {
    fn into_vector2(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}

pub trait ToUserData {
    fn to_user_data(&self) -> u128;
}

impl ToUserData for Entity {
    fn to_user_data(&self) -> u128 {
        self.to_bits() as u128
    }
}
pub trait ColliderSetExt {
    fn get_entity(&self, handle: ColliderHandle) -> Option<Entity>;
    fn get_parent(&self, handle: ColliderHandle) -> Option<RigidBodyHandle>;
}

impl ColliderSetExt for ColliderSet {
    fn get_entity(&self, handle: ColliderHandle) -> Option<Entity> {
        self.get(handle)
            .map(|it| Entity::from_bits(it.user_data as u64))
    }

    fn get_parent(&self, handle: ColliderHandle) -> Option<RigidBodyHandle> {
        self.get(handle).map(|it| it.parent())
    }
}

pub trait RigidBodySetExt {
    fn get_entity(&self, handle: RigidBodyHandle) -> Option<Entity>;
}

impl RigidBodySetExt for RigidBodySet {
    fn get_entity(&self, handle: RigidBodyHandle) -> Option<Entity> {
        self.get(handle)
            .map(|it| Entity::from_bits(it.user_data as u64))
    }
}
