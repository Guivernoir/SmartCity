use super::districts::{
    building_palette, building_style, district_theme, BuildingDensity, BuildingPalette,
    BuildingStyle, DistrictTheme,
};
use super::helpers::tile_to_world;
use super::materials::{surface_material, RenderAssetCache, SurfaceKind};
use super::props::{
    append_building_props, append_control_center_props, append_empty_lot_props,
    append_industrial_props, append_park_props, append_road_props, append_utility_props,
};
use super::CityVisual;
use crate::constants::{GRID_H, GRID_W, TILE_WORLD_SIZE};
use crate::game::GameState;
use crate::model::TileKind;
use crate::settings::{ActiveRenderTier, MeshDetail, PropDensity};
use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

const NORTH: u8 = 1;
const EAST: u8 = 2;
const SOUTH: u8 = 4;
const WEST: u8 = 8;

#[derive(Clone, Copy, Debug)]
pub(super) struct BlockoutPart {
    pub size: Vec3,
    pub offset: Vec3,
    pub color: Color,
    pub surface: SurfaceKind,
}

pub(super) fn spawn_city_terrain(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    render_assets: &mut RenderAssetCache,
    game: &GameState,
    render_tier: ActiveRenderTier,
) {
    commands.spawn((
        Mesh3d(meshes.add(build_terrain_mesh(game))),
        MeshMaterial3d(surface_material(
            render_assets,
            materials,
            SurfaceKind::Vegetation,
            Color::srgb(0.18, 0.22, 0.16),
            render_tier,
        )),
        Transform::IDENTITY,
        NotShadowCaster,
        CityVisual,
    ));
}

pub(super) fn tile_blockout_parts(
    game: &GameState,
    pos: IVec2,
    tile_kind: TileKind,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) -> Vec<BlockoutPart> {
    let mut parts = Vec::with_capacity(12);
    match tile_kind {
        TileKind::Road => push_road_parts(&mut parts, game, pos, mesh_detail, prop_density),
        TileKind::River => push_river_parts(&mut parts, game, pos),
        TileKind::Building => push_building_parts(&mut parts, game, pos, mesh_detail, prop_density),
        TileKind::Industrial => {
            push_industrial_parts(&mut parts, game, pos, mesh_detail, prop_density)
        }
        TileKind::ControlCenter => {
            push_control_center_parts(&mut parts, game, pos, mesh_detail, prop_density)
        }
        TileKind::Utility => push_utility_parts(&mut parts, game, pos, mesh_detail, prop_density),
        TileKind::Park => push_park_parts(&mut parts, game, pos, prop_density),
        TileKind::Empty => push_empty_lot_parts(&mut parts, game, pos, mesh_detail, prop_density),
    }
    parts
}

fn push_road_parts(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    let mask = road_mask(game, pos);
    let sidewalk_size = TILE_WORLD_SIZE * 0.96;
    let road_width = TILE_WORLD_SIZE * 0.50;
    let curb_width = TILE_WORLD_SIZE * 0.08;
    let asphalt_h = 0.045;
    let sidewalk_h = 0.08;

    parts.push(part(
        Vec3::new(sidewalk_size, sidewalk_h, sidewalk_size),
        Vec3::new(0.0, sidewalk_h * 0.5 - 0.01, 0.0),
        Color::srgb(0.47, 0.48, 0.50),
        SurfaceKind::Concrete,
    ));

    for (edge_bit, offset, horizontal) in [
        (NORTH, Vec3::new(0.0, 0.09, -TILE_WORLD_SIZE * 0.44), true),
        (SOUTH, Vec3::new(0.0, 0.09, TILE_WORLD_SIZE * 0.44), true),
        (EAST, Vec3::new(TILE_WORLD_SIZE * 0.44, 0.09, 0.0), false),
        (WEST, Vec3::new(-TILE_WORLD_SIZE * 0.44, 0.09, 0.0), false),
    ] {
        let size = if horizontal {
            Vec3::new(sidewalk_size, 0.026, curb_width)
        } else {
            Vec3::new(curb_width, 0.026, sidewalk_size)
        };
        let curb_color = if mask & edge_bit != 0 {
            Color::srgb(0.55, 0.56, 0.58)
        } else {
            Color::srgb(0.62, 0.63, 0.66)
        };
        parts.push(part(size, offset, curb_color, SurfaceKind::Concrete));
    }

    parts.push(part(
        Vec3::new(road_width, asphalt_h, road_width),
        Vec3::new(0.0, 0.088, 0.0),
        Color::srgb(0.12, 0.13, 0.15),
        SurfaceKind::Asphalt,
    ));

    let arm_len = TILE_WORLD_SIZE * 0.5 - road_width * 0.5 - 0.08;
    let arm_offset = road_width * 0.25 + arm_len * 0.5;
    if mask & NORTH != 0 {
        parts.push(part(
            Vec3::new(road_width, asphalt_h, arm_len),
            Vec3::new(0.0, 0.088, -arm_offset),
            Color::srgb(0.12, 0.13, 0.15),
            SurfaceKind::Asphalt,
        ));
    }
    if mask & SOUTH != 0 {
        parts.push(part(
            Vec3::new(road_width, asphalt_h, arm_len),
            Vec3::new(0.0, 0.088, arm_offset),
            Color::srgb(0.12, 0.13, 0.15),
            SurfaceKind::Asphalt,
        ));
    }
    if mask & EAST != 0 {
        parts.push(part(
            Vec3::new(arm_len, asphalt_h, road_width),
            Vec3::new(arm_offset, 0.088, 0.0),
            Color::srgb(0.12, 0.13, 0.15),
            SurfaceKind::Asphalt,
        ));
    }
    if mask & WEST != 0 {
        parts.push(part(
            Vec3::new(arm_len, asphalt_h, road_width),
            Vec3::new(-arm_offset, 0.088, 0.0),
            Color::srgb(0.12, 0.13, 0.15),
            SurfaceKind::Asphalt,
        ));
    }

    if !matches!(mesh_detail, MeshDetail::Low) {
        push_lane_markings(parts, mask, road_width);
    }

    if matches!(mesh_detail, MeshDetail::High) && mask.count_ones() >= 2 {
        parts.push(part(
            Vec3::new(road_width * 0.82, 0.01, road_width * 0.24),
            Vec3::new(0.16, 0.115, -0.06),
            Color::srgb(0.18, 0.17, 0.16),
            SurfaceKind::Wear,
        ));
    }

    append_road_props(parts, game, pos, mesh_detail, prop_density);
}

