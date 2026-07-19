//! Pure value types shared across the jam crates.
//!
//! These live in `jam-core` so that `jam-settings` and `jam-audio` can both
//! depend on them without depending on each other: settings stores `Volume`s
//! and turns them into an `sfx_scalar()` `Gain`, while audio consumes that
//! `Gain`. Keeping the types here breaks the settings↔audio dependency cycle.

use std::ops::Mul;

use derive_more::{AsRef, Deref, Display};
use godot::classes::AudioServer;
use godot::obj::Singleton;
use godot::prelude::GString;
use godot::register::GodotConvert;
use thiserror::Error;

/// A normalized audio gain in `0.0..=1.0`, applied per-sound before playback.
#[derive(Debug, Clone, Copy, PartialEq, AsRef, Deref)]
pub struct Gain(f32);

impl Gain {
    pub const fn new(gain: f32) -> Self {
        Self(gain.clamp(0.0, 1.0))
    }
}

impl Mul<&Gain> for Gain {
    type Output = Gain;

    fn mul(self, rhs: &Gain) -> Self::Output {
        Gain::new(self.0 * rhs.0)
    }
}

/// A volume level on the `0..=100` scale used by the settings UI and config.
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
        Self {
            master_volume,
            music_volume,
            sfx_volume,
        }
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

    pub fn sfx_scalar(&self) -> Gain {
        Gain::new((*self.sfx_volume / Volume::MAX) as f32)
    }
}

/// A window resolution offered in the settings dropdown.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display("{width} x {height}")]
pub struct SceneResolution {
    width: i32,
    height: i32,
}

impl SceneResolution {
    pub const RESOLUTIONS: [SceneResolution; 3] = [
        SceneResolution::new(1280, 720),
        SceneResolution::new(1920, 1080),
        SceneResolution::new(2560, 1440),
    ];
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

#[derive(Clone, PartialEq, Eq, Debug, Display)]
pub struct AudioOutputDevice(String);

impl AudioOutputDevice {
    pub const SETTINGS_SECTION: &str = "audio";
    pub const SETTINGS_KEY: &str = "output_device";
    pub fn new(device: GString) -> Self {
        Self(device.to_string())
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
    pub fn from_current() -> Self {
        let mut server = AudioServer::singleton();
        Self(server.get_output_device().to_string())
    }
}

impl From<String> for AudioOutputDevice {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Default for AudioOutputDevice {
    fn default() -> Self {
        Self::from_current()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AudioOutputDeviceList(Vec<AudioOutputDevice>);

impl AudioOutputDeviceList {
    pub fn new(devices: Vec<AudioOutputDevice>) -> Self {
        Self(devices)
    }
    pub fn get_devices(&self) -> Vec<AudioOutputDevice> {
        self.0.clone()
    }
}

impl Default for AudioOutputDeviceList {
    fn default() -> Self {
        let mut server = AudioServer::singleton();
        let devices = server.get_output_device_list().to_vec();
        Self(
            devices
                .into_iter()
                .map(|device| AudioOutputDevice::new(device))
                .collect(),
        )
    }
}
