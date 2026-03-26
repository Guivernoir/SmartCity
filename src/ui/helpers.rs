use super::{
    materials::{cuboid_mesh, surface_material, RenderAssetCache, SurfaceKind},
    props::asset_prop_parts,
    ActionRailAction, ActionSidebarState, EventModalState, ACTION_BAR_TOP,
    ACTION_BAR_WIDTH_COLLAPSED, ACTION_BAR_WIDTH_EXPANDED, MINIMAP_PANEL_HEIGHT,
    MINIMAP_PANEL_WIDTH, SETTINGS_PANEL_HEIGHT, SETTINGS_PANEL_TOP, TOP_BAR_HEIGHT,
};
use crate::constants::{GRID_H, GRID_W, HUD_MARGIN, TILE_WORLD_SIZE};
use crate::game::GameState;
use crate::model::{Asset, AssetKind, IncidentKind, TileKind, Tool};
use crate::settings::{
    ActiveRenderTier, GraphicsSettings, MeshDetail, PropDensity, RenderCapability,
    SettingsMenuState,
};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

pub(super) fn action_sidebar_bounds(
    window: &Window,
    action_sidebar_state: &ActionSidebarState,
) -> (f32, f32, f32, f32) {
    let width = if action_sidebar_state.collapsed {
        ACTION_BAR_WIDTH_COLLAPSED
    } else {
        ACTION_BAR_WIDTH_EXPANDED
    };

    (
        HUD_MARGIN,
        ACTION_BAR_TOP,
        HUD_MARGIN + width,
        window.height() - HUD_MARGIN,
    )
}

pub(super) fn cursor_over_ui(
    window: &Window,
    cursor_position: Vec2,
    graphics_settings: &GraphicsSettings,
    settings_menu: &SettingsMenuState,
    event_modal: &EventModalState,
    action_sidebar_state: &ActionSidebarState,
) -> bool {
    if event_modal.selected_incident.is_some() {
        return true;
    }

    if cursor_position.y <= TOP_BAR_HEIGHT {
        return true;
    }

    let (sidebar_left, sidebar_top, sidebar_right, _) =
        action_sidebar_bounds(window, action_sidebar_state);
    if cursor_position.x >= sidebar_left
        && cursor_position.x <= sidebar_right
        && cursor_position.y >= sidebar_top
    {
        return true;
    }

    if settings_menu.open {
        let settings_left = window.width() - graphics_settings.settings_panel_width() - HUD_MARGIN;
        if cursor_position.x >= settings_left
            && cursor_position.x <= window.width() - HUD_MARGIN
            && cursor_position.y >= SETTINGS_PANEL_TOP
            && cursor_position.y <= SETTINGS_PANEL_TOP + SETTINGS_PANEL_HEIGHT
        {
            return true;
        }
    }

    let minimap_left = window.width() - MINIMAP_PANEL_WIDTH - HUD_MARGIN;
    let minimap_top = window.height() - MINIMAP_PANEL_HEIGHT - HUD_MARGIN;
    cursor_position.x >= minimap_left
        && cursor_position.x <= window.width() - HUD_MARGIN
        && cursor_position.y >= minimap_top
        && cursor_position.y <= window.height() - HUD_MARGIN
}

pub(super) fn top_control_color(interaction: Interaction, paused: bool, speed_chip: bool) -> Color {
    let (base, hovered, pressed) = if paused {
        (
            Color::srgb(0.54, 0.38, 0.12),
            Color::srgb(0.66, 0.48, 0.16),
            Color::srgb(0.76, 0.58, 0.20),
        )
    } else if speed_chip {
        (
            Color::srgb(0.08, 0.22, 0.28),
            Color::srgb(0.12, 0.28, 0.34),
            Color::srgb(0.16, 0.34, 0.40),
        )
    } else {
        (
            Color::srgb(0.08, 0.14, 0.19),
            Color::srgb(0.12, 0.20, 0.26),
            Color::srgb(0.18, 0.26, 0.32),
        )
    };

    match interaction {
        Interaction::Hovered => hovered,
        Interaction::Pressed => pressed,
        Interaction::None => base,
    }
}

pub(super) fn top_control_border_color(paused: bool) -> Color {
    if paused {
        Color::srgba(0.96, 0.82, 0.42, 0.42)
    } else {
        Color::srgba(0.42, 0.70, 0.82, 0.28)
    }
}