fn push_lane_markings(parts: &mut Vec<BlockoutPart>, mask: u8, road_width: f32) {
    let stripe_h = 0.012;
    let stripe_color = Color::srgb(0.92, 0.84, 0.34);
    let straight_z = mask & (NORTH | SOUTH) == (NORTH | SOUTH);
    let straight_x = mask & (EAST | WEST) == (EAST | WEST);

    if straight_z {
        parts.push(part(
            Vec3::new(0.08, stripe_h, TILE_WORLD_SIZE * 0.82),
            Vec3::new(0.0, 0.117, 0.0),
            stripe_color,
            SurfaceKind::RoadPaint,
        ));
    }
    if straight_x {
        parts.push(part(
            Vec3::new(TILE_WORLD_SIZE * 0.82, stripe_h, 0.08),
            Vec3::new(0.0, 0.117, 0.0),
            stripe_color,
            SurfaceKind::RoadPaint,
        ));
    }
    if !straight_x && !straight_z && mask.count_ones() >= 3 {
        parts.push(part(
            Vec3::new(road_width * 0.32, stripe_h, road_width * 0.32),
            Vec3::new(0.0, 0.117, 0.0),
            stripe_color,
            SurfaceKind::RoadPaint,
        ));
    }
}

fn push_river_parts(parts: &mut Vec<BlockoutPart>, game: &GameState, pos: IVec2) {
    let boundary_mask = lot_boundary_mask(game, pos);
    for (edge_bit, offset, horizontal) in [
        (NORTH, Vec3::new(0.0, -0.08, -TILE_WORLD_SIZE * 0.40), true),
        (SOUTH, Vec3::new(0.0, -0.08, TILE_WORLD_SIZE * 0.40), true),
        (EAST, Vec3::new(TILE_WORLD_SIZE * 0.40, -0.08, 0.0), false),
        (WEST, Vec3::new(-TILE_WORLD_SIZE * 0.40, -0.08, 0.0), false),
    ] {
        if boundary_mask & edge_bit == 0 {
            continue;
        }
        let size = if horizontal {
            Vec3::new(TILE_WORLD_SIZE * 0.88, 0.10, TILE_WORLD_SIZE * 0.14)
        } else {
            Vec3::new(TILE_WORLD_SIZE * 0.14, 0.10, TILE_WORLD_SIZE * 0.88)
        };
        parts.push(part(
            size,
            offset,
            Color::srgb(0.42, 0.34, 0.22),
            SurfaceKind::Concrete,
        ));
    }

    parts.push(part(
        Vec3::new(TILE_WORLD_SIZE * 0.86, 0.03, TILE_WORLD_SIZE * 0.86),
        Vec3::new(0.0, -0.28, 0.0),
        Color::srgb(0.24, 0.58, 0.92),
        SurfaceKind::Water,
    ));
    parts.push(part(
        Vec3::new(TILE_WORLD_SIZE * 0.60, 0.18, TILE_WORLD_SIZE * 0.60),
        Vec3::new(0.0, -0.42, 0.0),
        Color::srgb(0.10, 0.24, 0.44),
        SurfaceKind::Water,
    ));
}

