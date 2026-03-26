use super::helpers::{action_sidebar_bounds, cursor_over_ui, world_to_tile};
use super::{
    ActionRailAction, ActionRailButton, ActionSidebarScrollArea, ActionSidebarState,
    CloseEventModalButton, EventIconButton, EventModalState, HoverState, MainCamera, MinimapMode,
    MinimapViewButton, TimeControl, TopBarClockButton, TopBarSpeedButton,
};
use crate::game::GameState;
use crate::model::Tool;
use crate::settings::{GraphicsSettings, RenderCapability, SettingsMenuState};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub fn handle_ui_buttons(
    interactions: Query<
        (
            &Interaction,
            Option<&TopBarClockButton>,
            Option<&TopBarSpeedButton>,
            Option<&MinimapViewButton>,
            Option<&EventIconButton>,
            Option<&CloseEventModalButton>,
            Option<&super::ActionSidebarToggleButton>,
            Option<&ActionRailButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut game: ResMut<GameState>,
    mut time_control: ResMut<TimeControl>,
    mut minimap_mode: ResMut<MinimapMode>,
    mut event_modal: ResMut<EventModalState>,
    mut action_sidebar_state: ResMut<ActionSidebarState>,
) {
    for (
        interaction,
        clock_button,
        speed_button,
        minimap_button,
        event_button,
        close_button,
        sidebar_toggle_button,
        action_button,
    ) in &interactions
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if clock_button.is_some() {
            time_control.toggle_pause();
            game.push_log(if time_control.paused() {
                "Simulation paused.".to_string()
            } else {
                "Simulation resumed.".to_string()
            });
            continue;
        }

        if speed_button.is_some() {
            time_control.cycle_speed();
            game.push_log(format!(
                "Simulation speed set to {}.",
                time_control.speed_label()
            ));
            continue;
        }

        if let Some(button) = minimap_button {
            *minimap_mode = button.mode;
            continue;
        }

        if let Some(button) = event_button {
            if game.incidents.get(button.slot).is_some() {
                event_modal.selected_incident = Some(button.slot);
            }
            continue;
        }

        if sidebar_toggle_button.is_some() {
            action_sidebar_state.collapsed = !action_sidebar_state.collapsed;
            continue;
        }

        if let Some(button) = action_button {
            match button.action {
                ActionRailAction::Tool(tool) => {
                    game.selected_tool = tool;
                }
                ActionRailAction::Regenerate => game.regenerate(),
                ActionRailAction::CycleBridgeProfile => game.cycle_bridge_profile(),
                ActionRailAction::CycleLogicProfile => game.cycle_logic_profile(),
                ActionRailAction::SpendResearch => game.spend_research(),
            }
            continue;
        }

        if close_button.is_some() {
            event_modal.selected_incident = None;
        }
    }
}

pub fn handle_keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<GameState>,
    mut time_control: ResMut<TimeControl>,
    mut graphics_settings: ResMut<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    mut settings_menu: ResMut<SettingsMenuState>,
    mut event_modal: ResMut<EventModalState>,
    mut action_sidebar_state: ResMut<ActionSidebarState>,
) {
    if event_modal.selected_incident.is_some() && keys.just_pressed(KeyCode::Escape) {
        event_modal.selected_incident = None;
        return;
    }

    if keys.just_pressed(KeyCode::F1) {
        settings_menu.open = !settings_menu.open;
        game.push_log(if settings_menu.open {
            "Configuration opened. Use arrows to change graphics settings.".to_string()
        } else {
            "Configuration closed.".to_string()
        });
        return;
    }

    if keys.just_pressed(KeyCode::Tab) {
        action_sidebar_state.collapsed = !action_sidebar_state.collapsed;
        return;
    }

    if settings_menu.open {
        if keys.just_pressed(KeyCode::Escape) {
            settings_menu.open = false;
            game.push_log("Configuration closed.".to_string());
            return;
        }

        if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
            settings_menu.move_selection(-1);
            return;
        }

        if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
            settings_menu.move_selection(1);
            return;
        }

        let direction = if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA)
        {
            Some(-1)
        } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
            Some(1)
        } else {
            None
        };

        if let Some(direction) = direction {
            let row = settings_menu.selected_row();
            graphics_settings.adjust(row, direction);
            game.push_log(format!(
                "Config {} -> {}.",
                row.label(),
                graphics_settings.row_value_label(row, *render_capability)
            ));
        }
        return;
    }

    if keys.just_pressed(KeyCode::KeyR) {
        game.regenerate();
    }
    if keys.just_pressed(KeyCode::Digit1) {
        game.selected_tool = Tool::Bridge;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        game.selected_tool = Tool::Sensor;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        game.selected_tool = Tool::Plc;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        game.selected_tool = Tool::Gateway;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        game.selected_tool = Tool::Substation;
    }
    if keys.just_pressed(KeyCode::Digit6) {
        game.selected_tool = Tool::PumpStation;
    }
    if keys.just_pressed(KeyCode::Escape) {
        game.selected_tool = Tool::Inspect;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        game.cycle_bridge_profile();
    }
    if keys.just_pressed(KeyCode::KeyL) {
        game.cycle_logic_profile();
    }
    if keys.just_pressed(KeyCode::KeyU) {
        game.spend_research();
    }
    if keys.just_pressed(KeyCode::Space) {
        time_control.toggle_pause();
        game.push_log(if time_control.paused() {
            "Simulation paused.".to_string()
        } else {
            "Simulation resumed.".to_string()
        });
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        time_control.slower();
        game.push_log(format!(
            "Simulation speed set to {}.",
            time_control.speed_label()
        ));
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        time_control.faster();
        game.push_log(format!(
            "Simulation speed set to {}.",
            time_control.speed_label()
        ));
    }
}

