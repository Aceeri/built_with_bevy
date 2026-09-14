//! 'Built with Bevy' Splashscreen
//!
//! Add [`BevySplashscreenPlugin`] to your app, then:
//!
//! - `commands.trigger(StartBevySplashscreen)` to play the splash.
//! - `commands.trigger(SkipBevySplashscreen)` to end it early.
//! - `app.add_observer(|_: On<BevySplashscreenEnded>| { })` to react to end.
//!
//! The plugin adds [`VelloPlugin`] with default settings if you haven't already
//! added it yourself.

use bevy::{
    prelude::*,
    render::{
        ExtractSchedule, MainWorld, Render, RenderApp, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_resource::PipelineCache,
        view::{ExtractedWindows, ViewTarget},
    },
};
use bevy_vello::{VelloPlugin, integrations::svg::load_svg_from_str, prelude::*};

const BIRD_SVG: &str = include_str!("../assets/bird-0.svg");
const BUILT_SVG: &str = include_str!("../assets/built.svg");
const WITH_SVG: &str = include_str!("../assets/with.svg");
const BEVY_TEXT_SVG: &str = include_str!("../assets/bevy_text.svg");

const BIRD_SOURCE_FILL: &str = "#ececec"; // EKEKEKEKEK, I think a cat got in here
const BIRD_SOURCE_BG_FILL: &str = "#232326";
const BEVY_TEXT_SOURCE_FILL: &str = "#ececec";
const BUILT_TEXT_SOURCE_FILL: &str = "#78787f";
const WITH_TEXT_SOURCE_FILL: &str = "#78787f";
const BIRD_NAMES: [&str; 3] = ["Birb 0 (front)", "Birb 1 (middle)", "Birb 2 (back)"];

const BIRD_SLIDE_OFFSET: f32 = 20.0;

/// Customization options for the splashscreen
#[derive(Resource)]
pub struct BevySplashscreenOptions {
    /// Set the background for the splashscreen.
    ///
    /// Defaults to a darkish grey: #232326
    pub background_color: Color,
    /// The splashscreen uses a separate 2D camera to display over all others.
    ///
    /// This sets the camera ordering for it.
    ///
    /// Defaults to 9999.
    pub splash_camera_order: isize,
    /// Colors of the bevy birbs, front to back.
    ///
    /// Defaults to:
    /// [#ececec /* cat got in here */, #b2b2b2, #787878]
    pub bird_colors: [Color; 3],

    /// Color of the "BEVY" text.
    ///
    /// Defaults to #ececec
    pub bevy_text_color: Color,

    /// Color of the "built" text.
    ///
    /// Defaults to #78787f
    pub built_text_color: Color,

    /// Color of the "with" text.
    ///
    /// Defaults to #78787f
    pub with_text_color: Color,

    // Animation options
    // TODO: Maybe worthwhile making it a bit more granular, but this is good enough for now.
    /// How long each fade takes.
    ///
    /// Defaults to 0.6s
    pub fade_duration: f32,
    /// How long each keyframe takes.
    ///
    /// Defaults to 0.7s
    pub keyframe_duration: f32,
    /// How long the full splashscreen holds before it starts fading.
    ///
    /// Defaults to 1.1s
    pub hold_duration: f32,
}

impl Default for BevySplashscreenOptions {
    fn default() -> Self {
        Self {
            background_color: Color::srgb_u8(0x23, 0x23, 0x26),
            splash_camera_order: 9999,
            bird_colors: [
                Color::srgb_u8(0xec, 0xec, 0xec),
                Color::srgb_u8(0xb2, 0xb2, 0xb2),
                Color::srgb_u8(0x78, 0x78, 0x78),
            ],

            bevy_text_color: Color::srgb_u8(0xec, 0xec, 0xec),
            built_text_color: Color::srgb_u8(0x78, 0x78, 0x7f),
            with_text_color: Color::srgb_u8(0x78, 0x78, 0x7f),

            fade_duration: 0.6,
            keyframe_duration: 0.7,
            hold_duration: 1.1,
        }
    }
}