fn push_empty_lot_parts(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    let theme = district_theme(game, pos);
    let (lot_color, surface) = match theme {
        DistrictTheme::Garden => (Color::srgb(0.26, 0.38, 0.24), SurfaceKind::Vegetation),
        DistrictTheme::Riverside => (Color::srgb(0.52, 0.48, 0.40), SurfaceKind::Concrete),
        DistrictTheme::Innovation => (Color::srgb(0.36, 0.42, 0.46), SurfaceKind::Concrete),
        DistrictTheme::Logistics => (Color::srgb(0.40, 0.34, 0.24), SurfaceKind::Concrete),
        DistrictTheme::Core => (Color::srgb(0.32, 0.35, 0.30), SurfaceKind::Concrete),
    };
    push_lot_base(parts, game, pos, lot_color, surface);

    match theme {
        DistrictTheme::Garden => {
            parts.push(part(
                Vec3::new(0.42, 0.02, 0.28),
                Vec3::new(-0.20, 0.07, -0.18),
                Color::srgb(0.32, 0.54, 0.30),
                SurfaceKind::Vegetation,
            ));
            parts.push(part(
                Vec3::new(0.28, 0.02, 0.22),
                Vec3::new(0.26, 0.07, 0.24),
                Color::srgb(0.26, 0.46, 0.26),
                SurfaceKind::Vegetation,
            ));
        }
        DistrictTheme::Riverside => {
            parts.push(part(
                Vec3::new(TILE_WORLD_SIZE * 0.72, 0.02, 0.14),
                Vec3::new(0.0, 0.07, 0.28),
                Color::srgb(0.80, 0.78, 0.70),
                SurfaceKind::Concrete,
            ));
        }
        DistrictTheme::Innovation => {
            parts.push(part(
                Vec3::new(0.76, 0.012, 0.10),
                Vec3::new(0.0, 0.07, -0.20),
                Color::srgb(0.76, 0.90, 0.98),
                SurfaceKind::LotPaint,
            ));
        }
        DistrictTheme::Logistics => {
            if matches!(mesh_detail, MeshDetail::High) {
                parts.push(part(
                    Vec3::new(0.86, 0.012, 0.42),
                    Vec3::new(0.0, 0.068, 0.0),
                    Color::srgb(0.18, 0.16, 0.14),
                    SurfaceKind::Wear,
                ));
            }
        }
        DistrictTheme::Core => {}
    }

    if matches!(mesh_detail, MeshDetail::High) && road_adjacency_count(game, pos) >= 1 {
        parts.push(part(
            Vec3::new(TILE_WORLD_SIZE * 0.52, 0.02, 0.08),
            Vec3::new(0.0, 0.064, 0.0),
            Color::srgb(0.70, 0.72, 0.74),
            SurfaceKind::LotPaint,
        ));
    }

    append_empty_lot_props(parts, game, pos, prop_density);
}

fn push_building_parts(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    let style = building_style(game, pos);
    let palette = building_palette(style);
    let lot_surface = if matches!(style.theme, DistrictTheme::Garden)
        && matches!(style.density, BuildingDensity::Low)
    {
        SurfaceKind::Vegetation
    } else {
        SurfaceKind::Concrete
    };

    push_lot_base(parts, game, pos, palette.lot, lot_surface);
    push_building_lot_features(parts, &style, &palette, mesh_detail);

    let roof_y = match style.density {
        BuildingDensity::Low => push_low_density_building(parts, &style, &palette, mesh_detail),
        BuildingDensity::Mid => push_mid_density_building(parts, &style, &palette, mesh_detail),
        BuildingDensity::High => push_high_density_building(parts, &style, &palette, mesh_detail),
        BuildingDensity::Tower => push_tower_building(parts, &style, &palette, mesh_detail),
    };

    push_facade_accents(parts, &style, &palette, roof_y, mesh_detail);
    append_building_props(
        parts,
        game,
        pos,
        mesh_detail,
        prop_density,
        roof_y,
        style.density.is_dense(),
    );
}

fn push_industrial_parts(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    let theme = district_theme(game, pos);
    let (lot_color, shell_color, annex_color, stack_color) = match theme {
        DistrictTheme::Innovation => (
            Color::srgb(0.38, 0.38, 0.40),
            Color::srgb(0.56, 0.60, 0.66),
            Color::srgb(0.46, 0.54, 0.60),
            Color::srgb(0.30, 0.36, 0.42),
        ),
        DistrictTheme::Riverside => (
            Color::srgb(0.48, 0.40, 0.28),
            Color::srgb(0.66, 0.56, 0.40),
            Color::srgb(0.58, 0.50, 0.38),
            Color::srgb(0.34, 0.28, 0.22),
        ),
        _ => (
            Color::srgb(0.44, 0.36, 0.24),
            Color::srgb(0.60, 0.48, 0.32),
            Color::srgb(0.54, 0.42, 0.28),
            Color::srgb(0.30, 0.24, 0.20),
        ),
    };
    push_lot_base(parts, game, pos, lot_color, SurfaceKind::Concrete);
    parts.push(part(
        Vec3::new(1.92, 1.18, 1.42),
        Vec3::new(0.0, 0.68, -0.06),
        shell_color,
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.76, 0.96, 0.66),
        Vec3::new(-0.54, 0.57, 0.52),
        annex_color,
        SurfaceKind::Brick,
    ));
    if !matches!(mesh_detail, MeshDetail::Low) {
        parts.push(part(
            Vec3::new(0.46, 0.78, 0.46),
            Vec3::new(0.60, 0.48, 0.42),
            Color::srgb(0.72, 0.58, 0.36),
            SurfaceKind::PaintedMetal,
        ));
        parts.push(part(
            Vec3::new(0.18, 2.20, 0.18),
            Vec3::new(0.68, 1.30, -0.48),
            stack_color,
            SurfaceKind::Concrete,
        ));
        parts.push(part(
            Vec3::new(1.04, 0.012, 0.42),
            Vec3::new(0.18, 0.096, -0.24),
            Color::srgb(0.18, 0.16, 0.14),
            SurfaceKind::Wear,
        ));
        if matches!(theme, DistrictTheme::Innovation) {
            parts.push(part(
                Vec3::new(0.58, 0.56, 0.32),
                Vec3::new(-0.42, 0.36, -0.44),
                Color::srgb(0.64, 0.84, 0.92),
                SurfaceKind::Glass,
            ));
        }
        if matches!(theme, DistrictTheme::Riverside) {
            parts.push(part(
                Vec3::new(1.24, 0.12, 0.18),
                Vec3::new(0.0, 0.14, 0.34),
                Color::srgb(0.74, 0.72, 0.66),
                SurfaceKind::Concrete,
            ));
        }
    }

    append_industrial_props(parts, game, pos, mesh_detail, prop_density);
}

