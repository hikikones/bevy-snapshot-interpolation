use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .insert_resource(RapierConfiguration {
                // gravity: Vec3::ZERO.into(),
                ..Default::default()
            });
    }
}

#[path = "."]
pub mod prelude {
    mod move_towards;
    pub use bevy_rapier3d::prelude::*;
    pub use move_towards::*;
}