/// Add to your project to set up the splashscreen.
///
/// This adds [`VelloPlugin`] if it is not already added.
///
/// `commands.trigger(StartBevySplashscreen)` to play the splash.
/// `commands.trigger(SkipBevySplashscreen)` to end it early.
/// `app.add_observer(|_: On<BevySplashscreenEnded>| { })` to react to end.
#[derive(Default)]
pub struct BevySplashscreenPlugin;

impl Plugin for BevySplashscreenPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<VelloPlugin>() {
            app.add_plugins(VelloPlugin::default());
        }

        app.init_resource::<BevySplashscreenOptions>()
            .init_resource::<RenderReadiness>()
            .add_plugins(ExtractComponentPlugin::<SplashCamera>::default())
            .register_type::<Fade>()
            .register_type::<KeyframeInterp>()
            .register_type::<KeyFrame>()
            .add_observer(on_start)
            .add_observer(on_skip)
            .add_observer(on_trigger_fade)
            .add_observer(on_trigger_keyframe)
            .add_systems(Update, (splash_dispatch, elapsed, fade, keyframe).chain());

        match app.get_sub_app_mut(RenderApp) {
            Some(render_app) => {
                render_app
                    .init_resource::<RenderReadiness>()
                    .add_systems(ExtractSchedule, sync_render_readiness)
                    .add_systems(
                        Render,
                        record_render_readiness.in_set(RenderSystems::Cleanup),
                    );
            }
            None => {
                app.insert_resource(RenderReadiness::READY);
            }
        }
    }
}

#[derive(Component, Clone, ExtractComponent)]
struct SplashCamera;

#[derive(Resource, Clone, Copy, Default, Debug)]
struct RenderReadiness {
    // a window has presented at least one frame.
    // TODO: maybe find SplashCamera and check specifically whether
    // THAT window has presented at least once?
    window_presented: bool,
    // no render pipelines are queued or still compiling.
    pipelines_idle: bool,
    // the splash camera made it through extraction and has a view target.
    camera_rendered: bool,
}

impl RenderReadiness {
    const READY: Self = Self {
        window_presented: true,
        pipelines_idle: true,
        camera_rendered: true,
    };

    fn ready(&self) -> bool {
        self.window_presented && self.pipelines_idle && self.camera_rendered
    }
}

fn record_render_readiness(
    mut readiness: ResMut<RenderReadiness>,
    windows: Res<ExtractedWindows>,
    pipeline_cache: Res<PipelineCache>,
    splash_views: Query<(), (With<SplashCamera>, With<ViewTarget>)>,
) {
    *readiness = RenderReadiness {
        window_presented: windows
            .windows
            .values()
            .any(|window| !window.needs_initial_present),
        pipelines_idle: pipeline_cache.waiting_pipelines().next().is_none(),
        camera_rendered: !splash_views.is_empty(),
    };
}

fn sync_render_readiness(readiness: Res<RenderReadiness>, mut main_world: ResMut<MainWorld>) {
    main_world.insert_resource(*readiness);
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
    started: bool,
    elapsed: f32,

    birds: [Entity; 3],
    built: Entity,
    with: Entity,
    bevy: Entity,
    cutoff: Entity,
    overlay: Entity,

    // time waited for render pipelines
    waited: f32,
}

fn color_to_hex(color: Color) -> String {
    let c = color.to_srgba();
    format!(
        "#{:02x}{:02x}{:02x}",
        (c.red * 255.0).round() as u8,
        (c.green * 255.0).round() as u8,
        (c.blue * 255.0).round() as u8
    )
}

fn bake_svg_fill(svg: &str, fills: &[(&str, Color)]) -> VelloSvg {
    let mut recolored = svg.to_owned();
    for (placeholder, fill) in fills {
        recolored = recolored.replace(placeholder, &color_to_hex(*fill));
    }

    load_svg_from_str(&recolored).expect("bundled svg failed to parse")
}

