//! Integration tests for the shared jam crates, run inside Godot headless via
//! `TestRunner.tscn` (see `run_godot.rs`, `just itest jam-test/rust`).
//!
//! Tests are split per crate to show each plugin works in isolation, plus a full-stack
//! module that boots them together. `#[itest]` registers into a global registry, so the
//! module layout here is purely organizational.

use bevy::app::PluginGroup;
use bevy::prelude::App;
use godot_bevy::prelude::{GodotBevyLogPlugin, GodotDefaultPlugins};

mod core_tests;
mod menu_tests;
mod settings_tests;

/// Boot the full jam plugin stack for a test, exactly as a game would via
/// [`crate::register_jam_plugins`]. `GodotBevyLogPlugin` is disabled because it installs a
/// global `tracing` subscriber; adding it on each per-test re-initialization panics
/// `BevyApp::ready`.
fn setup_game(app: &mut App) {
    app.add_plugins(GodotDefaultPlugins.build().disable::<GodotBevyLogPlugin>());
    crate::register_jam_plugins(app);
}
