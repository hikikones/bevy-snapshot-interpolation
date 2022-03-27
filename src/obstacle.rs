use bevy::prelude::*;
use physics::prelude::*;

use crate::run_criteria::game_server_run_criteria;

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_run_criteria(game_server_run_criteria)
                .with_system(move_obstacle),
        );
    }
}

#[derive(Bundle)]
pub struct ObstacleBundle {
    lerp_start: LerpStart,
    lerp_target: LerpTarget,
    lerp_timer: LerpTimer,
    marker: Obstacle,
}

impl ObstacleBundle {
    pub fn new(pos: Vec3) -> Self {
        let x = fastrand::i32(-10..=10);
        let z = fastrand::i32(-10..=10);
        Self {
            lerp_start: LerpStart(pos),
            lerp_target: LerpTarget(Vec3::new(x as f32, 0.0, z as f32)),
            lerp_timer: LerpTimer {
                duration: fastrand::i32(1..=4) as f32,
                timer: 0.0,
            },
            marker: Obstacle,
        }
    }
}

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct LerpStart(Vec3);

#[derive(Component)]
struct LerpTarget(Vec3);

#[derive(Component)]
struct LerpTimer {
    duration: f32,
    timer: f32,
}

fn move_obstacle(
    time: Res<Time>,
    mut q: Query<
        (
            &mut RigidBodyPositionComponent,
            &mut LerpStart,
            &mut LerpTarget,
            &mut LerpTimer,
        ),
        With<Obstacle>,
    >,
) {
    for (mut rb_pos, mut lerp_start, mut lerp_target, mut lerp_timer) in q.iter_mut() {
        lerp_timer.timer += time.delta_seconds();

        if lerp_timer.timer > lerp_timer.duration {
            lerp_timer.timer -= lerp_timer.duration;
            std::mem::swap(&mut lerp_start.0, &mut lerp_target.0);
        }

        let t = lerp_timer.timer / lerp_timer.duration;
        let smoothstep = 3.0 * t * t - 2.0 * t * t * t;

        let pos = Vec3::lerp(lerp_start.0, lerp_target.0, smoothstep);
        rb_pos.next_position = pos.into();
    }
}
