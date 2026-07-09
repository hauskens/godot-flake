//! `jam-core` in isolation: only `CorePlugin` is added, so the loading state has no asset
//! collections and auto-advances. Verifies the `GameState`/`MenuState` machinery.

use bevy::prelude::*;
use godot_bevy_test::prelude::*;
use jam_core::{CorePlugin, GameState, MenuState};

/// Booting with just `CorePlugin` advances `Loading → Menu` and lands on `MenuState::Main`.
#[itest(async)]
fn test_loading_advances_to_menu(ctx: &TestContext) -> godot::task::TaskHandle {
    let ctx = ctx.clone();
    godot::task::spawn(async move {
        let mut app = TestApp::new(&ctx, |app| {
            app.add_plugins(CorePlugin);
        })
        .await;

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
