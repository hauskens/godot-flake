use bevy::{prelude::*, state::app::StatesPlugin};
use godot_bevy::prelude::{
    GodotDefaultPlugins,
    godot_prelude::{ExtensionLibrary, gdextension},
    *,
};
use bevy_asset_loader::prelude::*;

use crate::game_settings::GameSettingsPlugin;

mod commands;
mod main_menu;
mod settings_menu;
mod game_settings;

#[bevy_app]
fn build_app(app: &mut App) {
    app.add_plugins(GodotDefaultPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_plugins(GameSettingsPlugin)
        // `MenuState` is a sub-state that only exists while `GameState::Menu` is
        // active. Entering the game (or otherwise leaving the menu) drops it
        // automatically, and it resets to `MenuState::Main` the next time we
        // return to the menu.
        .add_sub_state::<MenuState>()
        // Gate on the Loading state, then advance into the menu. This example
        // pulls its UI nodes from the scene tree rather than loading assets, so
        // no `.load_collection::<T>()` calls are needed here.
        .add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(GameState::Menu))
        .add_plugins(commands::CommandSystemPlugin)
        .add_plugins(main_menu::MainMenuPlugin)
        .add_plugins(settings_menu::SettingsMenuPlugin);
}

// fn hello_world_system(mut timer: Local<f32>, time: Res<Time>) {
// 	// This runs every frame in Bevy's Update schedule
// 	*timer += time.delta_secs();
// 	if *timer > 1.0 {
// 		*timer = 0.0;
// 		godot_print!("Hello from Bevy ECS!");
// 	}
// }

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Menu,
    InGame,
}

/// Which menu screen is currently shown. Only exists while `GameState::Menu`.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::Menu)]
enum MenuState {
    #[default]
    Main,
    Settings,
}