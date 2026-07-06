use crate::{
    GameState, MenuState,
    commands::{UICommand, UIElement, UIHandles},
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
    log::error,
    log::info,
    state::state::{NextState, OnEnter, OnExit, State},
};
use godot_bevy::interop::signal_names::BaseButtonSignals;
use godot_bevy::prelude::*;

#[derive(Resource, Default)]
pub struct MenuAssets {
    pub start_button: Option<GodotNodeHandle>,
    pub settings_button: Option<GodotNodeHandle>,
    pub exit_button: Option<GodotNodeHandle>,
}

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuAssets>()
            .add_plugins(GodotSignalsPlugin::<StartGameRequested>::default())
            .add_plugins(GodotSignalsPlugin::<OpenSettingsRequested>::default())
            .add_plugins(GodotSignalsPlugin::<ExitGameRequested>::default())
            .add_systems(
                OnExit(GameState::Loading),
                (init_menu_assets, connect_buttons.after(init_menu_assets)),
            )
            // Use observers instead of systems with MessageReader
            .add_observer(on_start_game_requested)
            .add_observer(on_open_settings_requested)
            .add_observer(on_exit_game_requested)
            // Show/hide the whole main-menu panel as the sub-state toggles.
            .add_systems(OnEnter(MenuState::Main), show_main_panel)
            .add_systems(OnExit(MenuState::Main), hide_main_panel);
    }
}

#[derive(NodeTreeView)]
pub struct MenuUi {
    #[node("MainPanel")]
    pub panel: GodotNodeHandle,

    #[node("MainPanel/BtnGroup/StartBtn")]
    pub start_button: GodotNodeHandle,

    #[node("MainPanel/BtnGroup/SettingsBtn")]
    pub settings_button: GodotNodeHandle,

    #[node("MainPanel/BtnGroup/ExitBtn")]
    pub exit_button: GodotNodeHandle,
}

fn init_menu_assets(
    mut menu_assets: ResMut<MenuAssets>,
    mut ui_handles: ResMut<UIHandles>,
    mut scene_tree: SceneTreeRef,
) {
    match scene_tree.get().get_current_scene() {
        Some(scene_root) => match MenuUi::from_node(scene_root) {
            Ok(menu_ui) => {
                // The command system toggles the panel and writes label text by
                // element id, so it needs these handles too.
                ui_handles.main_panel = Some(menu_ui.panel);

                menu_assets.start_button = Some(menu_ui.start_button);
                menu_assets.settings_button = Some(menu_ui.settings_button);
                menu_assets.exit_button = Some(menu_ui.exit_button);
            }
            Err(e) => {
                error!(
                    "Error initializing menu assets, check for missing nodes in menu scene: {}",
                    e
                );
            }
        },
        None => {
            error!("No scene root found");
        }
    }
}

#[derive(Event, Debug, Clone)]
struct StartGameRequested;

#[derive(Event, Debug, Clone)]
struct OpenSettingsRequested;

#[derive(Event, Debug, Clone)]
struct ExitGameRequested;

fn connect_buttons(
    menu_assets: Res<MenuAssets>,
    signal_start: GodotSignals<StartGameRequested>,
    signal_settings: GodotSignals<OpenSettingsRequested>,
    signal_exit: GodotSignals<ExitGameRequested>,
) {
    if let Some(handle) = menu_assets.start_button {
        signal_start.connect(
            handle,
            BaseButtonSignals::PRESSED,
            None,
            |_args, _node_handle, _ent| Some(StartGameRequested),
        );
    }
    if let Some(handle) = menu_assets.settings_button {
        signal_settings.connect(
            handle,
            BaseButtonSignals::PRESSED,
            None,
            |_args, _node_handle, _ent| Some(OpenSettingsRequested),
        );
    }
    if let Some(handle) = menu_assets.exit_button {
        signal_exit.connect(
            handle,
            BaseButtonSignals::PRESSED,
            None,
            |_args, _node_handle, _ent| Some(ExitGameRequested),
        );
    }
}

fn on_start_game_requested(
    _trigger: On<StartGameRequested>,
    menu_state: Option<Res<State<MenuState>>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if menu_state.is_some_and(|s| *s.get() == MenuState::Main) {
        info!("Starting game");
        game_state.set(GameState::InGame);
    }
}

fn on_open_settings_requested(
    _trigger: On<OpenSettingsRequested>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    menu_state.set(MenuState::Settings);
}

fn on_exit_game_requested(_trigger: On<ExitGameRequested>, mut scene_tree: SceneTreeRef) {
    info!("Quitting game");
    scene_tree.get().quit();
}

fn show_main_panel(mut ui_commands: MessageWriter<UICommand>) {
    ui_commands.write(UICommand::SetVisible {
        target: UIElement::MainPanel,
        visible: true,
    });
    ui_commands.write(UICommand::SetVisible {
        target: UIElement::SettingsPanel,
        visible: false,
    });
}

fn hide_main_panel(mut ui_commands: MessageWriter<UICommand>) {
    ui_commands.write(UICommand::SetVisible {
        target: UIElement::MainPanel,
        visible: false,
    });
}