fn push_utility_parts(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    let theme = district_theme(game, pos);
    let (lot_color, shell_color, module_color, accent_color) = match theme {
        DistrictTheme::Innovation => (
            Color::srgb(0.32, 0.38, 0.44),
            Color::srgb(0.54, 0.62, 0.70),
            Color::srgb(0.72, 0.86, 0.56),
            Color::srgb(0.76, 0.92, 0.98),
        ),
        DistrictTheme::Garden => (
            Color::srgb(0.30, 0.36, 0.34),
            Color::srgb(0.52, 0.56, 0.58),
            Color::srgb(0.68, 0.78, 0.48),
            Color::srgb(0.86, 0.94, 0.82),
        ),
        _ => (
            Color::srgb(0.34, 0.36, 0.42),
            Color::srgb(0.50, 0.52, 0.62),
            Color::srgb(0.64, 0.72, 0.42),
            Color::srgb(0.86, 0.90, 0.96),
        ),
    };
    push_lot_base(parts, game, pos, lot_color, SurfaceKind::Concrete);
    parts.push(part(
        Vec3::new(1.46, 1.12, 1.12),
        Vec3::new(0.0, 0.64, 0.0),
        shell_color,
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.42, 0.72, 0.42),
        Vec3::new(-0.54, 0.44, 0.46),
        module_color,
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.42, 0.72, 0.42),
        Vec3::new(0.56, 0.44, 0.40),
        module_color,
        SurfaceKind::PaintedMetal,
    ));
    if !matches!(mesh_detail, MeshDetail::Low) {
        parts.push(part(
            Vec3::new(0.96, 0.012, 0.12),
            Vec3::new(0.0, 0.094, -0.20),
            Color::srgb(0.78, 0.84, 0.92),
            SurfaceKind::LotPaint,
        ));
    }
    if matches!(mesh_detail, MeshDetail::High) {
        parts.push(part(
            Vec3::new(0.16, 1.18, 0.16),
            Vec3::new(0.0, 1.26, -0.52),
            accent_color,
            SurfaceKind::Glass,
        ));
        if matches!(theme, DistrictTheme::Innovation) {
            parts.push(part(
                Vec3::new(0.78, 0.08, 0.08),
                Vec3::new(0.0, 1.10, -0.10),
                Color::srgb(0.78, 0.92, 0.98),
                SurfaceKind::Glass,
            ));
        }
    }

    append_utility_props(parts, game, pos, mesh_detail, prop_density);
}

fn push_control_center_parts(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    push_lot_base(
        parts,
        game,
        pos,
        Color::srgb(0.34, 0.30, 0.42),
        SurfaceKind::Concrete,
    );
    parts.push(part(
        Vec3::new(1.96, 0.42, 1.68),
        Vec3::new(0.0, 0.24, 0.0),
        Color::srgb(0.46, 0.42, 0.60),
        SurfaceKind::Concrete,
    ));
    parts.push(part(
        Vec3::new(0.94, 4.20, 0.94),
        Vec3::new(-0.10, 2.50, -0.10),
        Color::srgb(0.58, 0.52, 0.78),
        SurfaceKind::Glass,
    ));
    parts.push(part(
        Vec3::new(0.90, 1.52, 0.74),
        Vec3::new(0.56, 0.80, 0.34),
        Color::srgb(0.52, 0.48, 0.68),
        SurfaceKind::PaintedMetal,
    ));
    if !matches!(mesh_detail, MeshDetail::Low) {
        parts.push(part(
            Vec3::new(0.16, 0.96, 0.16),
            Vec3::new(-0.10, 5.06, -0.10),
            Color::srgb(0.88, 0.94, 1.0),
            SurfaceKind::PaintedMetal,
        ));
    }

    append_control_center_props(parts, game, pos, mesh_detail, prop_density);
}