pub(super) fn action_button_label(
    action: ActionRailAction,
    game: &GameState,
    collapsed: bool,
) -> String {
    if collapsed {
        return match action {
            ActionRailAction::Tool(Tool::Inspect) => "IN".to_string(),
            ActionRailAction::Tool(Tool::Bridge) => "BR".to_string(),
            ActionRailAction::Tool(Tool::Sensor) => "SN".to_string(),
            ActionRailAction::Tool(Tool::Plc) => "PL".to_string(),
            ActionRailAction::Tool(Tool::Gateway) => "GW".to_string(),
            ActionRailAction::Tool(Tool::Substation) => "SS".to_string(),
            ActionRailAction::Tool(Tool::PumpStation) => "PU".to_string(),
            ActionRailAction::Regenerate => "RS".to_string(),
            ActionRailAction::CycleBridgeProfile => "BM".to_string(),
            ActionRailAction::CycleLogicProfile => "LM".to_string(),
            ActionRailAction::SpendResearch => "RD".to_string(),
        };
    }

    match action {
        ActionRailAction::Tool(tool) => match tool {
            Tool::Inspect => "Inspect [Esc]".to_string(),
            Tool::Bridge => "Bridge [1]".to_string(),
            Tool::Sensor => "Sensor [2]".to_string(),
            Tool::Plc => "PLC [3]".to_string(),
            Tool::Gateway => "Gateway [4]".to_string(),
            Tool::Substation => "Substation [5]".to_string(),
            Tool::PumpStation => "Pump [6]".to_string(),
        },
        ActionRailAction::Regenerate => "Reset city [R]".to_string(),
        ActionRailAction::CycleBridgeProfile => {
            format!("Bridge mode: {} [B]", game.bridge_profile.label())
        }
        ActionRailAction::CycleLogicProfile => {
            format!("PLC mode: {} [L]", game.logic_profile.label())
        }
        ActionRailAction::SpendResearch => {
            if game.advanced_unlocked {
                "Advanced controls ready".to_string()
            } else {
                format!("Unlock advanced ({}/3)", game.research_points)
            }
        }
    }
}

pub(super) fn action_button_color(
    action: ActionRailAction,
    interaction: Interaction,
    game: &GameState,
) -> Color {
    let selected = matches!(action, ActionRailAction::Tool(tool) if tool == game.selected_tool);
    let available_research = matches!(action, ActionRailAction::SpendResearch)
        && !game.advanced_unlocked
        && game.research_points >= 3;
    let unlocked_research =
        matches!(action, ActionRailAction::SpendResearch) && game.advanced_unlocked;

    let (base, hovered, pressed) = if selected {
        (
            Color::srgb(0.12, 0.54, 0.50),
            Color::srgb(0.18, 0.64, 0.60),
            Color::srgb(0.22, 0.74, 0.68),
        )
    } else if unlocked_research {
        (
            Color::srgb(0.18, 0.42, 0.26),
            Color::srgb(0.24, 0.52, 0.32),
            Color::srgb(0.30, 0.60, 0.38),
        )
    } else if available_research {
        (
            Color::srgb(0.44, 0.34, 0.10),
            Color::srgb(0.56, 0.44, 0.14),
            Color::srgb(0.66, 0.52, 0.18),
        )
    } else if matches!(action, ActionRailAction::Regenerate) {
        (
            Color::srgb(0.30, 0.16, 0.16),
            Color::srgb(0.40, 0.20, 0.20),
            Color::srgb(0.48, 0.26, 0.24),
        )
    } else {
        (
            Color::srgb(0.10, 0.16, 0.22),
            Color::srgb(0.14, 0.22, 0.30),
            Color::srgb(0.20, 0.28, 0.36),
        )
    };

    match interaction {
        Interaction::Hovered => hovered,
        Interaction::Pressed => pressed,
        Interaction::None => base,
    }
}

