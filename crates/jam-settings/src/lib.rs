//! Persistent game settings backed by a `user://` `ConfigFile`.
//!
//! Owns the [`GameSettings`] resource and its load/save flow; the actual value
//! types (`Volume`, `VolumeSettings`) live in [`jam_core`] so that `jam-audio`
//! can read them without depending on this crate.

use bevy::prelude::*;
use godot::{
    classes::{AudioServer, ConfigFile},
    meta::ToGodot,
    obj::{Gd, NewGd, Singleton},
};
use godot_bevy::plugins::signals::GodotSignalsPlugin;
use thiserror::Error;

use jam_core::{AudioOutputDevice, Volume, VolumeError, VolumeSettings};

const SETTINGS_PATH: &str = "user://settings.cfg";

#[derive(Event, Debug, Clone)]
pub struct SaveSettingsRequested;

#[derive(Event, Debug, Clone)]
pub struct LoadSettingsRequested;

pub struct GameSettingsPlugin;
impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        match GameSettings::load_settings() {
            Ok(settings) => {
                app.insert_resource(settings)
                    .add_plugins(GodotSignalsPlugin::<SaveSettingsRequested>::default())
                    .add_plugins(GodotSignalsPlugin::<LoadSettingsRequested>::default())
                    .add_observer(on_save_settings_requested)
                    .add_observer(on_load_settings_requested);
            }
            Err(e) => {
                // Fail early: a corrupt/invalid settings file should abort
                // startup rather than boot with a missing GameSettings
                // resource (which would panic later, far from the cause).
                error!("Failed to load game settings: {}", e);
            }
        }
    }
}

#[derive(Default, Resource)]
pub struct GameSettings {
    volume_settings: VolumeSettings,
    audio_output_device: Option<AudioOutputDevice>,
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum GameSettingsError {
    #[error("'{key}' in config is not a number")]
    WrongType { key: String },
    #[error("invalid volume for '{key}': {source}")]
    InvalidVolume {
        key: String,
        #[source]
        source: VolumeError,
    },
}

impl GameSettings {
    pub fn get_volume_settings(&self) -> &VolumeSettings {
        &self.volume_settings
    }

    pub fn set_volume_settings(&mut self, volume_settings: VolumeSettings) {
        self.volume_settings = volume_settings;
    }

    pub fn get_audio_output_device(&self) -> Option<&AudioOutputDevice> {
        self.audio_output_device.as_ref()
    }

    pub fn set_audio_output_device(&mut self, audio_output_device: AudioOutputDevice) {
        self.audio_output_device = Some(audio_output_device);
    }

    pub fn save_settings(&self) {
        let mut config = ConfigFile::new_gd();
        config.load(SETTINGS_PATH);

        let volume = &self.volume_settings;
        config.set_value(
            VolumeSettings::SETTINGS_SECTION,
            "master_volume",
            &(*volume.get_master_volume()).to_variant(),
        );
        config.set_value(
            VolumeSettings::SETTINGS_SECTION,
            "music_volume",
            &(*volume.get_music_volume()).to_variant(),
        );
        config.set_value(
            VolumeSettings::SETTINGS_SECTION,
            "sfx_volume",
            &(*volume.get_sfx_volume()).to_variant(),
        );

        if let Some(audio_output_device) = &self.audio_output_device {
            config.set_value(
                AudioOutputDevice::SETTINGS_SECTION,
                AudioOutputDevice::SETTINGS_KEY,
                &audio_output_device.to_string().to_variant(),
            );
        }

        info!("Saving settings to {}", SETTINGS_PATH);

        config.save(SETTINGS_PATH);
    }

    /// Loads settings from disk. A missing file yields all defaults; a present
    /// but out-of-range or wrong-typed value is an error.
    pub fn load_settings() -> Result<Self, GameSettingsError> {
        let mut config = ConfigFile::new_gd();
        if config.load(SETTINGS_PATH) != godot::global::Error::OK {
            return Ok(Self::default()); // no file yet → all defaults
        }

        let defaults = VolumeSettings::default();
        let volume_settings = VolumeSettings::new(
            load_volume(&config, "master_volume", defaults.get_master_volume())?,
            load_volume(&config, "music_volume", defaults.get_music_volume())?,
            load_volume(&config, "sfx_volume", defaults.get_sfx_volume())?,
        );

        info!("Loaded volume settings: {:?}", volume_settings);

        let audio_output_device = load_audio_output_device(&config)?;
        if let Some(device) = audio_output_device.clone() {
            info!("Loaded audio output device: {:?}", device);
            let mut audio_server = AudioServer::singleton();
            audio_server.set_output_device(device.to_string().as_str());
        }

        Ok(Self {
            volume_settings,
            audio_output_device,
        })
    }
}

fn load_volume(
    config: &Gd<ConfigFile>,
    key: &str,
    default: Volume,
) -> Result<Volume, GameSettingsError> {
    if !config.has_section_key(VolumeSettings::SETTINGS_SECTION, key) {
        return Ok(default);
    }

    let value = config
        .get_value(VolumeSettings::SETTINGS_SECTION, key)
        .try_to::<f64>()
        .map_err(|_| GameSettingsError::WrongType {
            key: key.to_owned(),
        })?;

    Volume::try_from(value).map_err(|source| GameSettingsError::InvalidVolume {
        key: key.to_owned(),
        source,
    })
}

fn load_audio_output_device(
    config: &Gd<ConfigFile>,
) -> Result<Option<AudioOutputDevice>, GameSettingsError> {
    if !config.has_section_key(
        AudioOutputDevice::SETTINGS_SECTION,
        AudioOutputDevice::SETTINGS_KEY,
    ) {
        return Ok(None);
    }

    let value = config
        .get_value(
            AudioOutputDevice::SETTINGS_SECTION,
            AudioOutputDevice::SETTINGS_KEY,
        )
        .try_to::<String>()
        .map_err(|_| GameSettingsError::WrongType {
            key: AudioOutputDevice::SETTINGS_KEY.to_owned(),
        })?;
    Ok(Some(AudioOutputDevice::from(value)))
}

fn on_save_settings_requested(
    _trigger: On<SaveSettingsRequested>,
    game_settings: Res<GameSettings>,
) {
    info!("Saving settings");
    game_settings.save_settings();
}

fn on_load_settings_requested(
    _trigger: On<LoadSettingsRequested>,
    mut game_settings: ResMut<GameSettings>,
) {
    info!("Loading settings");
    match GameSettings::load_settings() {
        Ok(loaded) => *game_settings = loaded,
        Err(e) => error!("Failed to load settings: {}", e),
    }
}
