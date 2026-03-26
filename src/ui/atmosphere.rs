use super::{CitySkyShell, MainCamera, SunLight};
use crate::game::GameState;
use crate::settings::{ActiveRenderTier, GraphicsSettings, RenderCapability};
use bevy::color::Mix;
use bevy::light::{
    CascadeShadowConfigBuilder, GlobalAmbientLight, NotShadowCaster, NotShadowReceiver,
};
use bevy::prelude::*;
use std::f32::consts::TAU;

pub fn setup_atmosphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
) {
    ambient_light.color = Color::srgb(0.44, 0.50, 0.62);
    ambient_light.brightness = 54.0;

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.88),
            illuminance: 28_000.0,
            shadows_enabled: false,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 26.0,
            maximum_distance: 132.0,
            ..default()
        }
        .build(),
        Transform::from_translation(Vec3::ZERO).looking_at(Vec3::new(-0.48, -0.82, 0.24), Vec3::Y),
        SunLight,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.30, 0.44),
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_scale(Vec3::splat(420.0)),
        NotShadowCaster,
        NotShadowReceiver,
        CitySkyShell,
    ));
}

pub fn sync_atmosphere(
    game: Res<GameState>,
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    camera: Single<
        (&Transform, &mut DistanceFog),
        (With<MainCamera>, Without<CitySkyShell>, Without<SunLight>),
    >,
    sun: Single<
        (&mut DirectionalLight, &mut Transform),
        (With<SunLight>, Without<MainCamera>, Without<CitySkyShell>),
    >,
    sky_shell: Single<
        (&MeshMaterial3d<StandardMaterial>, &mut Transform),
        (With<CitySkyShell>, Without<MainCamera>, Without<SunLight>),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let active_render_tier = graphics_settings.active_render_tier(*render_capability);
    let hour = game.hour_of_day() as f32;
    let daylight = daylight_factor(hour);
    let horizon_glow = horizon_glow_factor(hour) * (1.0 - daylight * 0.42);

    let night_sky = Color::srgb(0.03, 0.04, 0.08);
    let dawn_sky = Color::srgb(0.76, 0.45, 0.30);
    let day_sky = Color::srgb(0.45, 0.68, 0.92);
    let sky_color = night_sky
        .mix(&dawn_sky, horizon_glow)
        .mix(&day_sky, daylight);

    let night_fog = Color::srgb(0.07, 0.08, 0.12);
    let dawn_fog = Color::srgb(0.56, 0.40, 0.31);
    let day_fog = Color::srgb(0.66, 0.73, 0.82);
    let extinction_color = night_fog
        .mix(&dawn_fog, horizon_glow)
        .mix(&day_fog, daylight);

    let night_inscatter = Color::srgb(0.12, 0.13, 0.18);
    let dawn_inscatter = Color::srgb(0.94, 0.68, 0.46);
    let day_inscatter = Color::srgb(0.90, 0.95, 1.0);
    let inscattering_color = night_inscatter
        .mix(&dawn_inscatter, horizon_glow)
        .mix(&day_inscatter, daylight);

    let night_sun = Color::srgb(0.34, 0.38, 0.56);
    let dawn_sun = Color::srgb(1.0, 0.74, 0.46);
    let day_sun = Color::srgb(1.0, 0.98, 0.91);
    let sun_color = night_sun
        .mix(&dawn_sun, horizon_glow)
        .mix(&day_sun, daylight);

    let (camera_transform, mut fog) = camera.into_inner();
    let (mut sun_light, mut sun_transform) = sun.into_inner();
    let (sky_material_handle, mut sky_transform) = sky_shell.into_inner();

    clear_color.0 = sky_color.mix(&extinction_color, 0.22);
    ambient_light.color = sky_color.mix(&Color::WHITE, 0.26);
    ambient_light.brightness = match active_render_tier {
        ActiveRenderTier::FallbackDebug => 18.0,
        ActiveRenderTier::ShippingLow => 34.0 + daylight * 40.0,
        ActiveRenderTier::Full3d => 42.0 + daylight * 58.0,
    };

    let sky_translation = camera_transform.translation;
    sky_transform.translation = sky_translation;
    if let Some(material) = materials.get_mut(&sky_material_handle.0) {
        material.base_color = sky_color.mix(&inscattering_color, 0.16);
    }

    let sun_target = sun_direction(hour, daylight);
    *sun_transform = Transform::from_translation(Vec3::ZERO).looking_at(sun_target, Vec3::Y);
    sun_light.color = sun_color;
    sun_light.illuminance = match active_render_tier {
        ActiveRenderTier::FallbackDebug => 0.0,
        ActiveRenderTier::ShippingLow => 1_200.0 + daylight * 22_000.0,
        ActiveRenderTier::Full3d => 1_600.0 + daylight * 34_000.0,
    };
    sun_light.shadows_enabled =
        matches!(active_render_tier, ActiveRenderTier::Full3d) && daylight > 0.18;

    fog.color = extinction_color.with_alpha(1.0);
    fog.directional_light_color = sun_color.with_alpha(match active_render_tier {
        ActiveRenderTier::FallbackDebug => 0.0,
        ActiveRenderTier::ShippingLow => 0.34 + daylight * 0.18,
        ActiveRenderTier::Full3d => 0.40 + daylight * 0.24,
    });
    fog.directional_light_exponent = 12.0 + daylight * 26.0;
    fog.falloff = match active_render_tier {
        ActiveRenderTier::FallbackDebug => FogFalloff::Linear {
            start: 260.0,
            end: 340.0,
        },
        ActiveRenderTier::ShippingLow => {
            FogFalloff::from_visibility_colors(112.0, extinction_color, inscattering_color)
        }
        ActiveRenderTier::Full3d => {
            FogFalloff::from_visibility_colors(148.0, extinction_color, inscattering_color)
        }
    };
}

fn daylight_factor(hour: f32) -> f32 {
    let cycle = ((hour / 24.0) * TAU - TAU * 0.25).sin();
    smoothstep(-0.18, 0.24, cycle)
}

fn horizon_glow_factor(hour: f32) -> f32 {
    let sunrise = 1.0 - ((hour - 6.0).abs() / 3.0).clamp(0.0, 1.0);
    let sunset = 1.0 - ((hour - 18.0).abs() / 3.0).clamp(0.0, 1.0);
    sunrise.max(sunset)
}

fn sun_direction(hour: f32, daylight: f32) -> Vec3 {
    let azimuth = -0.9 + (hour / 24.0) * TAU * 0.72;
    let vertical = 0.18 + daylight * 0.92;
    Vec3::new(azimuth.cos() * 0.56, -vertical, azimuth.sin() * 0.56)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
