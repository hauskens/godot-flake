//! Shared main-menu and settings-menu screens driven by `GameState`/`MenuState`.
//!
//! The menu UI lives in the `jamkit` addon's `HUD.tscn`, which games instance
//! under an arbitrary parent. Because the HUD is no longer the current scene,
//! we locate its root by the Godot group [`JAMKIT_MENU_GROUP`] rather than via
//! `get_current_scene()`; all `#[node(...)]` paths then resolve relative to it.

pub mod main_menu;
pub mod settings_menu;

pub use main_menu::MainMenuPlugin;
pub use settings_menu::SettingsMenuPlugin;

use godot::classes::Node;
use godot::obj::Gd;
use godot_bevy::prelude::SceneTreeRef;

/// Godot group the `jamkit` HUD root node belongs to (set in `HUD.tscn`).
pub const JAMKIT_MENU_GROUP: &str = "jamkit_menu";

/// Returns the `jamkit` HUD root node, found by group membership.
///
/// Groups register at `_enter_tree`, so this is available by the time the
/// `OnExit(GameState::Loading)` init systems run.
pub(crate) fn jamkit_menu_root(scene_tree: &mut SceneTreeRef) -> Option<Gd<Node>> {
    scene_tree
        .get()
        .get_nodes_in_group(JAMKIT_MENU_GROUP)
        .iter_shared()
        .next()
}