pub fn scroll_action_sidebar(
    window: Single<&Window, With<PrimaryWindow>>,
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    scroll_area: Single<(&mut ScrollPosition, &Node, &ComputedNode), With<ActionSidebarScrollArea>>,
    action_sidebar_state: Res<ActionSidebarState>,
    event_modal: Res<EventModalState>,
) {
    if event_modal.selected_incident.is_some() {
        return;
    }

    let window = *window;
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (left, top, right, bottom) = action_sidebar_bounds(window, &action_sidebar_state);
    if cursor_position.x < left
        || cursor_position.x > right
        || cursor_position.y < top
        || cursor_position.y > bottom
    {
        return;
    }

    let (mut scroll_position, node, computed) = scroll_area.into_inner();
    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
    if max_offset.y <= 0.0 || node.overflow.y != OverflowAxis::Scroll {
        return;
    }

    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -mouse_wheel.y;
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= 26.0;
        }

        scroll_position.y = (scroll_position.y + delta).clamp(0.0, max_offset.y);
    }
}

pub fn advance_simulation(
    time: Res<Time>,
    mut time_control: ResMut<TimeControl>,
    mut game: ResMut<GameState>,
) {
    if time_control.paused() {
        return;
    }

    time_control.accumulator += time.delta_secs() * time_control.speed_multiplier();
    while time_control.accumulator >= crate::constants::SIM_SECONDS_PER_HOUR {
        time_control.accumulator -= crate::constants::SIM_SECONDS_PER_HOUR;
        game.advance_hour();
    }
}

pub fn update_hover_tile(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut hover_state: ResMut<HoverState>,
    graphics_settings: Res<GraphicsSettings>,
    settings_menu: Res<SettingsMenuState>,
    event_modal: Res<EventModalState>,
    action_sidebar_state: Res<ActionSidebarState>,
) {
    if settings_menu.open || event_modal.selected_incident.is_some() {
        hover_state.hovered_tile = None;
        return;
    }

    let (camera, camera_transform) = *camera_query;
    let window = *window;

    hover_state.hovered_tile = match window.cursor_position() {
        Some(cursor_position) => {
            if cursor_over_ui(
                window,
                cursor_position,
                &graphics_settings,
                &settings_menu,
                &event_modal,
                &action_sidebar_state,
            ) {
                None
            } else {
                match camera.viewport_to_world(camera_transform, cursor_position) {
                    Ok(ray) => match ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Y))
                    {
                        Some(distance) => world_to_tile(ray.get_point(distance)),
                        None => None,
                    },
                    Err(_) => None,
                }
            }
        }
        None => None,
    };
}

pub fn handle_mouse_clicks(
    window: Single<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    hover_state: Res<HoverState>,
    mut game: ResMut<GameState>,
    graphics_settings: Res<GraphicsSettings>,
    settings_menu: Res<SettingsMenuState>,
    event_modal: Res<EventModalState>,
    action_sidebar_state: Res<ActionSidebarState>,
) {
    let window = *window;

    if settings_menu.open || event_modal.selected_incident.is_some() {
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            if cursor_over_ui(
                window,
                cursor_position,
                &graphics_settings,
                &settings_menu,
                &event_modal,
                &action_sidebar_state,
            ) {
                return;
            }
        }

        if let Some(tile) = hover_state.hovered_tile {
            game.handle_click(tile);
        }
    }
}
