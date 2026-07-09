//! Shared game/menu states.
//!
//! These are intentionally **concrete**, not generic over `S: States`. Shared
//! jam plugins gate on `GameState`/`MenuState` directly; a game that needs
//! finer phases adds its own `SubStates` of `GameState::InGame`. Jam ergonomics
//! win over framework flexibility here.

use bevy::prelude::*;

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    InGame,
}

/// Which menu screen is currently shown. Only exists while `GameState::Menu`.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::Menu)]
pub enum MenuState {
    #[default]
    Main,
    Settings,
}
