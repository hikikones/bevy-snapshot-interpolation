use bevy::{ecs::schedule::ShouldRun, prelude::*};

use crate::{network::Server, AppState};

pub fn game_server_run_criteria(
    server: Option<Res<Server>>,
    game_state: Res<State<AppState>>,
) -> ShouldRun {
    if server.is_some() && *game_state.current() == AppState::Game {
        return ShouldRun::Yes;
    }

    ShouldRun::No
}

pub fn game_client_exclusive_run_criteria(
    server: Option<Res<Server>>,
    app_state: Res<State<AppState>>,
) -> ShouldRun {
    if server.is_some() || *app_state.current() != AppState::Game {
        return ShouldRun::No;
    }

    ShouldRun::Yes
}
