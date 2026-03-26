use super::helpers::{
    action_button_border_color, action_button_color, action_button_label, event_badge_color,
    incident_action_copy, incident_badge_label, incident_title, minimap_button_color,
    minimap_tile_color, top_control_border_color, top_control_color, wrap_text, yes_no,
};
use super::{
    ActionRailButton, ActionRailButtonText, ActionSidebarExpandedOnly, ActionSidebarHelpText,
    ActionSidebarRoot, ActionSidebarState, EventIconButton, EventIconLabel, EventModalRoot,
    EventModalState, EventModalText, EventStripStatusText, HoverState, HudText, MinimapLegendText,
    MinimapMode, MinimapTileCell, MinimapViewButton, SettingsText, TimeControl, TopBarClockButton,
    TopBarClockText, TopBarMetricsText, TopBarSpeedButton, TopBarSpeedText,
    ACTION_BAR_WIDTH_COLLAPSED, ACTION_BAR_WIDTH_EXPANDED,
};
use crate::asset_pipeline::AssetCatalog;
use crate::game::GameState;
use crate::settings::{GraphicsSettings, RenderCapability, SettingsMenuRow, SettingsMenuState};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub fn update_top_bar_clock_text(
    top_bar_clock: Single<&mut Text, With<TopBarClockText>>,
    game: Res<GameState>,
) {
    if !game.is_changed() {
        return;
    }

    let mut top_bar_clock = top_bar_clock.into_inner();
    top_bar_clock.0 = game.clock_label();
}

pub fn update_top_bar_speed_text(
    top_bar_speed: Single<&mut Text, With<TopBarSpeedText>>,
    time_control: Res<TimeControl>,
) {
    if !time_control.is_changed() {
        return;
    }

    let mut top_bar_speed = top_bar_speed.into_inner();
    top_bar_speed.0 = format!("{}x", TimeControl::SPEEDS[time_control.speed_index]);
}

pub fn update_top_bar_metrics_text(
    top_bar_metrics: Single<&mut Text, With<TopBarMetricsText>>,
    game: Res<GameState>,
    diagnostics: Res<DiagnosticsStore>,
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
) {
    let diagnostics_dirty = graphics_settings.show_telemetry && diagnostics.is_changed();
    if !game.is_changed()
        && !graphics_settings.is_changed()
        && !render_capability.is_changed()
        && !diagnostics_dirty
    {
        return;
    }

    let total = game.coverage.total_demand.max(1);
    let power_pct = (100 * game.coverage.powered / total).clamp(0, 100);
    let water_pct = (100 * game.coverage.watered / total).clamp(0, 100);
    let telemetry_suffix = if graphics_settings.show_telemetry {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|diagnostic| diagnostic.smoothed())
            .map(|value| format!("{value:.0}"))
            .unwrap_or_else(|| "n/a".to_string());
        format!(
            "{} | FPS {}",
            graphics_settings
                .active_render_tier(*render_capability)
                .label(),
            fps
        )
    } else {
        graphics_settings
            .active_render_tier(*render_capability)
            .label()
            .to_string()
    };

    let mut top_bar_metrics = top_bar_metrics.into_inner();
    top_bar_metrics.0 = format!(
        "Budget ${} | Ops ${} | Power {}% | Water {}%\nScore S {} E {} OT {} T {} | {}",
        game.budget,
        game.ops_budget,
        power_pct,
        water_pct,
        game.score.smartness,
        game.score.engineering,
        game.score.ot_security,
        game.score.citizen_trust,
        telemetry_suffix
    );
}

