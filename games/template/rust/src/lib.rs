use bevy::prelude::*;
use godot_bevy::prelude::{
    GodotDefaultPlugins,
    godot_prelude::{ExtensionLibrary, gdextension},
    *,
};

use jam_audio::AudioPlugin;
use jam_core::CorePlugin;
use jam_menu::{MainMenuPlugin, SettingsMenuPlugin};
use jam_settings::GameSettingsPlugin;

// The game is a thin shell: `#[bevy_app]` (the gdextension entry point) plus the
// list of shared jam plugins. Trim or extend this list per game; add game-specific
// plugins and SubStates of `GameState::InGame` here too.
#[bevy_app]
fn build_app(app: &mut App) {
    app.add_plugins(GodotDefaultPlugins)
        .add_plugins(CorePlugin)
        .add_plugins(GameSettingsPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(SettingsMenuPlugin);
}
