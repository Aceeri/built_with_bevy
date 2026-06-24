//! 'Built with Bevy' Splashscreen
//!
//! Add [`BevySplashscreenPlugin::default()`] to your app, then:
//!
//! - `commands.trigger(StartBevySplashscreen)` to play the splash.
//! - `commands.trigger(SkipBevySplashscreen)` to end it early.
//! - `app.add_observer(|_: On<BevySplashscreenEnded>, ...| ...)` to react to end.

use bevy::prelude::*;
use bevy_vello::{integrations::svg::load_svg_from_str, prelude::*};

const BIRD_SVG: &str = include_str!("../assets/bird-0.svg");
const BUILT_SVG: &str = include_str!("../assets/built.svg");
const WITH_SVG: &str = include_str!("../assets/with.svg");
const BEVY_TEXT_SVG: &str = include_str!("../assets/bevy_text.svg");

const BIRD_SOURCE_FILL: &str = "#ececec";
const BIRD_COLORS: [&str; 3] = ["#ececec", "#b2b2b2", "#787878"];
const BIRD_NAMES: [&str; 3] = ["Birb 0 (front)", "Birb 1 (middle)", "Birb 2 (back)"];

// TODO: These should all really just be configurable at the fade/keyframe callsite or something.
const FADE_DURATION: f32 = 0.6;
const KEYFRAME_DURATION: f32 = 0.7;
const HOLD_DURATION: f32 = 1.1;
const BIRD_SLIDE_OFFSET: f32 = 20.0;

const SPLASH_CAMERA_ORDER: isize = 9999;

#[derive(Default)]
pub struct BevySplashscreenPlugin;

impl Plugin for BevySplashscreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashBg(Color::srgb_u8(0x23, 0x23, 0x26)))
            .register_type::<Fade>()
            .register_type::<KeyframeInterp>()
            .register_type::<KeyFrame>()
            .add_observer(on_start)
            .add_observer(on_skip)
            .add_observer(on_trigger_fade)
            .add_observer(on_trigger_keyframe)
            .add_systems(Update, (splash_dispatch, elapsed, fade, keyframe).chain());
    }
}

/// Trigger this to play the splashscreen. Ignored if a splash is already running.
#[derive(Event, Default)]
pub struct StartBevySplashscreen;

/// Trigger this to end the splash immediately
#[derive(Event, Default)]
pub struct SkipBevySplashscreen;

/// Emitted after the splash finishes
#[derive(Event, Default)]
pub struct BevySplashscreenEnded;

#[derive(Resource)]
struct SplashBg(Color);

#[derive(Component)]
struct SplashEntity;

#[derive(Reflect, Clone, Default)]
struct KeyFrame {
    start: Transform,
    end: Transform,
}