fn push_park_parts(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    prop_density: PropDensity,
) {
    push_lot_base(
        parts,
        game,
        pos,
        Color::srgb(0.22, 0.46, 0.22),
        SurfaceKind::Vegetation,
    );
    parts.push(part(
        Vec3::new(TILE_WORLD_SIZE * 0.70, 0.02, 0.18),
        Vec3::new(0.0, 0.05, 0.0),
        Color::srgb(0.66, 0.58, 0.38),
        SurfaceKind::Concrete,
    ));
    parts.push(part(
        Vec3::new(0.18, 0.02, TILE_WORLD_SIZE * 0.66),
        Vec3::new(0.0, 0.05, 0.0),
        Color::srgb(0.66, 0.58, 0.38),
        SurfaceKind::Concrete,
    ));

    let tree_count = match prop_density {
        PropDensity::Off => 0,
        PropDensity::Low => 1,
        PropDensity::Medium => 2,
        PropDensity::High => 3,
    };
    for offset in [
        Vec3::new(0.44, 0.0, -0.38),
        Vec3::new(-0.34, 0.0, 0.30),
        Vec3::new(0.06, 0.0, 0.46),
    ]
    .into_iter()
    .take(tree_count)
    {
        parts.push(part(
            Vec3::new(0.24, 0.78, 0.24),
            Vec3::new(offset.x, 0.42, offset.z),
            Color::srgb(0.12, 0.36, 0.14),
            SurfaceKind::Vegetation,
        ));
    }

    append_park_props(parts, game, pos, prop_density);
}

fn push_building_lot_features(
    parts: &mut Vec<BlockoutPart>,
    style: &BuildingStyle,
    palette: &BuildingPalette,
    mesh_detail: MeshDetail,
) {
    if style.near_park || matches!(style.theme, DistrictTheme::Garden) {
        let grass_offset = if style.seed & 1 == 0 {
            Vec3::new(-0.26, 0.07, 0.28)
        } else {
            Vec3::new(0.26, 0.07, -0.28)
        };
        parts.push(part(
            Vec3::new(0.30, 0.02, 0.24),
            grass_offset,
            palette.courtyard,
            SurfaceKind::Vegetation,
        ));
    }

    if style.near_river {
        parts.push(part(
            Vec3::new(TILE_WORLD_SIZE * 0.64, 0.018, 0.12),
            Vec3::new(0.0, 0.068, 0.30),
            Color::srgb(0.84, 0.82, 0.76),
            SurfaceKind::Concrete,
        ));
    }

    if matches!(style.theme, DistrictTheme::Innovation) {
        parts.push(part(
            Vec3::new(0.66, 0.012, 0.10),
            Vec3::new(0.0, 0.068, -0.22),
            palette.accent,
            SurfaceKind::LotPaint,
        ));
    }

    if style.near_industrial && matches!(mesh_detail, MeshDetail::High) {
        parts.push(part(
            Vec3::new(0.42, 0.012, 0.22),
            Vec3::new(0.28, 0.068, 0.18),
            Color::srgb(0.16, 0.14, 0.12),
            SurfaceKind::Wear,
        ));
    }
}

fn push_low_density_building(
    parts: &mut Vec<BlockoutPart>,
    style: &BuildingStyle,
    palette: &BuildingPalette,
    mesh_detail: MeshDetail,
) -> f32 {
    match style.seed % 3 {
        0 => {
            parts.push(part(
                Vec3::new(0.44, 1.28, 0.78),
                Vec3::new(-0.34, 0.70, -0.08),
                palette.primary,
                SurfaceKind::Brick,
            ));
            parts.push(part(
                Vec3::new(0.42, 1.22, 0.76),
                Vec3::new(0.00, 0.66, 0.06),
                palette.secondary,
                SurfaceKind::Brick,
            ));
            parts.push(part(
                Vec3::new(0.40, 1.16, 0.72),
                Vec3::new(0.34, 0.62, 0.18),
                palette.primary,
                SurfaceKind::Brick,
            ));
            if !matches!(mesh_detail, MeshDetail::Low) {
                parts.push(part(
                    Vec3::new(0.34, 0.14, 0.22),
                    Vec3::new(-0.34, 1.38, -0.06),
                    palette.glass,
                    SurfaceKind::Glass,
                ));
            }
            1.36
        }
        1 => {
            parts.push(part(
                Vec3::new(0.74, 1.58, 0.82),
                Vec3::new(-0.20, 0.86, -0.10),
                palette.primary,
                SurfaceKind::Brick,
            ));
            parts.push(part(
                Vec3::new(0.46, 1.18, 0.62),
                Vec3::new(0.34, 0.66, 0.22),
                palette.secondary,
                SurfaceKind::Glass,
            ));
            parts.push(part(
                Vec3::new(0.44, 0.22, 0.36),
                Vec3::new(0.22, 0.16, -0.20),
                palette.podium,
                SurfaceKind::Concrete,
            ));
            1.64
        }
        _ => {
            parts.push(part(
                Vec3::new(0.90, 1.22, 0.72),
                Vec3::new(0.0, 0.68, -0.12),
                palette.primary,
                SurfaceKind::Brick,
            ));
            parts.push(part(
                Vec3::new(0.58, 0.94, 0.56),
                Vec3::new(0.18, 0.54, 0.28),
                palette.secondary,
                SurfaceKind::Glass,
            ));
            parts.push(part(
                Vec3::new(0.44, 0.88, 0.42),
                Vec3::new(-0.34, 0.50, 0.24),
                palette.secondary,
                SurfaceKind::Brick,
            ));
            if style.wealth_tier >= 1 {
                parts.push(part(
                    Vec3::new(0.26, 0.10, 0.20),
                    Vec3::new(0.22, 1.22, 0.30),
                    palette.accent,
                    SurfaceKind::PaintedMetal,
                ));
            }
            1.28
        }
    }
}

