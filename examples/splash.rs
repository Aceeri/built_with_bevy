use bevy::{prelude::*, window::PresentMode};
use built_with_bevy::{
    BevySplashscreenEnded, BevySplashscreenPlugin, SkipBevySplashscreen, StartBevySplashscreen,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BevySplashscreenPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, debug_keys)
        .add_observer(on_ended)
        .run();
}

fn setup(mut commands: Commands) {
    commands.trigger(StartBevySplashscreen);
}

fn debug_keys(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        commands.trigger(StartBevySplashscreen);
    }
    if keys.just_pressed(KeyCode::KeyS) {
        commands.trigger(SkipBevySplashscreen);
    }
}

fn on_ended(_: On<BevySplashscreenEnded>) {
    info!("splashscreen ended");
}
