use crate::{
    GameState, MenuState,
    commands::{UICommand, UIElement, UIHandles, WindowCommand},
};
use bevy::{
    app::{App, Plugin},
    ecs::{
        event::Event,
        message::MessageWriter,
        observer::On,
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Res, ResMut},
    },
    state::state::{NextState, OnEnter, OnExit},
};
use godot::global::godot_error;
use godot::obj::Singleton;
use godot_bevy::interop::signal_names::{BaseButtonSignals, OptionButtonSignals};
use godot_bevy::prelude::*;

/// Predefined resolutions offered by the settings screen. This is the single
/// source of truth for the built-in choices: the `ResolutionOption` dropdown is
/// filled from here at startup (see `populate_resolutions`), so the scene itself
/// carries no items. The window's current resolution is appended at startup if
/// it isn't already one of these.
const RESOLUTIONS: [(i32, i32); 3] = [(1280, 720), (1920, 1080), (2560, 1440)];

#[derive(Resource, Default)]
pub struct SettingsAssets {
    pub panel: Option<GodotNodeHandle>,
    pub resolution_option: Option<GodotNodeHandle>,
    pub back_button: Option<GodotNodeHandle>,
    /// The resolutions actually shown in the dropdown, in display order. Built
    /// at startup from `RESOLUTIONS` plus the current window resolution; the
    /// `item_selected` index maps directly into this list.
    pub resolutions: Vec<(i32, i32)>,
}

pub struct SettingsMenuPlugin;
impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsAssets>()
            .add_plugins(GodotSignalsPlugin::<BackToMainRequested>::default())
            .add_plugins(GodotSignalsPlugin::<ResolutionSelected>::default())
            // Same one-shot init/connect pattern as the main menu: the settings
            // panel lives in the same scene and just starts hidden.
            .add_systems(
                OnExit(GameState::Loading),
                (
                    init_settings_assets,
                    populate_resolutions.after(init_settings_assets),
                    connect_settings.after(init_settings_assets),
                ),
            )
            .add_observer(on_back_to_main_requested)
            .add_observer(on_resolution_selected)
            .add_systems(OnEnter(MenuState::Settings), show_settings_panel)
            .add_systems(OnExit(MenuState::Settings), hide_settings_panel);
    }
}

#[derive(NodeTreeView)]
pub struct SettingsUi {
    #[node("SettingsPanel")]
    pub panel: GodotNodeHandle,

    #[node("SettingsPanel/TabContainer/Display/Resolution/OptionDropdown")]
    pub resolution_option: GodotNodeHandle,

    #[node("SettingsPanel/BackBtn")]
    pub back_button: GodotNodeHandle,
}

fn init_settings_assets(
    mut assets: ResMut<SettingsAssets>,
    mut ui_handles: ResMut<UIHandles>,
    mut scene_tree: SceneTreeRef,
) {
    match scene_tree.get().get_current_scene() {
        Some(scene_root) => match SettingsUi::from_node(scene_root) {
            Ok(ui) => {
                ui_handles.settings_panel = Some(ui.panel);

                assets.panel = Some(ui.panel);
                assets.resolution_option = Some(ui.resolution_option);
                assets.back_button = Some(ui.back_button);
            }
            Err(e) => {
                godot_error!(
                    "Error initializing settings assets, check for missing nodes in menu scene: {}",
                    e
                );
            }
        },
        None => {
            godot_error!("No scene root found");
        }
    }
}

/// Fills the resolution dropdown from `RESOLUTIONS`, guaranteeing the window's
/// current resolution is present and pre-selected. Runs on the main thread
/// (via `GodotAccess`) after the node handles are grabbed.
fn populate_resolutions(mut assets: ResMut<SettingsAssets>, mut godot: GodotAccess) {
    use godot::classes::{DisplayServer, OptionButton};

    let Some(handle) = assets.resolution_option else {
        return;
    };
    let Some(mut option) = godot.try_get::<OptionButton>(handle) else {
        return;
    };

    let mut resolutions = RESOLUTIONS.to_vec();
    let current = DisplayServer::singleton().window_get_size();
    let current = (current.x, current.y);
    let selected = resolutions.iter().position(|&r| r == current).unwrap_or_else(|| {
        resolutions.push(current);
        resolutions.len() - 1
    });

    option.clear();
    for (width, height) in &resolutions {
        option.add_item(format!("{width} x {height}").as_str());
    }
    // `select` only updates the shown item; it does not emit `item_selected`,
    // so this does not trigger a resolution change on startup.
    option.select(selected as i32);

    assets.resolutions = resolutions;
}

#[derive(Event, Debug, Clone)]
struct BackToMainRequested;

#[derive(Event, Debug, Clone)]
struct ResolutionSelected {
    index: i64,
}

fn connect_settings(
    assets: Res<SettingsAssets>,
    signal_back: GodotSignals<BackToMainRequested>,
    signal_resolution: GodotSignals<ResolutionSelected>,
) {
    if let Some(handle) = assets.back_button {
        signal_back.connect(
            handle,
            BaseButtonSignals::PRESSED,
            None,
            |_args, _node_handle, _ent| Some(BackToMainRequested),
        );
    }
    if let Some(handle) = assets.resolution_option {
        signal_resolution.connect(
            handle,
            OptionButtonSignals::ITEM_SELECTED,
            None,
            |args, _node_handle, _ent| {
                // `item_selected` carries the chosen index as its single arg.
                let index = args.first().and_then(|v| v.try_to::<i64>().ok())?;
                Some(ResolutionSelected { index })
            },
        );
    }
}

fn on_back_to_main_requested(
    _trigger: On<BackToMainRequested>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    menu_state.set(MenuState::Main);
}

fn on_resolution_selected(
    trigger: On<ResolutionSelected>,
    assets: Res<SettingsAssets>,
    mut window_commands: MessageWriter<WindowCommand>,
) {
    if let Some(&(width, height)) = assets.resolutions.get(trigger.index as usize) {
        window_commands.write(WindowCommand::SetResolution { width, height });
    }
}

fn show_settings_panel(mut ui_commands: MessageWriter<UICommand>) {
    ui_commands.write(UICommand::SetVisible {
        target: UIElement::SettingsPanel,
        visible: true,
    });
}

fn hide_settings_panel(mut ui_commands: MessageWriter<UICommand>) {
    ui_commands.write(UICommand::SetVisible {
        target: UIElement::SettingsPanel,
        visible: false,
    });
}