fn push_mid_density_building(
    parts: &mut Vec<BlockoutPart>,
    style: &BuildingStyle,
    palette: &BuildingPalette,
    mesh_detail: MeshDetail,
) -> f32 {
    let variant = (style.seed + style.lot_span as u32) % 3;
    parts.push(part(
        Vec3::new(1.66, 0.28, 1.48),
        Vec3::new(0.0, 0.18, 0.0),
        palette.podium,
        SurfaceKind::Concrete,
    ));

    match variant {
        0 => {
            parts.push(part(
                Vec3::new(1.12, 2.24, 0.74),
                Vec3::new(-0.14, 1.34, -0.12),
                palette.primary,
                SurfaceKind::Brick,
            ));
            parts.push(part(
                Vec3::new(0.54, 1.72, 0.54),
                Vec3::new(0.46, 1.08, 0.30),
                palette.secondary,
                SurfaceKind::Glass,
            ));
            2.48
        }
        1 => {
            for offset in [
                Vec3::new(-0.34, 0.96, -0.30),
                Vec3::new(0.34, 0.96, -0.30),
                Vec3::new(-0.34, 0.96, 0.30),
                Vec3::new(0.34, 0.96, 0.30),
            ] {
                parts.push(part(
                    Vec3::new(0.42, 1.68, 0.42),
                    offset,
                    palette.primary,
                    SurfaceKind::Brick,
                ));
            }
            parts.push(part(
                Vec3::new(0.38, 0.48, 0.38),
                Vec3::new(0.0, 0.42, 0.0),
                palette.glass,
                SurfaceKind::Glass,
            ));
            1.88
        }
        _ => {
            parts.push(part(
                Vec3::new(1.08, 1.88, 0.84),
                Vec3::new(0.0, 1.10, -0.08),
                palette.primary,
                SurfaceKind::Brick,
            ));
            parts.push(part(
                Vec3::new(0.78, 1.24, 0.60),
                Vec3::new(0.18, 2.12, 0.16),
                palette.secondary,
                SurfaceKind::Glass,
            ));
            if matches!(mesh_detail, MeshDetail::High) {
                parts.push(part(
                    Vec3::new(0.36, 0.10, 0.22),
                    Vec3::new(-0.30, 1.94, 0.22),
                    palette.accent,
                    SurfaceKind::PaintedMetal,
                ));
            }
            2.70
        }
    }
}

fn push_high_density_building(
    parts: &mut Vec<BlockoutPart>,
    style: &BuildingStyle,
    palette: &BuildingPalette,
    mesh_detail: MeshDetail,
) -> f32 {
    let variant = (style.seed + style.road_frontage as u32) % 3;
    parts.push(part(
        Vec3::new(1.76, 0.32, 1.52),
        Vec3::new(0.0, 0.20, 0.0),
        palette.podium,
        SurfaceKind::Concrete,
    ));

    match variant {
        0 => {
            parts.push(part(
                Vec3::new(0.78, 4.56, 0.78),
                Vec3::new(-0.12, 2.62, -0.10),
                palette.primary,
                SurfaceKind::Glass,
            ));
            parts.push(part(
                Vec3::new(0.58, 2.24, 0.56),
                Vec3::new(0.40, 1.30, 0.32),
                palette.secondary,
                SurfaceKind::Brick,
            ));
            4.82
        }
        1 => {
            parts.push(part(
                Vec3::new(0.44, 3.60, 0.76),
                Vec3::new(-0.34, 2.10, -0.06),
                palette.primary,
                SurfaceKind::Glass,
            ));
            parts.push(part(
                Vec3::new(0.46, 3.30, 0.72),
                Vec3::new(0.26, 1.95, 0.18),
                palette.secondary,
                SurfaceKind::Glass,
            ));
            parts.push(part(
                Vec3::new(0.72, 0.22, 0.38),
                Vec3::new(-0.04, 1.04, -0.18),
                palette.podium,
                SurfaceKind::Concrete,
            ));
            3.86
        }
        _ => {
            parts.push(part(
                Vec3::new(0.92, 3.04, 0.84),
                Vec3::new(0.0, 1.88, 0.0),
                palette.primary,
                SurfaceKind::Brick,
            ));
            parts.push(part(
                Vec3::new(0.56, 2.18, 0.56),
                Vec3::new(0.10, 4.48, 0.10),
                palette.glass,
                SurfaceKind::Glass,
            ));
            if matches!(mesh_detail, MeshDetail::High) {
                parts.push(part(
                    Vec3::new(0.30, 0.16, 0.30),
                    Vec3::new(0.10, 5.66, 0.10),
                    palette.accent,
                    SurfaceKind::PaintedMetal,
                ));
            }
            5.58
        }
    }
}