fn bird_keyframes() -> [KeyFrame; 3] {
    [
        KeyFrame {
            start: Transform {
                translation: Vec3::new(-203.0 + BIRD_SLIDE_OFFSET, 0.5, 2.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.175),
                scale: Vec3::splat(3.8),
            },
            end: Transform {
                translation: Vec3::new(-203.0, 0.5, 2.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(3.8),
            },
        },
        KeyFrame {
            start: Transform {
                translation: Vec3::new(-200.0 + BIRD_SLIDE_OFFSET, 4.0, 1.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(3.3),
            },
            end: Transform {
                translation: Vec3::new(-175.7, 3.3, 1.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.5),
                scale: Vec3::splat(3.3),
            },
        },
        KeyFrame {
            start: Transform {
                translation: Vec3::new(-235.0 + BIRD_SLIDE_OFFSET, 0.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -0.0),
                scale: Vec3::splat(3.0),
            },
            end: Transform {
                translation: Vec3::new(-156.8, -6.0, 0.0),
                rotation: Quat::from_euler(EulerRot::XYZ, -0.0, 0.0, -1.1),
                scale: Vec3::splat(3.0),
            },
        },
    ]
}

#[derive(Resource)]
struct Splash {
    elapsed: f32,

    birds: [Entity; 3],
    built: Entity,
    with: Entity,
    bevy: Entity,
    cutoff: Entity,
    overlay: Entity,
}

fn bake_bird(fill_hex: &str) -> VelloSvg {
    let recolored = BIRD_SVG.replace(BIRD_SOURCE_FILL, fill_hex);
    load_svg_from_str(&recolored).expect("bird svg failed to parse")
}

fn on_start(
    _: On<StartBevySplashscreen>,
    existing: Option<Res<Splash>>,
    bg: Res<SplashBg>,
    mut commands: Commands,
    mut svgs: ResMut<Assets<VelloSvg>>,
) {
    if existing.is_some() {
        return;
    }

    let bg_hex = {
        let c = bg.0.to_srgba();
        format!(
            "#{:02x}{:02x}{:02x}",
            (c.red * 255.0).round() as u8,
            (c.green * 255.0).round() as u8,
            (c.blue * 255.0).round() as u8
        )
    };

    commands.spawn((
        Name::new("Splash camera"),
        Camera2d,
        Camera {
            order: SPLASH_CAMERA_ORDER,
            clear_color: ClearColorConfig::Custom(bg.0),
            ..default()
        },
        VelloView,
        Msaa::Sample4,
        SplashEntity,
    ));

    let built = commands
        .spawn((
            Name::new("'Built' text"),
            VelloSvg2d(svgs.add(load_svg_from_str(BUILT_SVG).expect("built svg failed to parse"))),
            VelloSvgAnchor::Center,
            Transform::from_xyz(55.0, 60.0, 0.0),
            Visibility::Hidden,
            Fade::default(),
            SplashEntity,
        ))
        .id();

    let with = commands
        .spawn((
            Name::new("'With' text"),
            VelloSvg2d(svgs.add(load_svg_from_str(WITH_SVG).expect("with svg failed to parse"))),
            VelloSvgAnchor::Center,
            Transform::from_xyz(180.0, 60.0, 0.0),
            Visibility::Hidden,
            Fade::default(),
            SplashEntity,
        ))
        .id();

    let bevy = commands
        .spawn((
            Name::new("Bevy text"),
            VelloSvg2d(
                svgs.add(load_svg_from_str(BEVY_TEXT_SVG).expect("bevy text svg failed to parse")),
            ),
            VelloSvgAnchor::Center,
            Transform::from_xyz(90.0, -20.0, 0.0),
            Visibility::Hidden,
            Fade::default(),
            SplashEntity,
        ))
        .id();

    let bodies: [Handle<VelloSvg>; 3] =
        std::array::from_fn(|i| svgs.add(bake_bird(BIRD_COLORS[i])));

    let cutoff_svg = svgs.add(
        load_svg_from_str(&format!(
            r#"<svg><rect width="1" height="1" fill="{bg_hex}"/></svg>"#
        ))
        .expect("cutoff svg failed to parse"),
    );

    let cutoff_rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, -0.3);
    let cutoff_scale = Vec3::new(50.0, 100.0, 1.0);
    let cutoff_keyframe = KeyFrame {
        start: Transform {
            translation: Vec3::new(-13.0, 5.8, 0.0),
            rotation: cutoff_rotation,
            scale: cutoff_scale,
        },
        end: Transform {
            translation: Vec3::new(-13.0, 5.8, 0.0),
            rotation: cutoff_rotation,
            scale: cutoff_scale,
        },
    };

    let cutoff = commands
        .spawn((
            Name::new("Tail/wing cutoff"),
            VelloSvg2d(cutoff_svg),
            VelloSvgAnchor::Center,
            cutoff_keyframe.start,
            Visibility::Hidden,
            KeyframeInterp::from_keyframe(cutoff_keyframe),
        ))
        .id();

    let overlay_svg = svgs.add(
        load_svg_from_str(&format!(
            r#"<svg><rect width="1" height="1" fill="{bg_hex}"/></svg>"#
        ))
        .expect("overlay svg failed to parse"),
    );

    let overlay = commands
        .spawn((
            Name::new("Fade-out overlay"),
            VelloSvg2d(overlay_svg),
            VelloSvgAnchor::Center,
            Transform::from_xyz(0.0, 0.0, 100.0).with_scale(Vec3::new(3000.0, 3000.0, 1.0)),
            Visibility::Hidden,
            Fade::default(),
            SplashEntity,
        ))
        .id();

    let mut birds = [Entity::PLACEHOLDER; 3];
    let keyframes = bird_keyframes();
    for index in 0u8..3 {
        let keyframe = keyframes[index as usize].clone();

        let mut bird_cmds = commands.spawn((
            Name::new(BIRD_NAMES[index as usize]),
            VelloSvg2d(bodies[index as usize].clone()),
            VelloSvgAnchor::Center,
            keyframe.start,
            Visibility::Hidden,
            KeyframeInterp {
                start: false,
                elapsed: 0.0,
                keyframe,

                translation: EaseFunction::BackInOut,
                rotation: EaseFunction::BackInOut,
                scale: EaseFunction::BackInOut,
            },
            SplashEntity,
        ));
        if index == 0 {
            bird_cmds.insert(Fade::default());
        }
        birds[index as usize] = bird_cmds.id();
    }

    commands.entity(cutoff).insert(ChildOf(birds[0]));

    commands.insert_resource(Splash {
        elapsed: 0.0,
        birds,
        built,
        with,
        bevy,
        cutoff,
        overlay,
    });
}

fn on_skip(
    _: On<SkipBevySplashscreen>,
    splash: Option<Res<Splash>>,
    splash_entities: Query<Entity, With<SplashEntity>>,
    mut commands: Commands,
) {
    if splash.is_none() {
        return;
    }
    end_splash(&mut commands, &splash_entities);
}

/// Despawns every splash entity
fn end_splash(commands: &mut Commands, splash_entities: &Query<Entity, With<SplashEntity>>) {
    for entity in splash_entities.iter() {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Splash>();
    commands.trigger(BevySplashscreenEnded);
}

/// A scheduled action on the splash timeline, applied by [`splash_dispatch`]
/// once its time is crossed.
enum Step {
    Fade(Entity),
    Keyframe(Entity),
    Show(Entity),
}

#[derive(EntityEvent)]
struct TriggerFade {
    entity: Entity,
}

fn on_trigger_fade(on: On<TriggerFade>, mut q: Query<(&mut Fade, &mut Visibility)>) {
    let Ok((mut fade, mut visibility)) = q.get_mut(on.entity) else {
        return;
    };
    fade.start = true;
    fade.elapsed = 0.0;
    *visibility = Visibility::Visible;
}

#[derive(EntityEvent)]
struct TriggerKeyFrame {
    entity: Entity,
}

fn on_trigger_keyframe(on: On<TriggerKeyFrame>, mut keyed: Query<&mut KeyframeInterp>) {
    let Ok(mut keyed) = keyed.get_mut(on.entity) else {
        return;
    };
    keyed.start = true;
    keyed.elapsed = 0.0;
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Fade {
    start: bool,
    elapsed: f32,
    from: f32,
    to: f32,
}

impl Default for Fade {
    fn default() -> Self {
        Self {
            start: false,
            elapsed: 0.0,
            from: 0.0,
            to: 1.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct KeyframeInterp {
    start: bool,
    elapsed: f32,
    keyframe: KeyFrame,

    translation: EaseFunction,
    rotation: EaseFunction,
    scale: EaseFunction,
}

impl KeyframeInterp {
    fn from_keyframe(key: KeyFrame) -> Self {
        Self {
            start: false,
            elapsed: 0.0,
            keyframe: key,
            translation: EaseFunction::CubicOut,
            rotation: EaseFunction::CubicOut,
            scale: EaseFunction::CubicOut,
        }
    }
}

fn elapsed(time: Res<Time>, mut fade: Query<&mut Fade>, mut key: Query<&mut KeyframeInterp>) {
    for mut fade in &mut fade {
        if fade.start {
            fade.elapsed += time.delta_secs();
        }
    }
    for mut key in &mut key {
        if key.start {
            key.elapsed += time.delta_secs();
        }
    }
}

fn splash_dispatch(
    time: Res<Time>,
    splash: Option<ResMut<Splash>>,
    splash_entities: Query<Entity, With<SplashEntity>>,
    mut commands: Commands,
) {
    let Some(mut splash) = splash else {
        return;
    };

    let last = splash.elapsed;
    splash.elapsed += time.delta_secs();

    let mut tasks = Vec::new();
    let mut c = 0.0;
    tasks.extend([
        (c, Step::Fade(splash.birds[0])),
        (c, Step::Fade(splash.built)),
        (c, Step::Fade(splash.with)),
        (c, Step::Fade(splash.bevy)),
    ]);

    c += FADE_DURATION;
    tasks.extend([
        (c, Step::Show(splash.cutoff)),
        (c, Step::Show(splash.birds[1])),
        (c, Step::Show(splash.birds[2])),
        (c, Step::Keyframe(splash.birds[1])),
        (c, Step::Keyframe(splash.birds[2])),
        (c, Step::Keyframe(splash.cutoff)),
        (c, Step::Keyframe(splash.birds[0])),
    ]);

    c += KEYFRAME_DURATION + HOLD_DURATION;
    tasks.push((c, Step::Fade(splash.overlay)));
    c += FADE_DURATION;

    for (t, step) in tasks {
        if t >= last && t < splash.elapsed {
            match step {
                Step::Fade(entity) => commands.trigger(TriggerFade { entity }),
                Step::Keyframe(entity) => commands.trigger(TriggerKeyFrame { entity }),
                Step::Show(entity) => {
                    commands.entity(entity).insert(Visibility::Visible);
                }
            }
        }
    }

    if splash.elapsed >= c {
        end_splash(&mut commands, &splash_entities);
    }
}

fn fade(fades: Query<(&Fade, &VelloSvg2d)>, mut svgs: ResMut<Assets<VelloSvg>>) {
    for (fade, vello) in &fades {
        if !fade.start {
            continue;
        }

        let t = (fade.elapsed / FADE_DURATION).clamp(0.0, 1.0);
        let alpha = fade.from + (fade.to - fade.from) * t;
        if let Some(svg) = svgs.get_mut(&vello.0) {
            svg.alpha = alpha;
        }
    }
}

fn keyframe(mut keyed: Query<(&mut Transform, &KeyframeInterp)>) {
    for (mut transform, interp) in &mut keyed {
        if !interp.start {
            continue;
        }

        transform.translation = interp.keyframe.start.translation.lerp(
            interp.keyframe.end.translation,
            interp
                .translation
                .sample_clamped(interp.elapsed / KEYFRAME_DURATION),
        );
        transform.rotation = interp.keyframe.start.rotation.slerp(
            interp.keyframe.end.rotation,
            interp
                .rotation
                .sample_clamped(interp.elapsed / KEYFRAME_DURATION),
        );
        transform.scale = interp.keyframe.start.scale.lerp(
            interp.keyframe.end.scale,
            interp
                .scale
                .sample_clamped(interp.elapsed / KEYFRAME_DURATION),
        );
    }
}
