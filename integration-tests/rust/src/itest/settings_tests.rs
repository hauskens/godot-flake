//! `jam-settings` in isolation: `GameSettingsPlugin` loads `user://settings.cfg` (falling
//! back to defaults on a fresh project) and inserts the `GameSettings` resource.

use godot_bevy_test::prelude::*;
use jam_core::Volume;
use jam_settings::{GameSettings, GameSettingsPlugin};

/// Booting with just `GameSettingsPlugin` makes `GameSettings` available with readable,
/// default volume settings.
#[itest(async)]
fn test_settings_defaults_loaded(ctx: &TestContext) -> godot::task::TaskHandle {
    let ctx = ctx.clone();
    godot::task::spawn(async move {
        let mut app = TestApp::new(&ctx, |app| {
            app.add_plugins(GameSettingsPlugin);
        })
        .await;

        app.update().await;

        app.with_world(|world| {
            let settings = world
                .get_resource::<GameSettings>()
                .expect("GameSettingsPlugin should insert the GameSettings resource");

            // On a fresh `user://` there is no settings.cfg, so values are defaults.
            assert_eq!(
                settings.get_volume_settings().get_master_volume(),
                Volume::default(),
                "expected default master volume on a fresh project"
            );
            assert_eq!(
                settings.get_volume_settings().get_music_volume(),
                Volume::default(),
                "expected default music volume on a fresh project"
            );
            assert_eq!(
                settings.get_volume_settings().get_sfx_volume(),
                Volume::default(),
                "expected default sfx volume on a fresh project"
            );
            assert_eq!(
                settings.get_audio_output_device(),
                None,
                "expected no audio output device on a fresh project"
            );
        });

        app.cleanup().await;
    })
}