pub(super) fn action_button_border_color(action: ActionRailAction, game: &GameState) -> Color {
    if matches!(action, ActionRailAction::Tool(tool) if tool == game.selected_tool) {
        return Color::srgba(0.60, 0.94, 0.88, 0.36);
    }

    if matches!(action, ActionRailAction::SpendResearch) && game.advanced_unlocked {
        return Color::srgba(0.60, 0.88, 0.64, 0.24);
    }

    if matches!(action, ActionRailAction::SpendResearch) && game.research_points >= 3 {
        return Color::srgba(0.96, 0.86, 0.52, 0.30);
    }

    Color::srgba(0.50, 0.72, 0.80, 0.12)
}

pub(super) fn minimap_button_color(selected: bool, interaction: Interaction) -> Color {
    if selected {
        return match interaction {
            Interaction::Hovered => Color::srgb(0.20, 0.74, 0.70),
            Interaction::Pressed => Color::srgb(0.14, 0.88, 0.78),
            Interaction::None => Color::srgb(0.12, 0.60, 0.58),
        };
    }

    match interaction {
        Interaction::Hovered => Color::srgb(0.30, 0.34, 0.40),
        Interaction::Pressed => Color::srgb(0.38, 0.42, 0.50),
        Interaction::None => Color::srgba(0.16, 0.18, 0.22, 0.72),
    }
}

pub(super) fn event_badge_color(
    kind: IncidentKind,
    interaction: Interaction,
    selected: bool,
) -> Color {
    let (base, hovered, pressed, selected_color) = match kind {
        IncidentKind::Brownout => (
            Color::srgb(0.88, 0.66, 0.16),
            Color::srgb(0.94, 0.74, 0.26),
            Color::srgb(0.98, 0.82, 0.34),
            Color::srgb(0.74, 0.54, 0.12),
        ),
        IncidentKind::LowPressure => (
            Color::srgb(0.20, 0.62, 0.92),
            Color::srgb(0.32, 0.72, 0.98),
            Color::srgb(0.42, 0.80, 1.0),
            Color::srgb(0.16, 0.46, 0.72),
        ),
        IncidentKind::BridgeStress => (
            Color::srgb(0.88, 0.34, 0.22),
            Color::srgb(0.94, 0.46, 0.32),
            Color::srgb(0.98, 0.56, 0.40),
            Color::srgb(0.70, 0.26, 0.16),
        ),
        IncidentKind::OtIntrusion => (
            Color::srgb(0.72, 0.28, 0.82),
            Color::srgb(0.80, 0.42, 0.90),
            Color::srgb(0.88, 0.52, 0.96),
            Color::srgb(0.56, 0.20, 0.66),
        ),
    };

    if selected {
        return selected_color;
    }

    match interaction {
        Interaction::Hovered => hovered,
        Interaction::Pressed => pressed,
        Interaction::None => base,
    }
}

pub(super) fn incident_badge_label(kind: IncidentKind) -> &'static str {
    match kind {
        IncidentKind::Brownout => "PWR",
        IncidentKind::LowPressure => "H2O",
        IncidentKind::BridgeStress => "BRG",
        IncidentKind::OtIntrusion => "OT",
    }
}

pub(super) fn incident_title(kind: IncidentKind) -> &'static str {
    match kind {
        IncidentKind::Brownout => "Brownout Risk",
        IncidentKind::LowPressure => "Low Water Pressure",
        IncidentKind::BridgeStress => "Bridge Stress Alert",
        IncidentKind::OtIntrusion => "OT Intrusion Risk",
    }
}

pub(super) fn incident_action_copy(kind: IncidentKind) -> &'static str {
    match kind {
        IncidentKind::Brownout => {
            "Expand substation coverage and reduce unsupported electrical demand."
        }
        IncidentKind::LowPressure => {
            "Improve pump-station placement near water access and restore distribution."
        }
        IncidentKind::BridgeStress => {
            "Replace weak bridge builds with compliant profiles before trust collapses."
        }
        IncidentKind::OtIntrusion => {
            "Add compliant gateways and harden PLC logic before the city notices."
        }
    }
}