pub fn update_top_bar_control_styles(
    clock_button: Single<
        (Ref<Interaction>, &mut BackgroundColor, &mut BorderColor),
        (With<TopBarClockButton>, Without<TopBarSpeedButton>),
    >,
    speed_button: Single<
        (Ref<Interaction>, &mut BackgroundColor, &mut BorderColor),
        (With<TopBarSpeedButton>, Without<TopBarClockButton>),
    >,
    time_control: Res<TimeControl>,
) {
    let (clock_interaction, mut clock_background, mut clock_border) = clock_button.into_inner();
    let (speed_interaction, mut speed_background, mut speed_border) = speed_button.into_inner();

    if !time_control.is_changed()
        && !clock_interaction.is_changed()
        && !speed_interaction.is_changed()
    {
        return;
    }

    *clock_background = BackgroundColor(top_control_color(
        *clock_interaction,
        time_control.paused(),
        false,
    ));
    *clock_border = BorderColor::all(top_control_border_color(time_control.paused()));
    *speed_background = BackgroundColor(top_control_color(*speed_interaction, false, true));
    *speed_border = BorderColor::all(top_control_border_color(false));
}

pub fn update_event_strip_status_text(
    event_strip_status: Single<&mut Text, With<EventStripStatusText>>,
    game: Res<GameState>,
) {
    if !game.is_changed() {
        return;
    }

    let mut event_strip_status = event_strip_status.into_inner();
    event_strip_status.0 = if game.incidents.is_empty() {
        "Events stable".to_string()
    } else {
        format!("{} active alert(s)", game.incidents.len())
    };
}

pub fn update_minimap_legend_text(
    minimap_legend: Single<&mut Text, With<MinimapLegendText>>,
    minimap_mode: Res<MinimapMode>,
) {
    if !minimap_mode.is_changed() {
        return;
    }

    let mut minimap_legend = minimap_legend.into_inner();
    minimap_legend.0 = format!("MINIMAP // {}", minimap_mode.label());
}

pub fn update_minimap_button_styles(
    mut minimap_buttons: Query<
        (&MinimapViewButton, Ref<Interaction>, &mut BackgroundColor),
        With<Button>,
    >,
    minimap_mode: Res<MinimapMode>,
) {
    let mode_changed = minimap_mode.is_changed();
    for (button, interaction, mut background) in &mut minimap_buttons {
        if !mode_changed && !interaction.is_changed() {
            continue;
        }

        *background = BackgroundColor(minimap_button_color(
            button.mode == *minimap_mode,
            *interaction,
        ));
    }
}

pub fn update_minimap_cell_colors(
    mut minimap_cells: Query<(&MinimapTileCell, &mut BackgroundColor)>,
    game: Res<GameState>,
    hover_state: Res<HoverState>,
    minimap_mode: Res<MinimapMode>,
) {
    if !game.is_changed() && !hover_state.is_changed() && !minimap_mode.is_changed() {
        return;
    }

    for (cell, mut background) in &mut minimap_cells {
        *background = BackgroundColor(minimap_tile_color(
            *minimap_mode,
            &game,
            cell.pos,
            hover_state.hovered_tile,
        ));
    }
}

pub fn update_event_badge_buttons(
    mut event_buttons: Query<
        (
            &EventIconButton,
            Ref<Interaction>,
            &mut BackgroundColor,
            &mut Visibility,
        ),
        With<Button>,
    >,
    game: Res<GameState>,
    event_modal: Res<EventModalState>,
) {
    let game_changed = game.is_changed();
    let modal_changed = event_modal.is_changed();
    for (button, interaction, mut background, mut visibility) in &mut event_buttons {
        if !game_changed && !modal_changed && !interaction.is_changed() {
            continue;
        }

        if let Some(incident) = game.incidents.get(button.slot) {
            *visibility = Visibility::Visible;
            *background = BackgroundColor(event_badge_color(
                incident.kind,
                *interaction,
                event_modal.selected_incident == Some(button.slot),
            ));
        } else {
            *visibility = Visibility::Hidden;
            *background = BackgroundColor(Color::srgba(0.16, 0.18, 0.22, 0.75));
        }
    }
}

pub fn update_event_badge_labels(
    mut event_labels: Query<(&EventIconLabel, &mut Text)>,
    game: Res<GameState>,
) {
    if !game.is_changed() {
        return;
    }

    for (label, mut text) in &mut event_labels {
        text.0 = game
            .incidents
            .get(label.slot)
            .map(|incident| incident_badge_label(incident.kind).to_string())
            .unwrap_or_default();
    }
}

