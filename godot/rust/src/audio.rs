use bevy::{
    app::{App, Plugin},
    prelude::*,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::{
    LoadingStateAppExt,
    config::{ConfigureLoadingState, LoadingStateConfig},
};
use derive_more::{AsRef, Deref};
use godot::classes::AudioServer;
use godot::global::linear_to_db;
use godot::obj::Singleton;
use godot_bevy::plugins::signals::GodotSignalsPlugin;
use godot_bevy::prelude::{AudioApp, AudioChannel, AudioChannelMarker, GodotResource};
use std::ops::Mul;

use crate::game_settings::{GameSettings, Volume};

/// Master bus name as it appears in `audio_bus_layout.tres`. `Master` is the
/// implicit bus 0 and always exists.
const MASTER_BUS: &str = "Master";

#[derive(Resource, Debug, Clone, Copy, PartialEq, AsRef, Deref)]
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

pub struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        // Apply once at startup so the loaded settings take effect, then again
        // whenever `GameSettings` is mutated (e.g. from the settings menu).
        // Load the audio collection during `GameState::Loading` so `GameAudio`
        // exists before `on_play_sfx` (or anything else) reads it.
        app.configure_loading_state(
            LoadingStateConfig::new(crate::GameState::Loading).load_collection::<GameAudio>(),
        )
        .add_systems(Startup, apply_master_volume_settings)
        .add_audio_channel::<MusicChannel>()
        .add_audio_channel::<SfxChannel>()
        .add_systems(
            Update,
            apply_master_volume_settings.run_if(resource_changed::<GameSettings>),
        )
        .add_plugins(GodotSignalsPlugin::<PlaySfxMessage>::default())
        .add_observer(on_play_sfx);
    }
}

/// Audio channel for game music
#[derive(Resource)]
pub struct MusicChannel;

impl AudioChannelMarker for MusicChannel {
    const CHANNEL_NAME: &'static str = "Music";
}

/// Audio channel for game sound effects
#[derive(Resource)]
pub struct SfxChannel;

impl AudioChannelMarker for SfxChannel {
    const CHANNEL_NAME: &'static str = "SFX";
}

/// Audio assets loaded via bevy_asset_loader
#[derive(AssetCollection, Resource, Debug)]
pub struct GameAudio {
    #[asset(path = "audio/test_sound.mp3")]
    pub test_sound: Handle<GodotResource>,
}

/// Event to trigger sound effects
#[derive(Event, Debug, Clone)]
pub enum PlaySfxMessage {
    TestSound,
}

trait PlaySfxMessageGain {
    fn individual_gain(&self) -> Gain;
    fn gain(&self, sfx: &Gain) -> Gain;
}

impl PlaySfxMessageGain for PlaySfxMessage {
    fn individual_gain(&self) -> Gain {
        match self {
            PlaySfxMessage::TestSound => Gain::new(0.8),
        }
    }
    fn gain(&self, sfx: &Gain) -> Gain {
        self.individual_gain() * sfx
    }
}

fn on_play_sfx(
    trigger: On<PlaySfxMessage>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    game_audio: Res<GameAudio>,
    settings: Res<GameSettings>,
) {
    // godot-bevy channels all play on the Master bus, so the SFX volume can't be
    // applied at the bus level; fold it into each sound's gain instead.
    let sfx = settings.get_volume_settings().sfx_scalar();
    match trigger.event() {
        message @ PlaySfxMessage::TestSound => {
            sfx_channel
                .play(game_audio.test_sound.clone())
                .volume(*message.gain(&sfx));
            info!("Played test sound effect");
        }
    }
}

fn apply_master_volume_settings(settings: Res<GameSettings>) {
    let volume = settings.get_volume_settings();
    let mut server = AudioServer::singleton();
    let index = server.get_bus_index(MASTER_BUS);
    if index < 0 {
        godot::global::godot_error!("Audio bus '{MASTER_BUS}' not found");
        return;
    }

    // 0 linear must map to -inf dB (silence); linear_to_db(0) already does this.
    let linear = *volume.get_master_volume() / Volume::MAX; // 0.0..=1.0
    server.set_bus_volume_db(index, linear_to_db(linear) as f32);
}
