use super::helpers::cursor_over_ui;
use super::{ActionSidebarState, CameraPreset, CameraRig, EventModalState, MainCamera};
use crate::constants::{GRID_H, GRID_W, TILE_WORLD_SIZE};
use crate::game::GameState;
use crate::settings::{GraphicsSettings, SettingsMenuState};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::f32::consts::{PI, TAU};

const CAMERA_DAMPING: f32 = 10.0;
const CAMERA_ROTATION_SPEED: f32 = 0.0065;
const CAMERA_PAN_SCALAR: f32 = 1.55;
const KEYBOARD_PAN_SPEED: f32 = 26.0;
const KEYBOARD_ROTATION_SPEED: f32 = 1.7;
const KEYBOARD_ZOOM_SPEED: f32 = 2.8;
const CAMERA_MIN_DISTANCE: f32 = 22.0;
const CAMERA_MAX_DISTANCE: f32 = 126.0;
const CAMERA_MIN_PITCH: f32 = -1.24;
const CAMERA_MAX_PITCH: f32 = -0.34;

pub fn handle_camera_input(
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_reader: MessageReader<MouseMotion>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut camera_rig: ResMut<CameraRig>,
    mut game: ResMut<GameState>,
    graphics_settings: Res<GraphicsSettings>,
    settings_menu: Res<SettingsMenuState>,
    event_modal: Res<EventModalState>,
    action_sidebar_state: Res<ActionSidebarState>,
) {
    if keys.just_pressed(KeyCode::F2) {
        apply_camera_preset(&mut camera_rig, &mut game, CameraPreset::Strategic);
    }
    if keys.just_pressed(KeyCode::F3) {
        apply_camera_preset(&mut camera_rig, &mut game, CameraPreset::CityView);
    }
    if keys.just_pressed(KeyCode::F4) {
        apply_camera_preset(&mut camera_rig, &mut game, CameraPreset::Cinematic);
    }

    if !settings_menu.open && event_modal.selected_incident.is_none() {
        handle_keyboard_camera_controls(&keys, &mut camera_rig, time.delta_secs());
    }

    let mut motion_delta = Vec2::ZERO;
    for event in mouse_motion_reader.read() {
        motion_delta += event.delta;
    }

    let mut zoom_delta = 0.0;
    for event in mouse_wheel_reader.read() {
        zoom_delta += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.035,
        };
    }

    let window = *window;
    let cursor_over_overlay = window.cursor_position().is_some_and(|cursor_position| {
        cursor_over_ui(
            window,
            cursor_position,
            &graphics_settings,
            &settings_menu,
            &event_modal,
            &action_sidebar_state,
        )
    });

    if event_modal.selected_incident.is_some() || cursor_over_overlay {
        return;
    }

    if zoom_delta.abs() > f32::EPSILON {
        apply_zoom_delta(&mut camera_rig, zoom_delta);
    }

    if motion_delta.length_squared() <= f32::EPSILON {
        return;
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        camera_rig.target_yaw =
            wrap_angle(camera_rig.target_yaw - motion_delta.x * CAMERA_ROTATION_SPEED);
        camera_rig.target_pitch = (camera_rig.target_pitch
            - motion_delta.y * CAMERA_ROTATION_SPEED * 0.85)
            .clamp(CAMERA_MIN_PITCH, CAMERA_MAX_PITCH);
        return;
    }

    if mouse_buttons.pressed(MouseButton::Middle) {
        let viewport_scale = (window.height() / 900.0).max(0.72);
        let pan_per_pixel = camera_rig.target_distance / 900.0 * CAMERA_PAN_SCALAR * viewport_scale;
        let forward = Vec3::new(
            camera_rig.target_yaw.sin(),
            0.0,
            camera_rig.target_yaw.cos(),
        )
        .normalize_or_zero();
        let right = Vec3::new(forward.z, 0.0, -forward.x).normalize_or_zero();

        camera_rig.target_pivot -= right * motion_delta.x * pan_per_pixel;
        camera_rig.target_pivot += forward * motion_delta.y * pan_per_pixel;
        camera_rig.target_pivot = clamp_pivot(camera_rig.target_pivot);
    }
}