pub fn update_event_modal_panel(
    modal_root: Single<&mut Visibility, With<EventModalRoot>>,
    modal_text: Single<&mut Text, With<EventModalText>>,
    game: Res<GameState>,
    mut event_modal: ResMut<EventModalState>,
) {
    if !game.is_changed() && !event_modal.is_changed() {
        return;
    }

    let mut modal_root = modal_root.into_inner();
    let mut modal_text = modal_text.into_inner();

    if let Some(index) = event_modal.selected_incident {
        if let Some(incident) = game.incidents.get(index) {
            *modal_root = Visibility::Visible;
            modal_text.0 = format!(
                "{}\nRemaining: {:.0}h\nRecommended response: {}\n\n{}",
                incident_title(incident.kind),
                incident.ttl,
                incident_action_copy(incident.kind),
                incident.note
            );
        } else {
            event_modal.selected_incident = None;
            *modal_root = Visibility::Hidden;
        }
    } else {
        *modal_root = Visibility::Hidden;
    }
}

pub fn update_hud_panel(
    hud_panel: Single<(&mut Text, &mut TextFont), With<HudText>>,
    game: Res<GameState>,
    hover_state: Res<HoverState>,
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    diagnostics: Res<DiagnosticsStore>,
    action_sidebar_state: Res<ActionSidebarState>,
) {
    let show_live_perf = graphics_settings.show_telemetry && !action_sidebar_state.collapsed;
    let diagnostics_dirty = show_live_perf && diagnostics.is_changed();
    if !game.is_changed()
        && !hover_state.is_changed()
        && !graphics_settings.is_changed()
        && !render_capability.is_changed()
        && !action_sidebar_state.is_changed()
        && !diagnostics_dirty
    {
        return;
    }

    let (mut hud_text, mut hud_font) = hud_panel.into_inner();
    hud_font.font_size = (graphics_settings.hud_font_size() - 3.0).max(12.0);

    let total = game.coverage.total_demand.max(1);
    let power_pct = (100 * game.coverage.powered / total).clamp(0, 100);
    let water_pct = (100 * game.coverage.watered / total).clamp(0, 100);
    let active_render_tier = graphics_settings.active_render_tier(*render_capability);
    let wrap_width = 26;

    let mut lines = vec![
        format!("Active tool: {}", game.selected_tool.label()),
        format!(
            "Bridge {} • PLC {} • R&D {}",
            game.bridge_profile.label(),
            game.logic_profile.label(),
            if game.advanced_unlocked {
                "unlocked".to_string()
            } else {
                format!("{}/3 points", game.research_points)
            }
        ),
        format!(
            "Services {}% power / {}% water • {} alerts",
            power_pct,
            water_pct,
            game.incidents.len()
        ),
    ];

    if let Some(tile) = game.selected_tile {
        let selected = game.tile(tile);
        lines.push(format!(
            "Selected ({}, {}) {:?}",
            tile.x, tile.y, selected.kind
        ));
        lines.push(format!(
            "Powered {} • Watered {}",
            yes_no(game.powered_at(tile)),
            yes_no(game.watered_at(tile))
        ));
        if let Some(asset_idx) = selected.asset {
            let asset = &game.assets[asset_idx];
            lines.push(format!(
                "{:?} • compliant {} • zone {:?}",
                asset.kind,
                yes_no(asset.compliant),
                asset.zone
            ));
            if let Some(profile) = asset.bridge_profile {
                lines.push(format!("Bridge profile {}", profile.label()));
            }
            if let Some(profile) = asset.logic_profile {
                lines.push(format!("PLC profile {}", profile.label()));
            }
            for line in wrap_text(&asset.note, wrap_width).into_iter().take(2) {
                lines.push(line);
            }
        }
    } else if let Some(tile) = hover_state.hovered_tile {
        lines.push(format!(
            "Hover ({}, {}) {:?}",
            tile.x,
            tile.y,
            game.tile(tile).kind
        ));
    }

    if show_live_perf {
        let fps = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|diagnostic| diagnostic.smoothed())
            .map(|value| format!("{value:.0}"))
            .unwrap_or_else(|| "n/a".to_string());
        let frame_ms = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|diagnostic| diagnostic.smoothed())
            .map(|value| format!("{value:.1}"))
            .unwrap_or_else(|| "n/a".to_string());
        lines.push(format!(
            "Perf {} FPS • {} ms • {} assets",
            fps,
            frame_ms,
            game.assets.len()
        ));
        lines.push(format!(
            "{} • {} • {}",
            active_render_tier.label(),
            render_capability.label(),
            graphics_settings
                .effective_mesh_detail(*render_capability)
                .label()
        ));
    }

    if let Some(entry) = game.log.last() {
        lines.push(String::new());
        lines.push("Latest log".to_string());
        for line in wrap_text(entry, wrap_width).into_iter().take(3) {
            lines.push(line);
        }
    }

    hud_text.0 = lines.join("\n");
}