fn on_start(
    _: On<StartBevySplashscreen>,
    existing: Option<Res<Splash>>,
    options: Res<BevySplashscreenOptions>,
    mut commands: Commands,
    mut svgs: ResMut<Assets<VelloSvg>>,
) {
    if existing.is_some() {
        return;
    }

    let bg_hex = color_to_hex(options.background_color);

    commands.spawn((
        Name::new("Splash camera"),
        SplashCamera,
        Camera2d,
        Camera {
            order: options.splash_camera_order,
            clear_color: ClearColorConfig::Custom(options.background_color),
            ..default()
        },
        VelloView,
        Msaa::Sample4,
        SplashEntity,
    ));

    let built = commands
        .spawn((
            Name::new("'Built' text"),
            VelloSvg2d(svgs.add(bake_svg_fill(
                BUILT_SVG,
                &[(BUILT_TEXT_SOURCE_FILL, options.built_text_color)],
            ))),
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
            VelloSvg2d(svgs.add(bake_svg_fill(
                WITH_SVG,
                &[(WITH_TEXT_SOURCE_FILL, options.with_text_color)],
            ))),
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
            VelloSvg2d(svgs.add(bake_svg_fill(
                BEVY_TEXT_SVG,
                &[(BEVY_TEXT_SOURCE_FILL, options.bevy_text_color)],
            ))),
            VelloSvgAnchor::Center,
            Transform::from_xyz(90.0, -20.0, 0.0),
            Visibility::Hidden,
            Fade::default(),
            SplashEntity,
        ))
        .id();

    let bodies: [Handle<VelloSvg>; 3] = std::array::from_fn(|i| {
        svgs.add(bake_svg_fill(
            BIRD_SVG,
            &[
                (BIRD_SOURCE_FILL, options.bird_colors[i]),
                (BIRD_SOURCE_BG_FILL, options.background_color),
            ],
        ))
    });

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
        started: false,
        waited: 0.0,
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
    real: Res<Time<Real>>,
    options: Res<BevySplashscreenOptions>,
    readiness: Res<RenderReadiness>,
    splash: Option<ResMut<Splash>>,
    splash_entities: Query<Entity, With<SplashEntity>>,
    fades: Query<&Fade>,
    mut commands: Commands,
) {
    let Some(mut splash) = splash else {
        return;
    };

    if !splash.started {
        splash.waited += real.delta_secs();
        if !readiness.ready() {
            return;
        }

        info!("splash render ready after {:.0}ms", splash.waited * 1000.0);
        splash.started = true;
    }

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

    c += options.fade_duration;
    tasks.extend([
        (c, Step::Show(splash.cutoff)),
        (c, Step::Show(splash.birds[1])),
        (c, Step::Show(splash.birds[2])),
        (c, Step::Keyframe(splash.birds[1])),
        (c, Step::Keyframe(splash.birds[2])),
        (c, Step::Keyframe(splash.cutoff)),
        (c, Step::Keyframe(splash.birds[0])),
    ]);

    c += options.keyframe_duration + options.hold_duration;
    tasks.push((c, Step::Fade(splash.overlay)));

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

    let overlay_covered = fades
        .get(splash.overlay)
        .is_ok_and(|fade| fade.start && fade.elapsed >= options.fade_duration);
    if overlay_covered {
        end_splash(&mut commands, &splash_entities);
    }
}

fn fade(
    fades: Query<(&Fade, &VelloSvg2d)>,
    mut svgs: ResMut<Assets<VelloSvg>>,
    options: Res<BevySplashscreenOptions>,
) {
    for (fade, vello) in &fades {
        if !fade.start {
            continue;
        }

        let t = (fade.elapsed / options.fade_duration).clamp(0.0, 1.0);
        let alpha = fade.from + (fade.to - fade.from) * t;
        if let Some(mut svg) = svgs.get_mut(&vello.0) {
            svg.alpha = alpha;
        }
    }
}

fn keyframe(
    mut keyed: Query<(&mut Transform, &KeyframeInterp)>,
    options: Res<BevySplashscreenOptions>,
) {
    for (mut transform, interp) in &mut keyed {
        if !interp.start {
            continue;
        }

        transform.translation = interp.keyframe.start.translation.lerp(
            interp.keyframe.end.translation,
            interp
                .translation
                .sample_clamped(interp.elapsed / options.keyframe_duration),
        );
        transform.rotation = interp.keyframe.start.rotation.slerp(
            interp.keyframe.end.rotation,
            interp
                .rotation
                .sample_clamped(interp.elapsed / options.keyframe_duration),
        );
        transform.scale = interp.keyframe.start.scale.lerp(
            interp.keyframe.end.scale,
            interp
                .scale
                .sample_clamped(interp.elapsed / options.keyframe_duration),
        );
    }
}
