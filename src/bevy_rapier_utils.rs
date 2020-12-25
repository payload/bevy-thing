use bevy::prelude::*;

pub use bevy_rapier2d::{
    na::{self, Vector2},
    physics::{ColliderHandleComponent, EventQueue, RapierConfiguration, RapierPhysicsPlugin, RigidBodyHandleComponent},
    rapier::{
        math::Isometry,
        dynamics::{RigidBody, RigidBodyBuilder, RigidBodySet, BodyStatus},
        geometry::{Collider, ColliderBuilder, ShapeType, ColliderHandle, ColliderSet, Proximity},
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

pub trait ToUserData {
    fn to_user_data(&self) -> u128;
}

impl ToUserData for Entity {
    fn to_user_data(&self) -> u128 {
        self.to_bits() as u128
    }
}
