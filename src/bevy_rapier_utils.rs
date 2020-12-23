use bevy::math::Vec3;
use bevy_rapier2d::na::Vector2;

pub trait IntoVector2 {
    fn into_vector2(self) -> Vector2<f32>;
}

impl IntoVector2 for Vec3 {
    fn into_vector2(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}