fn push_tower_building(
    parts: &mut Vec<BlockoutPart>,
    style: &BuildingStyle,
    palette: &BuildingPalette,
    mesh_detail: MeshDetail,
) -> f32 {
    parts.push(part(
        Vec3::new(1.86, 0.34, 1.56),
        Vec3::new(0.0, 0.22, 0.0),
        palette.podium,
        SurfaceKind::Concrete,
    ));

    if style.seed & 1 == 0 {
        parts.push(part(
            Vec3::new(0.62, 6.12, 0.62),
            Vec3::new(0.0, 3.42, -0.06),
            palette.primary,
            SurfaceKind::Glass,
        ));
        parts.push(part(
            Vec3::new(0.64, 2.38, 0.56),
            Vec3::new(0.42, 1.38, 0.30),
            palette.secondary,
            SurfaceKind::Brick,
        ));
        if matches!(mesh_detail, MeshDetail::High) {
            parts.push(part(
                Vec3::new(0.30, 0.18, 0.30),
                Vec3::new(0.0, 6.62, -0.06),
                palette.accent,
                SurfaceKind::PaintedMetal,
            ));
        }
        6.70
    } else {
        parts.push(part(
            Vec3::new(0.40, 4.84, 0.54),
            Vec3::new(-0.26, 2.76, -0.02),
            palette.primary,
            SurfaceKind::Glass,
        ));
        parts.push(part(
            Vec3::new(0.40, 5.44, 0.54),
            Vec3::new(0.24, 3.06, 0.12),
            palette.secondary,
            SurfaceKind::Glass,
        ));
        parts.push(part(
            Vec3::new(0.76, 0.22, 0.28),
            Vec3::new(-0.02, 3.72, 0.06),
            palette.accent,
            SurfaceKind::PaintedMetal,
        ));
        if matches!(mesh_detail, MeshDetail::High) {
            parts.push(part(
                Vec3::new(0.22, 0.14, 0.22),
                Vec3::new(0.24, 5.92, 0.12),
                palette.accent,
                SurfaceKind::PaintedMetal,
            ));
        }
        5.88
    }
}

fn push_facade_accents(
    parts: &mut Vec<BlockoutPart>,
    style: &BuildingStyle,
    palette: &BuildingPalette,
    roof_y: f32,
    mesh_detail: MeshDetail,
) {
    if matches!(mesh_detail, MeshDetail::Low) {
        return;
    }

    let accent_surface = if style.tech_tier >= 2 {
        SurfaceKind::Glass
    } else {
        SurfaceKind::PaintedMetal
    };
    let band_width = match style.density {
        BuildingDensity::Low => 0.52,
        BuildingDensity::Mid => 0.92,
        BuildingDensity::High => 1.12,
        BuildingDensity::Tower => 1.18,
    };
    let band_z = if style.near_river { 0.26 } else { 0.40 };

    parts.push(part(
        Vec3::new(band_width, 0.06, 0.05),
        Vec3::new(0.0, roof_y - 0.14, band_z),
        palette.accent,
        accent_surface,
    ));

    let spine_height = (roof_y * 0.58).max(0.72);
    let spine_x = if style.seed & 1 == 0 { 0.44 } else { -0.36 };
    parts.push(part(
        Vec3::new(0.06, spine_height, 0.06),
        Vec3::new(spine_x, 0.24 + spine_height * 0.5, -0.12),
        if style.tech_tier >= 1 {
            palette.glass
        } else {
            palette.accent
        },
        accent_surface,
    ));

    if style.road_frontage >= 2 && style.density.is_dense() {
        parts.push(part(
            Vec3::new(0.76, 0.05, 0.05),
            Vec3::new(0.0, 1.08, 0.42),
            palette.secondary,
            SurfaceKind::PaintedMetal,
        ));
    }

    if matches!(mesh_detail, MeshDetail::High) && style.wealth_tier >= 1 {
        parts.push(part(
            Vec3::new(0.20, 0.20, 0.20),
            Vec3::new(0.0, roof_y + 0.16, 0.0),
            palette.accent,
            SurfaceKind::Glass,
        ));
    }
}

fn push_lot_base(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    lot_color: Color,
    surface: SurfaceKind,
) {
    parts.push(part(
        Vec3::new(TILE_WORLD_SIZE * 0.90, 0.06, TILE_WORLD_SIZE * 0.90),
        Vec3::new(0.0, 0.03, 0.0),
        lot_color,
        surface,
    ));
    push_lot_boundaries(parts, lot_boundary_mask(game, pos));
}

fn push_lot_boundaries(parts: &mut Vec<BlockoutPart>, mask: u8) {
    let boundary_color = Color::srgb(0.74, 0.78, 0.82);
    for (bit, offset, horizontal) in [
        (NORTH, Vec3::new(0.0, 0.072, -TILE_WORLD_SIZE * 0.39), true),
        (SOUTH, Vec3::new(0.0, 0.072, TILE_WORLD_SIZE * 0.39), true),
        (EAST, Vec3::new(TILE_WORLD_SIZE * 0.39, 0.072, 0.0), false),
        (WEST, Vec3::new(-TILE_WORLD_SIZE * 0.39, 0.072, 0.0), false),
    ] {
        if mask & bit == 0 {
            continue;
        }
        let size = if horizontal {
            Vec3::new(TILE_WORLD_SIZE * 0.82, 0.02, 0.06)
        } else {
            Vec3::new(0.06, 0.02, TILE_WORLD_SIZE * 0.82)
        };
        parts.push(part(size, offset, boundary_color, SurfaceKind::LotPaint));
    }
}

fn part(size: Vec3, offset: Vec3, color: Color, surface: SurfaceKind) -> BlockoutPart {
    BlockoutPart {
        size,
        offset,
        color,
        surface,
    }
}

