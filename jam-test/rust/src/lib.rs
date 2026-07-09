//! `jam-test` is a game-independent host for the shared jam crates' integration tests.
//!
//! It is structurally "the template game minus the game": a cdylib whose `#[bevy_app]`
//! entry point registers no gameplay, plus (under the `itest` feature) the godot-bevy-test
//! runner and the test modules in [`itest`]. Build/run with `--features itest`, or via
//! `just itest jam-test/rust`.

use bevy::prelude::*;
use godot_bevy::prelude::{
    godot_prelude::{ExtensionLibrary, gdextension},
    *,
};

use jam_audio::AudioPlugin;
use jam_core::CorePlugin;
use jam_menu::{MainMenuPlugin, SettingsMenuPlugin};
use jam_settings::GameSettingsPlugin;

// No game here: jam-test only hosts integration tests. `#[bevy_app]` still provides the
// sole gdextension entry point (`gdext_rust_init`), but its init func is empty so the
// `BevyAppSingleton` autoload stays idle — `TestApp` drives each test (see
// `itest::setup_game`). This also avoids the autoload installing a global tracing
// subscriber that per-test re-initialization would collide with.
#[bevy_app]
fn build_app(_app: &mut App) {}

/// The jam plugin stack under test, mirroring what a real game wires up. Reused by the
/// full-stack tests in [`itest`] so they don't drift from production plugin ordering.
pub fn register_jam_plugins(app: &mut App) {
    app.add_plugins(CorePlugin)
        .add_plugins(GameSettingsPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(MainMenuPlugin)
        .add_plugins(SettingsMenuPlugin);
}

// Integration-test harness. `declare_test_runner!` registers the `IntegrationTests`
// GodotClass (instantiated by `res://addons/godot-bevy/test/TestRunner.tscn`) into this
// dylib's single gdextension; the test modules compile in alongside it.
#[cfg(feature = "itest")]
godot_bevy_test::declare_test_runner!();

#[cfg(feature = "itest")]
mod itest;
