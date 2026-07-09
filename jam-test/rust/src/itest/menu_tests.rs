//! Full-stack integration: all four jam plugins boot together (via
//! [`super::setup_game`]) and the main menu appears — at the state level and, with the
//! jamkit HUD instanced, at the Godot-node level.

use bevy::prelude::*;
use godot::classes::{Button, Control, Node, PackedScene};
use godot::obj::Gd;
use godot::tools::load;
use godot_bevy_test::prelude::*;
use jam_core::{GameState, MenuState};

use super::setup_game;

/// Instance the jamkit HUD and add it to the test scene tree. Returned handle must be
/// kept alive (and freed) by the caller. Shared by the node-level menu tests.
fn spawn_hud(ctx: &TestContext) -> Gd<Node> {
    let hud: Gd<Node> = load::<PackedScene>("res://addons/jamkit/HUD.tscn")
        .instantiate()
        .expect("HUD.tscn should instantiate");
    ctx.scene_tree.clone().add_child(&hud);
    hud
}

/// State-level: booting the whole stack advances `Loading → Menu` and lands on
/// `MenuState::Main`. Proves all four plugins coexist and boot (including `AudioPlugin`'s
/// asset collection loading), which the isolated per-crate tests don't cover.
#[itest(async)]
fn test_menu_state_appears(ctx: &TestContext) -> godot::task::TaskHandle {
    let ctx = ctx.clone();
    godot::task::spawn(async move {
        let mut app = TestApp::new(&ctx, setup_game).await;

        app.updates(10).await;

        app.with_world(|world| {
            let game_state = world.resource::<State<GameState>>().get();
            assert_eq!(
                *game_state,
                GameState::Menu,
                "expected GameState::Menu after startup, got {game_state:?}"
            );

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

/// Node-level: with the jamkit HUD instanced into the scene tree, booting the stack makes
/// the main-menu panel visible. `MainPanel` starts hidden in `HUD.tscn`, so a visible
/// panel proves the show-panel command flowed through the real Godot node.
#[itest(async)]
fn test_menu_panel_visible(ctx: &TestContext) -> godot::task::TaskHandle {
    let ctx = ctx.clone();
    godot::task::spawn(async move {
        // Instance the HUD before booting so it is registered in the `jamkit_menu` group
        // before the OnExit(Loading) init system runs.
        let hud = spawn_hud(&ctx);

        let mut app = TestApp::new(&ctx, setup_game).await;

        app.updates(10).await;

        let panel = hud.get_node_as::<Control>("MainPanel");
        assert!(
            panel.is_visible(),
            "expected MainPanel to be visible once the main menu is shown"
        );

        hud.clone().queue_free();
        app.cleanup().await;
    })
}

/// Simulated click: emitting the Settings button's Godot `pressed` signal drives the same
/// GodotSignals -> observer path as a real click, so `MenuState` moves `Main -> Settings`.
/// (Start would go to `InGame` and Exit calls `quit()`, so Settings is the safe button.)
#[itest(async)]
fn test_settings_button_opens_settings(ctx: &TestContext) -> godot::task::TaskHandle {
    let ctx = ctx.clone();
    godot::task::spawn(async move {
        let hud = spawn_hud(&ctx);

        let mut app = TestApp::new(&ctx, setup_game).await;

        // Reach the menu so `connect_buttons` (OnExit(Loading)) has wired the signal.
        app.updates(10).await;

        // Simulate the click by emitting the button's `pressed` signal.
        let mut settings_btn = hud.get_node_as::<Button>("MainPanel/BtnGroup/SettingsBtn");
        settings_btn.emit_signal("pressed", &[]);

        // Let the signal cross the bridge, fire the observer, and apply the state change.
        app.updates(5).await;

        app.with_world(|world| {
            let menu_state = world
                .get_resource::<State<MenuState>>()
                .map(|s| s.get().clone());
            assert_eq!(
                menu_state,
                Some(MenuState::Settings),
                "expected MenuState::Settings after clicking Settings, got {menu_state:?}"
            );
        });

        hud.clone().queue_free();
        app.cleanup().await;
    })
}