pub(super) fn minimap_tile_color(
    mode: super::MinimapMode,
    game: &GameState,
    pos: IVec2,
    hovered_tile: Option<IVec2>,
) -> Color {
    if Some(pos) == game.selected_tile {
        return Color::srgb(0.18, 1.0, 0.40);
    }

    if Some(pos) == hovered_tile {
        return Color::srgb(1.0, 0.95, 0.22);
    }

    let tile = game.tile(pos);
    match mode {
        super::MinimapMode::Real => minimap_real_color(tile.kind),
        super::MinimapMode::Services => match tile.kind {
            TileKind::River => Color::srgb(0.18, 0.44, 0.88),
            TileKind::Road | TileKind::Empty | TileKind::Park => minimap_real_color(tile.kind),
            _ => match (game.powered_at(pos), game.watered_at(pos)) {
                (true, true) => Color::srgb(0.12, 0.74, 0.42),
                (true, false) => Color::srgb(0.84, 0.62, 0.18),
                (false, true) => Color::srgb(0.18, 0.62, 0.90),
                (false, false) => Color::srgb(0.82, 0.22, 0.18),
            },
        },
        super::MinimapMode::Electrical => match tile.kind {
            TileKind::Road | TileKind::River | TileKind::Empty | TileKind::Park => {
                minimap_real_color(tile.kind)
            }
            _ => {
                if game.powered_at(pos) {
                    Color::srgb(0.96, 0.84, 0.18)
                } else {
                    Color::srgb(0.58, 0.16, 0.16)
                }
            }
        },
        super::MinimapMode::Water => match tile.kind {
            TileKind::Road | TileKind::River | TileKind::Empty | TileKind::Park => {
                minimap_real_color(tile.kind)
            }
            _ => {
                if game.watered_at(pos) {
                    Color::srgb(0.18, 0.62, 0.96)
                } else {
                    Color::srgb(0.58, 0.16, 0.16)
                }
            }
        },
        super::MinimapMode::Risk => {
            if let Some(asset_idx) = tile.asset {
                let asset = &game.assets[asset_idx];
                if asset.compliant {
                    Color::srgb(0.16, 0.70, 0.42)
                } else {
                    Color::srgb(0.94, 0.18, 0.18)
                }
            } else {
                match tile.kind {
                    TileKind::Industrial | TileKind::Utility => Color::srgb(0.70, 0.42, 0.18),
                    TileKind::ControlCenter => Color::srgb(0.54, 0.40, 0.72),
                    _ => Color::srgb(0.16, 0.18, 0.22),
                }
            }
        }
    }
}

pub(super) fn minimap_real_color(tile_kind: TileKind) -> Color {
    match tile_kind {
        TileKind::Empty => Color::srgb(0.20, 0.22, 0.25),
        TileKind::Road => Color::srgb(0.30, 0.31, 0.34),
        TileKind::Building => Color::srgb(0.46, 0.62, 0.78),
        TileKind::Industrial => Color::srgb(0.70, 0.50, 0.24),
        TileKind::ControlCenter => Color::srgb(0.56, 0.42, 0.76),
        TileKind::River => Color::srgb(0.14, 0.40, 0.84),
        TileKind::Park => Color::srgb(0.20, 0.52, 0.24),
        TileKind::Utility => Color::srgb(0.46, 0.44, 0.62),
    }
}

pub(super) fn spawn_tile_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    render_assets: &mut RenderAssetCache,
    graphics_settings: &GraphicsSettings,
    render_capability: RenderCapability,
    game: &GameState,
    pos: IVec2,
    tile_kind: TileKind,
) {
    let render_tier = graphics_settings.active_render_tier(render_capability);
    let mesh_detail = graphics_settings.effective_mesh_detail(render_capability);
    let prop_density = graphics_settings.effective_prop_density(render_capability);
    for part in
        super::blockout::tile_blockout_parts(game, pos, tile_kind, mesh_detail, prop_density)
    {
        spawn_visual_box(
            commands,
            meshes,
            materials,
            render_assets,
            part.size,
            part.color,
            part.surface,
            render_tier,
            tile_to_world(pos, 0.0) + part.offset,
        );
    }
}

