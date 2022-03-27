use bevy::prelude::*;

use crate::{cleanup::Cleanup, player::LocalPlayer, AppState};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Game).with_system(spawn_camera))
            .add_system_set(SystemSet::on_update(AppState::Game).with_system(follow_player));
    }
}

#[derive(Component)]
struct CameraPivot;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(CameraPivot)
        .insert(Cleanup)
        .with_children(|child| {
            child.spawn_bundle(PerspectiveCameraBundle {
                transform: Transform::from_xyz(0.0, 10.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            });
        });
}

fn follow_player(
    player_q: Query<&Transform, With<LocalPlayer>>,
    mut pivot_q: Query<&mut Transform, (With<CameraPivot>, Without<LocalPlayer>)>,
    time: Res<Time>,
) {
    for player in player_q.iter() {
        for mut pivot in pivot_q.iter_mut() {
            let distance = player.translation.distance(pivot.translation);
            let t = f32::powf(0.25, distance * time.delta_seconds());
            pivot.translation = Vec3::lerp(player.translation, pivot.translation, t);
        }
    }
}