pub fn update_camera_transform(
    time: Res<Time>,
    mut camera_rig: ResMut<CameraRig>,
    mut camera_transform: Single<&mut Transform, With<MainCamera>>,
) {
    let blend = 1.0 - (-CAMERA_DAMPING * time.delta_secs()).exp();
    if blend <= f32::EPSILON {
        return;
    }

    camera_rig.current_pivot = camera_rig
        .current_pivot
        .lerp(camera_rig.target_pivot, blend);
    camera_rig.current_distance = camera_rig
        .current_distance
        .lerp(camera_rig.target_distance, blend);
    camera_rig.current_yaw = smooth_angle(camera_rig.current_yaw, camera_rig.target_yaw, blend);
    camera_rig.current_pitch += (camera_rig.target_pitch - camera_rig.current_pitch) * blend;

    **camera_transform = camera_rig.camera_transform();
}

fn apply_camera_preset(camera_rig: &mut CameraRig, game: &mut GameState, preset: CameraPreset) {
    camera_rig.apply_preset(preset, false);
    game.push_log(format!("Camera preset set to {}.", preset.label()));
}

fn handle_keyboard_camera_controls(
    keys: &ButtonInput<KeyCode>,
    camera_rig: &mut CameraRig,
    delta_secs: f32,
) {
    let mut pan_axis = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        pan_axis.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        pan_axis.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        pan_axis.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        pan_axis.x -= 1.0;
    }

    if pan_axis.length_squared() > 0.0 {
        let pan_axis = pan_axis.normalize();
        let view_forward = Vec3::new(
            -camera_rig.target_yaw.sin(),
            0.0,
            -camera_rig.target_yaw.cos(),
        )
        .normalize_or_zero();
        let right = view_forward.cross(Vec3::Y).normalize_or_zero();
        let pan_speed =
            KEYBOARD_PAN_SPEED * delta_secs * (camera_rig.target_distance / 56.0).max(0.65);

        camera_rig.target_pivot += view_forward * pan_axis.y * pan_speed;
        camera_rig.target_pivot += right * pan_axis.x * pan_speed;
        camera_rig.target_pivot = clamp_pivot(camera_rig.target_pivot);
    }

    let rotation_axis =
        (keys.pressed(KeyCode::KeyE) as i32 - keys.pressed(KeyCode::KeyQ) as i32) as f32;
    if rotation_axis.abs() > f32::EPSILON {
        camera_rig.target_yaw = wrap_angle(
            camera_rig.target_yaw + rotation_axis * KEYBOARD_ROTATION_SPEED * delta_secs,
        );
    }

    let zoom_axis = (keys.pressed(KeyCode::KeyX) as i32 + keys.pressed(KeyCode::PageDown) as i32
        - keys.pressed(KeyCode::KeyZ) as i32
        - keys.pressed(KeyCode::PageUp) as i32) as f32;
    if zoom_axis.abs() > f32::EPSILON {
        apply_zoom_delta(camera_rig, zoom_axis * KEYBOARD_ZOOM_SPEED * delta_secs);
    }
}

fn apply_zoom_delta(camera_rig: &mut CameraRig, zoom_units: f32) {
    if zoom_units.abs() <= f32::EPSILON {
        return;
    }

    let zoom_step = (camera_rig.target_distance * 0.11).clamp(2.6, 9.2);
    camera_rig.target_distance = (camera_rig.target_distance - zoom_units * zoom_step)
        .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
    camera_rig.target_pitch = zoom_pitch_for_distance(camera_rig.target_distance);
}

fn zoom_pitch_for_distance(distance: f32) -> f32 {
    let t = ((distance - CAMERA_MIN_DISTANCE) / (CAMERA_MAX_DISTANCE - CAMERA_MIN_DISTANCE))
        .clamp(0.0, 1.0);
    let eased = t * t * (3.0 - 2.0 * t);
    CAMERA_MAX_PITCH + (CAMERA_MIN_PITCH - CAMERA_MAX_PITCH) * eased
}

fn clamp_pivot(mut pivot: Vec3) -> Vec3 {
    let max_x = ((GRID_W - 1) as f32 * TILE_WORLD_SIZE) * 0.5 + TILE_WORLD_SIZE * 4.0;
    let max_z = ((GRID_H - 1) as f32 * TILE_WORLD_SIZE) * 0.5 + TILE_WORLD_SIZE * 4.0;
    pivot.x = pivot.x.clamp(-max_x, max_x);
    pivot.y = 0.0;
    pivot.z = pivot.z.clamp(-max_z, max_z);
    pivot
}

fn smooth_angle(current: f32, target: f32, blend: f32) -> f32 {
    current + shortest_angle_delta(current, target) * blend
}

fn shortest_angle_delta(current: f32, target: f32) -> f32 {
    (target - current + PI).rem_euclid(TAU) - PI
}

fn wrap_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}
