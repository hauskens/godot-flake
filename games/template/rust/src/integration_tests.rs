//! Integration tests for the template game, run inside Godot headless via
//! `TestRunner.tscn` (see `run_godot.rs`, `cargo run --features itest`).
//!
//! Each test boots the real game plugin stack with [`crate::register_game_plugins`]
//! inside a [`TestApp`] and steps real Godot frames, so the assertions exercise the
//! same `GameState`/`MenuState` machinery and Godot nodes as production.

use bevy::app::PluginGroup;
use bevy::prelude::*;
use godot::classes::{Control, Node, PackedScene};
use godot::obj::Gd;
use godot::tools::load;
use godot_bevy::prelude::{GodotBevyLogPlugin, GodotDefaultPlugins};
use godot_bevy_test::prelude::*;
use jam_core::{GameState, MenuState};

/// Boot the real game plugin stack for a test. `GodotBevyLogPlugin` is disabled because
/// it installs a global `tracing` subscriber, which Godot has already set — adding it a
/// second time panics `BevyApp::ready`. Everything else matches production via
/// [`crate::register_jam_plugins`].
fn setup_game(app: &mut App) {
    app.add_plugins(GodotDefaultPlugins.build().disable::<GodotBevyLogPlugin>());
    crate::register_jam_plugins(app);
}

/// State-level smoke test: booting the game advances `Loading → Menu` and lands on
/// the main menu sub-state. This proves the whole plugin stack loads and the menu
/// logic activates, without depending on any scene being present.
#[itest(async)]
fn test_menu_state_appears(ctx: &TestContext) -> godot::task::TaskHandle {
    let ctx = ctx.clone();
    godot::task::spawn(async move {
        let mut app = TestApp::new(&ctx, setup_game).await;

        // Let the (empty) loading state advance Loading -> Menu.
        app.updates(10).await;

        app.with_world(|world| {
            let game_state = world.resource::<State<GameState>>().get();
            assert_eq!(
                *game_state,
                GameState::Menu,
                "expected GameState::Menu after startup, got {game_state:?}"
            );

            // MenuState only exists while GameState::Menu is active.
            let menu_state = world
                .get_resource::<State<MenuState>>()
                .map(|s| s.get().clone());
            assert_eq!(
                menu_state,
                Some(MenuState::Main),
                "expected MenuState::Main after startup, got {menu_state:?}"
            );
        });

        app.cleanup().await;
    })
}

/// Node-level test: with the jamkit HUD instanced into the scene tree, booting the
/// game makes the main-menu panel visible. `MainPanel` starts hidden in `HUD.tscn`,
/// so a visible panel proves the show-panel command flowed through the real Godot node.
#[itest(async)]
fn test_menu_panel_visible(ctx: &TestContext) -> godot::task::TaskHandle {
    let ctx = ctx.clone();
    godot::task::spawn(async move {
        // Instance the HUD before booting the app so it is registered in the
        // `jamkit_menu` group before the OnExit(Loading) init system runs.
        let hud: Gd<Node> = load::<PackedScene>("res://addons/jamkit/HUD.tscn")
            .instantiate()
            .expect("HUD.tscn should instantiate");
        ctx.scene_tree.clone().add_child(&hud);

        let mut app = TestApp::new(&ctx, setup_game).await;

        app.updates(10).await;

        let panel = hud.get_node_as::<Control>("MainPanel");
        assert!(
            panel.is_visible(),
            "expected MainPanel to be visible once the main menu is shown"
        );

        // Tear down the scene node we added.
        hud.clone().queue_free();
        app.cleanup().await;
    })
}
