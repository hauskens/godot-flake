use crate::jamkit_menu_root;
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
    state::state::{NextState, OnEnter, OnExit},
};
use godot::classes::Slider;
use godot::obj::Singleton;
use godot_bevy::interop::signal_names::{BaseButtonSignals, OptionButtonSignals};
use godot_bevy::prelude::*;

use jam_audio::PlaySfxMessage;
use jam_core::{GameState, MenuState, SceneResolution, UICommand, UIElement, UIHandles, Volume, WindowCommand};
use jam_settings::{GameSettings, LoadSettingsRequested, SaveSettingsRequested};

#[derive(Resource, Default)]
pub struct SettingsAssets {
    pub panel: Option<GodotNodeHandle>,
    pub resolution_option: Option<GodotNodeHandle>,
    pub master_volume_slider: Option<GodotNodeHandle>,
    pub music_volume_slider: Option<GodotNodeHandle>,
    pub sfx_volume_slider: Option<GodotNodeHandle>,
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
            .add_plugins(GodotSignalsPlugin::<MusicVolumeChanged>::default())
            .add_plugins(GodotSignalsPlugin::<SfxVolumeChanged>::default())
            // Same one-shot init/connect pattern as the main menu: the settings
            // panel lives in the same scene and just starts hidden.
            .add_systems(
                OnExit(GameState::Loading),
                (
                    init_settings_assets,
                    populate_resolutions.after(init_settings_assets),
                    set_volume_sliders.after(init_settings_assets),
                    connect_settings.after(init_settings_assets),
                ),
            )
            .add_observer(on_back_to_main_requested)
            .add_observer(on_resolution_selected)
            .add_observer(on_master_volume_changed)
            .add_observer(on_music_volume_changed)
            .add_observer(on_sfx_volume_changed)
            .add_systems(
                OnEnter(MenuState::Settings),
                (show_settings_panel, set_volume_sliders),
            )
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

    #[node("SettingsPanel/TabContainer/Audio/Volumes/MusicVolume/HSlider")]
    pub music_volume_slider: GodotNodeHandle,