pub fn update_action_sidebar_help_text(
    help_text: Single<&mut Text, With<ActionSidebarHelpText>>,
    asset_catalog: Res<AssetCatalog>,
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
) {
    if !graphics_settings.is_changed()
        && !render_capability.is_changed()
        && !asset_catalog.is_changed()
    {
        return;
    }

    let mut help_text = help_text.into_inner();
    let launch_mode = graphics_settings
        .active_render_tier(*render_capability)
        .label();
    let wrap_width = 26;

    let mut lines = vec![
        format!("Mode: {}", launch_mode),
        String::new(),
        "Mouse".to_string(),
        "Click time to pause.".to_string(),
        "Click speed to cycle.".to_string(),
        "Wheel zoom | MMB pan".to_string(),
        "RMB orbit + tilt".to_string(),
        "Click alerts for details.".to_string(),
        "Scroll this rail.".to_string(),
        String::new(),
        "Keys".to_string(),
        "F1 settings | Tab rail".to_string(),
        "F2/F3/F4 camera presets".to_string(),
        "WASD/arrows move".to_string(),
        "Q/E rotate".to_string(),
        "Z/X or PgUp/PgDn zoom".to_string(),
        "1-6 tools | Esc inspect".to_string(),
        String::new(),
        asset_catalog.summary_line(),
        String::new(),
    ];
    if render_capability.software_fallback {
        lines.push("Launch this laptop:".to_string());
        lines.extend(wrap_text(
            "SMARTCITY_FORCE_SOFTWARE_RENDERER=1 cargo run",
            wrap_width,
        ));
        lines.push(String::new());
        lines.push("GPU desktop: cargo run".to_string());
    } else {
        lines.push("Launch: cargo run".to_string());
        lines.push(String::new());
        lines.push("Old laptop fallback:".to_string());
        lines.extend(wrap_text(
            "SMARTCITY_FORCE_SOFTWARE_RENDERER=1 cargo run",
            wrap_width,
        ));
    }

    help_text.0 = lines.join("\n");
}

