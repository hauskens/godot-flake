use crate::{
    GameState, MenuState,
    commands::{UICommand, UIElement, UIHandles, WindowCommand}, game_settings::{GameSettings, Volume},
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
use godot::{global::{godot_error, godot_print}};
use godot::obj::Singleton;
use godot_bevy::interop::signal_names::{BaseButtonSignals, OptionButtonSignals};
use godot_bevy::prelude::*;
use derive_more::Display;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display("{width} x {height}")]
pub struct SceneResolution {
    width: i32,
    height: i32,
}

impl SceneResolution {
    pub const RESOLUTIONS: [SceneResolution; 3] = [SceneResolution::new(1280, 720), SceneResolution::new(1920, 1080), SceneResolution::new(2560, 1440)];
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
    pub fn get_width(&self) -> i32 {
        self.width
    }
    pub fn get_height(&self) -> i32 {
        self.height
    }
    pub fn _to_tuple(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}



#[derive(Resource, Default)]
pub struct SettingsAssets {
    pub panel: Option<GodotNodeHandle>,
    pub resolution_option: Option<GodotNodeHandle>,
    pub master_volume_slider: Option<GodotNodeHandle>,
    pub back_button: Option<GodotNodeHandle>,
    pub save_button: Option<GodotNodeHandle>,
    pub load_button: Option<GodotNodeHandle>,
    /// The resolutions actually shown in the dropdown, in display order. Built
    /// at startup from `RESOLUTIONS` plus the current window resolution; the
    /// `item_selected` index maps directly into this list.
    pub resolutions: Vec<SceneResolution>,
}

pub struct SettingsMenuPlugin;
impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsAssets>()
            .add_plugins(GodotSignalsPlugin::<BackToMainRequested>::default())
            .add_plugins(GodotSignalsPlugin::<ResolutionSelected>::default())
            .add_plugins(GodotSignalsPlugin::<MasterVolumeChanged>::default())
            .add_plugins(GodotSignalsPlugin::<SaveSettingsRequested>::default())
            .add_plugins(GodotSignalsPlugin::<LoadSettingsRequested>::default())
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
            .add_observer(on_save_settings_requested)
            .add_observer(on_load_settings_requested)
            .add_observer(on_master_volume_changed)
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

    #[node("SettingsPanel/TabContainer/Audio/Volumes/MasterVolume/HSlider")]
    pub master_volume_slider: GodotNodeHandle,

    #[node("SettingsPanel/Nav/BackBtn")]
    pub back_button: GodotNodeHandle,

    #[node("SettingsPanel/Nav/SaveBtn")]
    pub save_button: GodotNodeHandle,

    #[node("SettingsPanel/Nav/LoadBtn")]
    pub load_button: GodotNodeHandle,
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
                assets.master_volume_slider = Some(ui.master_volume_slider);
                assets.back_button = Some(ui.back_button);
                assets.save_button = Some(ui.save_button);
                assets.load_button = Some(ui.load_button);
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

    let mut resolutions = SceneResolution::RESOLUTIONS.to_vec();
    let current = DisplayServer::singleton().window_get_size();
    let current = SceneResolution::new(current.x, current.y);
    let selected = resolutions.iter().position(|&r| r == current).unwrap_or_else(|| {
        resolutions.push(current);
        resolutions.len() - 1
    });

    option.clear();
    for resolution in &resolutions {
        option.add_item(resolution.to_string().as_str());
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
#[derive(Event, Debug, Clone)]
struct MasterVolumeChanged {
    volume: Volume,
}

#[derive(Event, Debug, Clone)]
struct SaveSettingsRequested;

#[derive(Event, Debug, Clone)]
struct LoadSettingsRequested;

fn connect_settings(
    assets: Res<SettingsAssets>,
    signal_back: GodotSignals<BackToMainRequested>,
    signal_resolution: GodotSignals<ResolutionSelected>,
    signal_master_volume: GodotSignals<MasterVolumeChanged>,
    signal_load: GodotSignals<LoadSettingsRequested>,
    signal_save: GodotSignals<SaveSettingsRequested>,
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
    if let Some(handle) = assets.master_volume_slider {
        signal_master_volume.connect(
            handle,
            RangeSignals::VALUE_CHANGED,
            None,
            |args, _node_handle, _ent| {
                let raw_slider_value = args
                    .first()
                    .and_then(|v| v.try_to::<f64>().ok());
                if let Some(volume) = raw_slider_value {
                    match volume.try_into() {
                        Ok(volume) => Some(MasterVolumeChanged { volume }),
                        Err(e) => {
                            godot_error!("Failed to convert volume to Volume: {}", e);
                            None
                        }
                    }
                } else {
                    godot_error!("Failed to convert volume to f64");
                    None
                }
            },
        );
    }
    if let Some(handle) = assets.load_button {
        signal_load.connect(
            handle,
            BaseButtonSignals::PRESSED,
            None,
            |_args, _node_handle, _ent| Some(LoadSettingsRequested),
        );
    }
    if let Some(handle) = assets.save_button {
        signal_save.connect(
            handle,
            BaseButtonSignals::PRESSED,
            None,
            |_args, _node_handle, _ent| Some(SaveSettingsRequested),
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
    if let Some(resolution) = assets.resolutions.get(trigger.index as usize) {
        window_commands.write(WindowCommand::SetResolution { resolution: *resolution });
    }
}

fn on_save_settings_requested(
    _trigger: On<SaveSettingsRequested>,
    game_settings: Res<GameSettings>,
) {
    godot_print!("Saving settings");
    game_settings.save_settings();
}

fn on_load_settings_requested(
    _trigger: On<LoadSettingsRequested>,
    mut game_settings: ResMut<GameSettings>,
) {
    godot_print!("Loading settings");
    match GameSettings::load_settings() {
        Ok(loaded) => *game_settings = loaded,
        Err(e) => godot_error!("Failed to load settings: {}", e),
    }
}

fn on_master_volume_changed(
    trigger: On<MasterVolumeChanged>,
    mut game_settings: ResMut<GameSettings>,
) {
    let mut volume_settings = game_settings.get_volume_settings().clone();
    volume_settings.set_master_volume(trigger.volume);
    game_settings.set_volume_settings(volume_settings);
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