    #[node("SettingsPanel/TabContainer/Audio/Volumes/SFXVolume/HSlider")]
    pub sfx_volume_slider: GodotNodeHandle,

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
    match jamkit_menu_root(&mut scene_tree) {
        Some(menu_root) => match SettingsUi::from_node(menu_root) {
            Ok(ui) => {
                ui_handles.settings_panel = Some(ui.panel);

                assets.panel = Some(ui.panel);
                assets.resolution_option = Some(ui.resolution_option);
                assets.master_volume_slider = Some(ui.master_volume_slider);
                assets.music_volume_slider = Some(ui.music_volume_slider);
                assets.sfx_volume_slider = Some(ui.sfx_volume_slider);
                assets.back_button = Some(ui.back_button);
                assets.save_button = Some(ui.save_button);
                assets.load_button = Some(ui.load_button);
            }
            Err(e) => {
                error!(
                    "Error initializing settings assets, check for missing nodes in menu scene: {}",
                    e
                );
            }
        },
        None => {
            error!("No jamkit menu root found (group '{}')", crate::JAMKIT_MENU_GROUP);
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
    let selected = resolutions
        .iter()
        .position(|&r| r == current)
        .unwrap_or_else(|| {
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

fn set_volume_sliders(
    assets: Res<SettingsAssets>,
    mut godot: GodotAccess,
    settings: Res<GameSettings>,
) {
    if let Some(handle) = assets.master_volume_slider
        && let Some(mut slider) = godot.try_get::<Slider>(handle) {
            slider.set_value(*settings.get_volume_settings().get_master_volume());
        }
    if let Some(handle) = assets.music_volume_slider
        && let Some(mut slider) = godot.try_get::<Slider>(handle) {
            slider.set_value(*settings.get_volume_settings().get_music_volume());
        }
    if let Some(handle) = assets.sfx_volume_slider
        && let Some(mut slider) = godot.try_get::<Slider>(handle) {
            slider.set_value(*settings.get_volume_settings().get_sfx_volume());
        }
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
struct MusicVolumeChanged {
    volume: Volume,
}
#[derive(Event, Debug, Clone)]
struct SfxVolumeChanged {
    volume: Volume,
}

// Each settings control needs its own strongly-typed signal handle, so the
// argument count is inherent rather than a smell worth splitting up.
#[allow(clippy::too_many_arguments)]
fn connect_settings(
    assets: Res<SettingsAssets>,
    signal_back: GodotSignals<BackToMainRequested>,
    signal_resolution: GodotSignals<ResolutionSelected>,
    signal_master_volume: GodotSignals<MasterVolumeChanged>,
    signal_music_volume: GodotSignals<MusicVolumeChanged>,
    signal_sfx_volume: GodotSignals<SfxVolumeChanged>,
    signal_load: GodotSignals<LoadSettingsRequested>,
    signal_save: GodotSignals<SaveSettingsRequested>,
    signal_play_test_sound: GodotSignals<PlaySfxMessage>,
) {
    if let Some(handle) = assets.back_button {
        signal_load.connect(
            handle,
            BaseButtonSignals::PRESSED,
            None,
            |_args, _node_handle, _ent| Some(LoadSettingsRequested),
        );
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
        signal_play_test_sound.connect(
            handle,
            SliderSignals::DRAG_ENDED,
            None,
            |_args, _node_handle, _ent| Some(PlaySfxMessage::TestSound),
        );
        signal_master_volume.connect(
            handle,
            RangeSignals::VALUE_CHANGED,
            None,
            |args, _node_handle, _ent| {
                let raw_slider_value = args.first().and_then(|v| v.try_to::<f64>().ok());
                if let Some(volume) = raw_slider_value {
                    match volume.try_into() {
                        Ok(volume) => Some(MasterVolumeChanged { volume }),
                        Err(e) => {
                            error!("Failed to convert volume to Volume: {}", e);
                            None
                        }
                    }
                } else {
                    error!("Failed to convert volume to f64");
                    None
                }
            },
        );
    }
    if let Some(handle) = assets.music_volume_slider {
        signal_music_volume.connect(
            handle,
            RangeSignals::VALUE_CHANGED,
            None,
            |args, _node_handle, _ent| {
                let raw_slider_value = args.first().and_then(|v| v.try_to::<f64>().ok());
                if let Some(volume) = raw_slider_value {
                    match volume.try_into() {
                        Ok(volume) => Some(MusicVolumeChanged { volume }),
                        Err(e) => {
                            error!("Failed to convert volume to Volume: {}", e);
                            None
                        }
                    }
                } else {
                    error!("Failed to convert volume to f64");
                    None
                }
            },
        );
    }
    if let Some(handle) = assets.sfx_volume_slider {
        signal_play_test_sound.connect(
            handle,
            SliderSignals::DRAG_ENDED,
            None,
            |_args, _node_handle, _ent| Some(PlaySfxMessage::TestSound),
        );
        signal_sfx_volume.connect(
            handle,
            RangeSignals::VALUE_CHANGED,
            None,
            |args, _node_handle, _ent| {
                let raw_slider_value = args.first().and_then(|v| v.try_to::<f64>().ok());
                if let Some(volume) = raw_slider_value {
                    match volume.try_into() {
                        Ok(volume) => Some(SfxVolumeChanged { volume }),
                        Err(e) => {
                            error!("Failed to convert volume to Volume: {}", e);
                            None
                        }
                    }
                } else {
                    error!("Failed to convert volume to f64");
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
        window_commands.write(WindowCommand::SetResolution {
            resolution: *resolution,
        });
    }
}

fn on_master_volume_changed(
    trigger: On<MasterVolumeChanged>,
    mut game_settings: ResMut<GameSettings>,
) {
    let mut volume_settings = *game_settings.get_volume_settings();
    volume_settings.set_master_volume(trigger.volume);
    game_settings.set_volume_settings(volume_settings);
}

fn on_music_volume_changed(
    trigger: On<MusicVolumeChanged>,
    mut game_settings: ResMut<GameSettings>,
) {
    let mut volume_settings = *game_settings.get_volume_settings();
    volume_settings.set_music_volume(trigger.volume);
    game_settings.set_volume_settings(volume_settings);
}

fn on_sfx_volume_changed(trigger: On<SfxVolumeChanged>, mut game_settings: ResMut<GameSettings>) {
    let mut volume_settings = *game_settings.get_volume_settings();
    volume_settings.set_sfx_volume(trigger.volume);
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