pub(super) fn spawn_asset_visual(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    render_assets: &mut RenderAssetCache,
    graphics_settings: &GraphicsSettings,
    render_capability: RenderCapability,
    asset: &Asset,
    tile_kind: TileKind,
) {
    let render_tier = graphics_settings.active_render_tier(render_capability);
    let mesh_detail = graphics_settings.effective_mesh_detail(render_capability);
    let prop_density = graphics_settings.effective_prop_density(render_capability);
    let (size, color) = asset_style(asset.kind);
    let asset_y = tile_top_y(tile_kind) + size.y * 0.5 + 0.05;

    spawn_visual_box(
        commands,
        meshes,
        materials,
        render_assets,
        size,
        color,
        asset_surface_kind(asset.kind),
        render_tier,
        tile_to_world(asset.pos, asset_y),
    );

    if !matches!(mesh_detail, MeshDetail::Low) {
        let cap_size = Vec3::new((size.x * 0.55).max(0.18), 0.12, (size.z * 0.55).max(0.18));
        spawn_visual_box(
            commands,
            meshes,
            materials,
            render_assets,
            cap_size,
            asset_accent_color(asset.kind),
            asset_accent_surface_kind(asset.kind),
            render_tier,
            tile_to_world(asset.pos, asset_y + size.y * 0.5 + cap_size.y * 0.5 + 0.03),
        );
    }

    if matches!(mesh_detail, MeshDetail::High)
        && matches!(asset.kind, AssetKind::Sensor | AssetKind::Gateway)
    {
        spawn_visual_box(
            commands,
            meshes,
            materials,
            render_assets,
            Vec3::new(0.12, 0.9, 0.12),
            Color::srgb(0.92, 0.96, 1.0),
            SurfaceKind::Glass,
            render_tier,
            tile_to_world(asset.pos, asset_y + size.y * 0.5 + 0.45),
        );
    }

    for part in asset_prop_parts(asset, size, mesh_detail, prop_density) {
        spawn_visual_box(
            commands,
            meshes,
            materials,
            render_assets,
            part.size,
            part.color,
            part.surface,
            render_tier,
            tile_to_world(asset.pos, asset_y) + part.offset,
        );
    }

    if !asset.compliant && !matches!(prop_density, PropDensity::Off) {
        spawn_visual_box(
            commands,
            meshes,
            materials,
            render_assets,
            Vec3::new(0.28, 0.9, 0.28),
            Color::srgb(0.92, 0.1, 0.12),
            SurfaceKind::LotPaint,
            render_tier,
            tile_to_world(asset.pos, asset_y + size.y * 0.5 + 0.45),
        );
    }
}

fn spawn_visual_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    render_assets: &mut RenderAssetCache,
    size: Vec3,
    color: Color,
    surface: SurfaceKind,
    render_tier: ActiveRenderTier,
    translation: Vec3,
) {
    let mesh = cuboid_mesh(render_assets, meshes, size);
    let material = surface_material(render_assets, materials, surface, color, render_tier);
    let casts_shadows = matches!(render_tier, ActiveRenderTier::Full3d)
        && size.y >= 0.18
        && !matches!(
            surface,
            SurfaceKind::Water | SurfaceKind::RoadPaint | SurfaceKind::LotPaint | SurfaceKind::Wear
        );

    let mut entity = commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(translation),
        super::CityVisual,
    ));

    if !casts_shadows {
        entity.insert(NotShadowCaster);
    }
}

fn asset_accent_color(asset_kind: AssetKind) -> Color {
    match asset_kind {
        AssetKind::Bridge => Color::srgb(0.94, 0.86, 0.44),
        AssetKind::Sensor => Color::srgb(0.72, 0.92, 0.98),
        AssetKind::Plc => Color::srgb(0.98, 0.72, 0.38),
        AssetKind::Gateway => Color::srgb(0.96, 0.62, 0.84),
        AssetKind::Substation => Color::srgb(0.72, 0.94, 0.48),
        AssetKind::PumpStation => Color::srgb(0.52, 0.74, 0.98),
    }
}

fn asset_surface_kind(asset_kind: AssetKind) -> SurfaceKind {
    match asset_kind {
        AssetKind::Bridge => SurfaceKind::PaintedMetal,
        AssetKind::Sensor => SurfaceKind::PaintedMetal,
        AssetKind::Plc => SurfaceKind::PaintedMetal,
        AssetKind::Gateway => SurfaceKind::PaintedMetal,
        AssetKind::Substation => SurfaceKind::Concrete,
        AssetKind::PumpStation => SurfaceKind::Concrete,
    }
}

fn asset_accent_surface_kind(asset_kind: AssetKind) -> SurfaceKind {
    match asset_kind {
        AssetKind::Sensor | AssetKind::Gateway => SurfaceKind::Glass,
        AssetKind::Bridge => SurfaceKind::PaintedMetal,
        AssetKind::Plc => SurfaceKind::PaintedMetal,
        AssetKind::Substation => SurfaceKind::LotPaint,
        AssetKind::PumpStation => SurfaceKind::LotPaint,
    }
}