fn road_mask(game: &GameState, pos: IVec2) -> u8 {
    let mut mask = 0;
    for (delta, bit) in [
        (IVec2::new(0, -1), NORTH),
        (IVec2::new(1, 0), EAST),
        (IVec2::new(0, 1), SOUTH),
        (IVec2::new(-1, 0), WEST),
    ] {
        let neighbor = pos + delta;
        if in_bounds(neighbor) && game.tile(neighbor).kind == TileKind::Road {
            mask |= bit;
        }
    }
    mask
}

fn lot_boundary_mask(game: &GameState, pos: IVec2) -> u8 {
    let kind = game.tile(pos).kind;
    if matches!(kind, TileKind::Road) {
        return 0;
    }

    let mut mask = 0;
    for (delta, bit) in [
        (IVec2::new(0, -1), NORTH),
        (IVec2::new(1, 0), EAST),
        (IVec2::new(0, 1), SOUTH),
        (IVec2::new(-1, 0), WEST),
    ] {
        let neighbor = pos + delta;
        if !in_bounds(neighbor) {
            mask |= bit;
            continue;
        }

        let neighbor_kind = game.tile(neighbor).kind;
        if neighbor_kind == TileKind::Road
            || neighbor_kind == TileKind::River
            || neighbor_kind != kind
        {
            mask |= bit;
        }
    }
    mask
}

fn road_adjacency_count(game: &GameState, pos: IVec2) -> i32 {
    [
        IVec2::new(0, -1),
        IVec2::new(1, 0),
        IVec2::new(0, 1),
        IVec2::new(-1, 0),
    ]
    .into_iter()
    .filter(|delta| {
        let neighbor = pos + *delta;
        in_bounds(neighbor) && game.tile(neighbor).kind == TileKind::Road
    })
    .count() as i32
}

fn build_terrain_mesh(game: &GameState) -> Mesh {
    let margin_tiles = 4.0;
    let samples_per_tile = 2;
    let width = ((GRID_W as f32 + margin_tiles * 2.0) * samples_per_tile as f32) as usize;
    let depth = ((GRID_H as f32 + margin_tiles * 2.0) * samples_per_tile as f32) as usize;
    let step = TILE_WORLD_SIZE / samples_per_tile as f32;
    let start = tile_to_world(IVec2::ZERO, 0.0)
        - Vec3::new(
            TILE_WORLD_SIZE * margin_tiles,
            0.0,
            TILE_WORLD_SIZE * margin_tiles,
        );

    let mut heights = Vec::with_capacity((width + 1) * (depth + 1));
    let mut positions = Vec::with_capacity((width + 1) * (depth + 1));
    let mut normals = Vec::with_capacity((width + 1) * (depth + 1));
    let mut uvs = Vec::with_capacity((width + 1) * (depth + 1));

    for z in 0..=depth {
        for x in 0..=width {
            let world_x = start.x + x as f32 * step;
            let world_z = start.z + z as f32 * step;
            let height = terrain_height_at(game, world_x, world_z);
            heights.push(height);
            positions.push([world_x, height, world_z]);
            uvs.push([x as f32 / width as f32, z as f32 / depth as f32]);
        }
    }

    for z in 0..=depth {
        for x in 0..=width {
            let left = sample_height(&heights, width + 1, x.saturating_sub(1), z);
            let right = sample_height(&heights, width + 1, (x + 1).min(width), z);
            let down = sample_height(&heights, width + 1, x, z.saturating_sub(1));
            let up = sample_height(&heights, width + 1, x, (z + 1).min(depth));
            let normal = Vec3::new(left - right, step * 2.0, down - up).normalize_or_zero();
            normals.push([normal.x, normal.y.max(0.32), normal.z]);
        }
    }

    let mut indices = Vec::with_capacity(width * depth * 6);
    for z in 0..depth {
        for x in 0..width {
            let i = (z * (width + 1) + x) as u32;
            let row = (width + 1) as u32;
            indices.extend_from_slice(&[i, i + row, i + 1, i + 1, i + row, i + row + 1]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn terrain_height_at(game: &GameState, world_x: f32, world_z: f32) -> f32 {
    let base = -0.46 + (world_x * 0.07).sin() * 0.04 + (world_z * 0.05).cos() * 0.05;
    let mut river_distance = f32::INFINITY;

    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let pos = IVec2::new(x, y);
            if game.tile(pos).kind != TileKind::River {
                continue;
            }
            let center = tile_to_world(pos, 0.0);
            river_distance =
                river_distance.min(Vec2::new(center.x - world_x, center.z - world_z).length());
        }
    }

    let trench = if river_distance.is_finite() {
        let t = (1.0 - (river_distance / (TILE_WORLD_SIZE * 1.25)).clamp(0.0, 1.0)).powi(2);
        t * 0.58
    } else {
        0.0
    };

    base - trench
}

fn sample_height(heights: &[f32], row_len: usize, x: usize, z: usize) -> f32 {
    heights[z * row_len + x]
}

fn in_bounds(pos: IVec2) -> bool {
    (0..GRID_W).contains(&pos.x) && (0..GRID_H).contains(&pos.y)
}