pub fn update_action_sidebar_layout(
    sidebar_root: Single<
        &mut Node,
        (
            With<ActionSidebarRoot>,
            Without<super::ActionSidebarScrollArea>,
            Without<ActionRailButton>,
        ),
    >,
    scroll_area: Single<
        &mut Node,
        (
            With<super::ActionSidebarScrollArea>,
            Without<ActionSidebarRoot>,
            Without<ActionRailButton>,
        ),
    >,
    toggle_text: Single<&mut Text, With<super::ActionSidebarToggleText>>,
    mut expanded_only: Query<&mut Visibility, With<ActionSidebarExpandedOnly>>,
    mut action_buttons: Query<
        &mut Node,
        (
            With<ActionRailButton>,
            Without<ActionSidebarRoot>,
            Without<super::ActionSidebarScrollArea>,
        ),
    >,
    mut action_labels: Query<&mut TextFont, With<ActionRailButtonText>>,
    action_sidebar_state: Res<ActionSidebarState>,
) {
    if !action_sidebar_state.is_changed() {
        return;
    }

    let collapsed = action_sidebar_state.collapsed;
    let mut sidebar_root = sidebar_root.into_inner();
    sidebar_root.width = Val::Px(if collapsed {
        ACTION_BAR_WIDTH_COLLAPSED
    } else {
        ACTION_BAR_WIDTH_EXPANDED
    });
    sidebar_root.padding = UiRect::all(Val::Px(if collapsed { 10.0 } else { 14.0 }));
    sidebar_root.row_gap = Val::Px(if collapsed { 10.0 } else { 12.0 });

    let mut scroll_area = scroll_area.into_inner();
    scroll_area.padding = UiRect::right(Val::Px(if collapsed { 0.0 } else { 4.0 }));

    let mut toggle_text = toggle_text.into_inner();
    toggle_text.0 = if collapsed {
        ">>".to_string()
    } else {
        "<<".to_string()
    };

    for mut visibility in &mut expanded_only {
        *visibility = if collapsed {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    for mut button in &mut action_buttons {
        button.height = Val::Px(if collapsed { 36.0 } else { 40.0 });
        button.padding = UiRect::axes(
            Val::Px(if collapsed { 4.0 } else { 12.0 }),
            Val::Px(if collapsed { 6.0 } else { 8.0 }),
        );
        button.justify_content = if collapsed {
            JustifyContent::Center
        } else {
            JustifyContent::FlexStart
        };
    }

    for mut font in &mut action_labels {
        font.font_size = if collapsed { 11.0 } else { 13.0 };
    }
}

pub fn update_action_rail_button_styles(
    mut action_buttons: Query<
        (
            &ActionRailButton,
            Ref<Interaction>,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    game: Res<GameState>,
) {
    let game_changed = game.is_changed();
    for (button, interaction, mut background, mut border) in &mut action_buttons {
        if !game_changed && !interaction.is_changed() {
            continue;
        }

        *background = BackgroundColor(action_button_color(button.action, *interaction, &game));
        *border = BorderColor::all(action_button_border_color(button.action, &game));
    }
}

pub fn update_action_rail_button_labels(
    mut action_labels: Query<(&ActionRailButtonText, &mut Text)>,
    game: Res<GameState>,
    action_sidebar_state: Res<ActionSidebarState>,
) {
    if !game.is_changed() && !action_sidebar_state.is_changed() {
        return;
    }

    for (label, mut text) in &mut action_labels {
        text.0 = action_button_label(label.action, &game, action_sidebar_state.collapsed);
    }
}

pub fn update_settings_panel(
    settings_panel: Single<
        (&mut Text, &mut TextFont, &mut Node, &mut Visibility),
        With<SettingsText>,
    >,
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    settings_menu: Res<SettingsMenuState>,
) {
    if !settings_menu.is_changed()
        && !graphics_settings.is_changed()
        && !render_capability.is_changed()
    {
        return;
    }

    let (mut settings_text, mut settings_font, mut settings_node, mut settings_visibility) =
        settings_panel.into_inner();
    settings_font.font_size = graphics_settings.hud_font_size();
    settings_node.width = Val::Px(graphics_settings.settings_panel_width());
    *settings_visibility = if settings_menu.open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    let selected_row = settings_menu.selected_row();
    let mut settings_lines = vec![
        "CONFIG // F1 OR ESC CLOSE".to_string(),
        format!("Hardware Capability: {}", render_capability.label()),
        format!(
            "Active Render Tier: {}",
            graphics_settings
                .active_render_tier(*render_capability)
                .label()
        ),
        String::new(),
    ];

    for row in SettingsMenuRow::ALL {
        let cursor = if row == selected_row { ">" } else { " " };
        settings_lines.push(format!(
            "{} {:<10} {}",
            cursor,
            row.label(),
            graphics_settings.row_value_label(row, *render_capability)
        ));
    }

    settings_lines.push(String::new());
    settings_lines.push("Up/Down select | Left/Right adjust".to_string());
    settings_lines.extend(wrap_text(
        "Auto and high tiers clamp to fallback on software-only machines.",
        44,
    ));
    settings_lines.extend(wrap_text(
        "Use Shipping Low to target weaker real GPUs with the mesh renderer.",
        44,
    ));

    settings_text.0 = settings_lines.join("\n");
}
