//! Core building blocks shared by every jam game: states, value types, and the
//! thread-safe Godot command system, bundled into a single [`CorePlugin`].

pub mod commands;
pub mod states;
pub mod types;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

pub use commands::{
    CommandSystemPlugin, NodeCommand, UICommand, UIElement, UIHandles, WindowCommand,
};
pub use states::{GameState, MenuState};
pub use types::{
    AudioOutputDevice, AudioOutputDeviceList, Gain, SceneResolution, Volume, VolumeError,
    VolumeSettings,
};

/// Wires up the shared foundation:
/// - Bevy's [`StatesPlugin`] plus the `GameState`/`MenuState` states,
/// - a loading state that advances `Loading → Menu` once assets are ready,
/// - the [`CommandSystemPlugin`] for main-thread Godot access.
///
/// Games add [`bevy_asset_loader`] collections to `GameState::Loading` (via
/// `configure_loading_state`) so they finish loading before the menu appears.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<MenuState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading).continue_to_state(GameState::Menu),
            )
            .add_plugins(CommandSystemPlugin);
    }
}
