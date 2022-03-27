use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub trait MoveTowards {
    type Target;

    fn move_towards(&mut self, target: Self::Target, max_delta: f32) -> bool;
}

impl MoveTowards for RigidBodyPositionComponent {
    type Target = Vec3;

    fn move_towards(&mut self, target: Self::Target, max_delta: f32) -> bool {
        let mut pos: Vec3 = self.position.translation.vector.into();
        let reached_target =
            bevy_extensions::MoveTowards::move_towards(&mut pos, target, max_delta);
        self.next_position = pos.into();

        reached_target
    }
}
