//! Command system for thread-safe Godot API access
//!
//! This module provides a command/event pattern that allows most game logic
//! to run multi-threaded while keeping Godot API access on the main thread.

use bevy::prelude::*;
use godot::builtin::{Vector2, Vector2i};
use godot_bevy::prelude::*;

use crate::types::SceneResolution;

/// Commands for UI operations
#[derive(Message, Debug, Clone)]
pub enum UICommand {
    /// Set visibility of a UI element
    SetVisible { target: UIElement, visible: bool },
}

/// Commands for the game window / display.
#[derive(Message, Debug, Clone)]
pub enum WindowCommand {
    /// Resize the game window.
    SetResolution { resolution: SceneResolution },
}

/// Commands for node operations
#[derive(Message, Debug, Clone)]
pub enum NodeCommand {
    /// Set visibility of any node
    #[allow(dead_code)]
    SetVisible { entity: Entity, visible: bool },
    /// Set position of a node
    #[allow(dead_code)]
    SetPosition { entity: Entity, position: Vector2 },
}

/// UI element identifiers
#[derive(Debug, Clone, PartialEq)]
pub enum UIElement {
    /// Root panel of the main menu screen.
    MainPanel,
    /// Root panel of the settings screen.
    SettingsPanel,
}

/// Resource to hold UI element handles
#[derive(Resource, Default)]
pub struct UIHandles {
    pub main_panel: Option<GodotNodeHandle>,
    pub settings_panel: Option<GodotNodeHandle>,
}

impl UIHandles {
    pub fn get_handle(&self, element: &UIElement) -> Option<GodotNodeHandle> {
        match element {
            UIElement::MainPanel => self.main_panel,
            UIElement::SettingsPanel => self.settings_panel,
        }
    }
}

/// Plugin that sets up the command system
pub struct CommandSystemPlugin;

impl Plugin for CommandSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UIHandles>()
            .add_message::<UICommand>()
            .add_message::<WindowCommand>()
            .add_message::<NodeCommand>()
            .add_systems(
                Update,
                (
                    // Main thread systems that process commands
                    process_ui_commands,
                    process_window_commands,
                    process_node_commands,
                ),
            );
    }
}

/// Main thread system that processes UI commands
fn process_ui_commands(
    mut ui_commands: MessageReader<UICommand>,
    ui_handles: Res<UIHandles>,
    mut godot: GodotAccess,
) {
    use godot::classes::CanvasItem;

    for command in ui_commands.read() {
        match command {
            UICommand::SetVisible { target, visible } => {
                // `CanvasItem` is the common base of panels, buttons and labels,
                // so this toggles whole screens as well as individual widgets.
                if let Some(handle) = ui_handles.get_handle(target)
                    && let Some(mut item) = godot.try_get::<CanvasItem>(handle)
                {
                    item.set_visible(*visible);
                }
            }
        }
    }
}

/// Main thread system that applies window/display commands.
///
/// Takes `SceneTreeRef` both to reach the root `Window` and because it forces
/// this system onto the main thread, where Godot API calls are safe.
fn process_window_commands(
    mut window_commands: MessageReader<WindowCommand>,
    mut scene_tree: SceneTreeRef,
) {
    for command in window_commands.read() {
        match command {
            WindowCommand::SetResolution { resolution } => {
                if let Some(mut window) = scene_tree.get().get_root() {
                    window.set_size(Vector2i::new(
                        resolution.get_width(),
                        resolution.get_height(),
                    ));
                }
            }
        }
    }
}

/// Main thread system that processes node commands
fn process_node_commands(
    mut node_commands: MessageReader<NodeCommand>,
    nodes: Query<&GodotNodeHandle>,
    mut godot: GodotAccess,
) {
    use godot::classes::CanvasItem;

    for command in node_commands.read() {
        match command {
            NodeCommand::SetVisible { entity, visible } => {
                if let Ok(handle) = nodes.get(*entity)
                    && let Some(mut canvas_item) = godot.try_get::<CanvasItem>(*handle)
                {
                    canvas_item.set_visible(*visible);
                }
            }
            NodeCommand::SetPosition { entity, position } => {
                if let Ok(handle) = nodes.get(*entity)
                    && let Some(mut node) = godot.try_get::<godot::classes::Node2D>(*handle)
                {
                    node.set_position(*position);
                }
            }
        }
    }
}
