use bevy::ecs::resource::Resource;
use bevy::{app::{App, Plugin}, prelude::*};
use godot::global::godot_error;
use godot::{
    classes::ConfigFile, global::godot_print, meta::ToGodot, obj::{Gd, NewGd}, register::GodotConvert
};
use thiserror::Error;
use derive_more::{Display, AsRef, Deref};

const SETTINGS_PATH: &str = "user://settings.cfg";

pub struct GameSettingsPlugin;
impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        match GameSettings::load_settings() {
            Ok(settings) => {
                app.insert_resource(settings);
            }
            Err(e) => {
                // Fail early: a corrupt/invalid settings file should abort
                // startup rather than boot with a missing GameSettings
                // resource (which would panic later, far from the cause).
                godot_error!("Failed to load game settings: {}", e);
                panic!("Failed to load game settings: {e}");
            }
        }
    }
}


#[derive(Default, Resource)]
pub struct GameSettings {
    volume_settings: VolumeSettings,
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

    pub fn save_settings(&self) {
        let mut config = ConfigFile::new_gd();
        config.load(SETTINGS_PATH);

        let volume = &self.volume_settings;
        config.set_value(VolumeSettings::SETTINGS_SECTION, "master_volume", &(*volume.get_master_volume()).to_variant());
        config.set_value(VolumeSettings::SETTINGS_SECTION, "music_volume", &(*volume.get_music_volume()).to_variant());
        config.set_value(VolumeSettings::SETTINGS_SECTION, "sfx_volume", &(*volume.get_sfx_volume()).to_variant());

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

        godot_print!("Loaded volume settings: {:?}", volume_settings);
        Ok(Self { volume_settings })
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
        .map_err(|_| GameSettingsError::WrongType { key: key.to_owned() })?;

    Volume::try_from(value).map_err(|source| GameSettingsError::InvalidVolume {
        key: key.to_owned(),
        source,
    })
}


#[derive(Clone, Copy, PartialEq, Debug, Display, AsRef, Deref, GodotConvert)]
#[godot(transparent)]
pub struct Volume(f64);

#[derive(Error, Clone, Debug, PartialEq)]
#[error("volume {value} is outside {min}..={max}")]
pub struct VolumeError {
    value: f64,
    min: f64,
    max: f64,
}

impl Volume {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 100.0;

    pub const fn new(value: f64) -> Result<Self, VolumeError> {
        if value < Self::MIN || value > Self::MAX {
            Err(VolumeError {
                value,
                min: Self::MIN,
                max: Self::MAX,
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<f64> for Volume {
    type Error = VolumeError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self::new(Self::MAX / 2.0).expect("50 is within valid volume range")
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Display, Default)]
#[display("Master Volume: {master_volume}, Music Volume: {music_volume}, SFX Volume: {sfx_volume}")]
pub struct VolumeSettings {
    master_volume: Volume,
    music_volume: Volume,
    sfx_volume: Volume,
}

impl VolumeSettings {
    pub const SETTINGS_SECTION: &str = "audio";
    pub fn new(master_volume: Volume, music_volume: Volume, sfx_volume: Volume) -> Self {
        Self { master_volume, music_volume, sfx_volume }
    }
    pub fn get_master_volume(&self) -> Volume {
        self.master_volume
    }
    pub fn get_music_volume(&self) -> Volume {
        self.music_volume
    }
    pub fn get_sfx_volume(&self) -> Volume {
        self.sfx_volume
    }
    pub fn set_master_volume(&mut self, volume: Volume) {
        self.master_volume = volume;
    }
    pub fn set_music_volume(&mut self, volume: Volume) {
        self.music_volume = volume;
    }
    pub fn set_sfx_volume(&mut self, volume: Volume) {
        self.sfx_volume = volume;
    }
}