pub(super) fn tile_style(tile_kind: TileKind) -> (f32, f32, Color) {
    match tile_kind {
        TileKind::Empty => (0.12, 0.06, Color::srgb(0.27, 0.31, 0.30)),
        TileKind::Road => (0.07, 0.035, Color::srgb(0.18, 0.20, 0.22)),
        TileKind::Building => (3.8, 1.9, Color::srgb(0.48, 0.62, 0.74)),
        TileKind::Industrial => (2.9, 1.45, Color::srgb(0.62, 0.48, 0.30)),
        TileKind::ControlCenter => (5.8, 2.9, Color::srgb(0.54, 0.40, 0.72)),
        TileKind::River => (0.20, -0.44, Color::srgb(0.12, 0.34, 0.66)),
        TileKind::Park => (0.10, 0.05, Color::srgb(0.24, 0.52, 0.24)),
        TileKind::Utility => (2.1, 1.05, Color::srgb(0.45, 0.42, 0.60)),
    }
}

pub(super) fn tile_top_y(tile_kind: TileKind) -> f32 {
    let (height, y, _) = tile_style(tile_kind);
    y + height * 0.5
}

pub(super) fn asset_style(asset_kind: AssetKind) -> (Vec3, Color) {
    match asset_kind {
        AssetKind::Bridge => (
            Vec3::new(TILE_WORLD_SIZE * 1.55, 0.24, TILE_WORLD_SIZE * 0.55),
            Color::srgb(0.82, 0.72, 0.22),
        ),
        AssetKind::Sensor => (Vec3::new(0.24, 0.95, 0.24), Color::srgb(0.36, 0.82, 0.92)),
        AssetKind::Plc => (Vec3::new(0.62, 0.62, 0.62), Color::srgb(0.92, 0.52, 0.22)),
        AssetKind::Gateway => (Vec3::new(0.34, 1.35, 0.34), Color::srgb(0.90, 0.42, 0.68)),
        AssetKind::Substation => (Vec3::new(1.2, 0.44, 1.2), Color::srgb(0.52, 0.86, 0.28)),
        AssetKind::PumpStation => (Vec3::new(0.95, 0.8, 0.95), Color::srgb(0.28, 0.56, 0.92)),
    }
}

pub(super) fn tile_to_world(tile: IVec2, y: f32) -> Vec3 {
    let origin_x = -((GRID_W - 1) as f32 * TILE_WORLD_SIZE) * 0.5;
    let origin_z = -((GRID_H - 1) as f32 * TILE_WORLD_SIZE) * 0.5;
    Vec3::new(
        origin_x + tile.x as f32 * TILE_WORLD_SIZE,
        y,
        origin_z + tile.y as f32 * TILE_WORLD_SIZE,
    )
}

pub(super) fn world_to_tile(world: Vec3) -> Option<IVec2> {
    let origin_x = -((GRID_W - 1) as f32 * TILE_WORLD_SIZE) * 0.5;
    let origin_z = -((GRID_H - 1) as f32 * TILE_WORLD_SIZE) * 0.5;
    let x = ((world.x - origin_x) / TILE_WORLD_SIZE).round() as i32;
    let y = ((world.z - origin_z) / TILE_WORLD_SIZE).round() as i32;

    if (0..GRID_W).contains(&x) && (0..GRID_H).contains(&y) {
        Some(IVec2::new(x, y))
    } else {
        None
    }
}

pub(super) fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if word.chars().count() > max_chars {
            if !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }

            let mut chunk = String::new();
            let mut chunk_len = 0;
            for ch in word.chars() {
                if chunk_len >= max_chars {
                    lines.push(chunk.clone());
                    chunk.clear();
                    chunk_len = 0;
                }
                chunk.push(ch);
                chunk_len += 1;
            }
            if !chunk.is_empty() {
                lines.push(chunk);
            }
            continue;
        }

        let next_len = if current.is_empty() {
            word.chars().count()
        } else {
            current.chars().count() + 1 + word.chars().count()
        };

        if next_len > max_chars && !current.is_empty() {
            lines.push(current.clone());
            current.clear();
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

pub(super) fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}
