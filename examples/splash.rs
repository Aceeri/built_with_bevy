use bevy::{asset::LoadState, prelude::*, window::PresentMode};
use bevy_vello::prelude::VelloSvg;
use built_with_bevy::{
    BevySplashscreenEnded, BevySplashscreenOptions, BevySplashscreenPlugin, SkipBevySplashscreen,
    StartBevySplashscreen,
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
        .insert_resource(BevySplashscreenOptions {
            bird_colors: [
                Color::srgb(1.0, 0.0, 0.0),
                Color::srgb(0.0, 1.0, 0.0),
                Color::srgb(0.0, 0.0, 1.0),
            ],
            ..default()
        })
        .add_plugins(BevySplashscreenPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (start_when_loaded, debug_keys))
        .add_observer(on_ended)
        .run();
}

#[derive(Resource)]
struct PendingAssets(Vec<UntypedHandle>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handles = ["bird-0.svg", "built.svg", "with.svg", "bevy_text.svg"]
        .map(|path| asset_server.load::<VelloSvg>(path).untyped())
        .to_vec();
    commands.insert_resource(PendingAssets(handles));
}

fn start_when_loaded(
    pending: Option<Res<PendingAssets>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(pending) = pending else {
        return;
    };

    let mut all_done = true;
    for handle in &pending.0 {
        match asset_server.load_state(handle.id()) {
            LoadState::Loaded => {}
            LoadState::Failed(err) => error!("failed to load {:?}: {err}", handle.path()),
            _ => all_done = false,
        }
    }

    if !all_done {
        return;
    }

    info!("assets loaded, starting splashscreen");
    commands.remove_resource::<PendingAssets>();
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
