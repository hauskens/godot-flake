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
    // Under `itest` the autoload stays idle: `TestApp` installs the game plugins
    // per-test (see `integration_tests::setup_game`). This mirrors godot-bevy's itest
    // harness, where no `#[bevy_app]` init func runs so tests fully control the app —
    // and it avoids the autoload installing a global tracing subscriber that the
    // per-test re-initialization would then collide with.
    #[cfg(not(feature = "itest"))]
    register_game_plugins(app);
    #[cfg(feature = "itest")]
    let _ = app;
}

/// Shared plugin set for the game. Kept separate from the `#[bevy_app]` entry point
/// so integration tests (under the `itest` feature) boot the exact same plugin stack.
///
/// The core scene-tree plugins (`GodotBaseCorePlugin`/`GodotSceneTreePlugin`) are added
/// automatically by `#[bevy_app]` in production and by `TestApp` in tests, so they must
/// not be added here; `GodotDefaultPlugins` does not overlap with them.
pub fn register_game_plugins(app: &mut App) {
    app.add_plugins(GodotDefaultPlugins);
    register_jam_plugins(app);
}

/// The game-specific jam plugins, without the Godot engine plugin group. Integration
/// tests reuse this so they don't drift from the real game, while supplying their own
/// (log-plugin-disabled) variant of `GodotDefaultPlugins`.
pub fn register_jam_plugins(app: &mut App) {
    app.add_plugins(CorePlugin)
        .add_plugins(GameSettingsPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(SettingsMenuPlugin);
}

// Integration-test harness: `declare_test_runner!` registers the `IntegrationTests`
// GodotClass (instantiated by `res://addons/godot-bevy/test/TestRunner.tscn`) into the
// game's single gdextension. It emits no extra entry point, so it coexists with
// `#[bevy_app]`. Tests compile into this cdylib as a module. Build with `--features itest`.
#[cfg(feature = "itest")]
godot_bevy_test::declare_test_runner!();

#[cfg(feature = "itest")]
mod integration_tests;
