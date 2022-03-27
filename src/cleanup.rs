use bevy::prelude::*;

use crate::AppState;

pub struct CleanupPlugin;

impl Plugin for CleanupPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Menu).with_system(on_enter_game));
    }
}

#[derive(Component)]
pub struct Cleanup;

fn on_enter_game(cleanup_q: Query<Entity, With<Cleanup>>, mut commands: Commands) {
    for entity in cleanup_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
