use super::blockout::BlockoutPart;
use super::materials::SurfaceKind;
use crate::constants::{GRID_H, GRID_W, TILE_WORLD_SIZE};
use crate::game::GameState;
use crate::model::{Asset, AssetKind, TileKind};
use crate::settings::{MeshDetail, PropDensity};
use bevy::prelude::*;

pub(super) fn append_road_props(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    if matches!(prop_density, PropDensity::Off) {
        return;
    }

    let mask = road_mask(game, pos);
    let straight_ns = mask == (NORTH | SOUTH);
    let straight_ew = mask == (EAST | WEST);
    let intersection = mask.count_ones() >= 3;
    let seed = tile_seed(pos);
    let low = density_at_least(prop_density, PropDensity::Low);
    let medium = density_at_least(prop_density, PropDensity::Medium);
    let high = density_at_least(prop_density, PropDensity::High);

    if low && seed % 3 == 0 {
        if straight_ns || intersection {
            push_streetlight(parts, Vec3::new(-0.36, 0.10, -0.22));
            if high {
                push_streetlight(parts, Vec3::new(0.36, 0.10, 0.22));
            }
        } else if straight_ew {
            push_streetlight(parts, Vec3::new(-0.22, 0.10, -0.36));
            if high {
                push_streetlight(parts, Vec3::new(0.22, 0.10, 0.36));
            }
        }
    }

    if medium && intersection {
        push_traffic_light(parts, Vec3::new(-0.36, 0.10, -0.36), true);
        push_traffic_light(parts, Vec3::new(0.36, 0.10, 0.36), false);
    }

    if low && has_adjacent_kind(game, pos, TileKind::Park) && seed % 4 == 1 {
        let offset = if straight_ns {
            Vec3::new(0.34, 0.10, 0.22)
        } else {
            Vec3::new(0.22, 0.10, 0.34)
        };
        push_bench(parts, offset);
        if !matches!(mesh_detail, MeshDetail::Low) {
            push_sign(parts, offset + Vec3::new(-0.18, 0.0, -0.02));
        }
    }

    if medium && has_adjacent_kind(game, pos, TileKind::River) {
        for (delta, offset, horizontal) in [
            (IVec2::new(0, -1), Vec3::new(0.0, 0.10, -0.32), true),
            (IVec2::new(0, 1), Vec3::new(0.0, 0.10, 0.32), true),
            (IVec2::new(1, 0), Vec3::new(0.32, 0.10, 0.0), false),
            (IVec2::new(-1, 0), Vec3::new(-0.32, 0.10, 0.0), false),
        ] {
            if tile_kind_at(game, pos + delta) == Some(TileKind::River) {
                push_barrier(parts, offset, horizontal);
            }
        }
    }

    if high && (straight_ns || straight_ew) && adjacent_building_demand(game, pos) && seed % 5 == 2
    {
        let offset = if straight_ns {
            Vec3::new(0.30, 0.10, -0.08)
        } else {
            Vec3::new(-0.08, 0.10, 0.30)
        };
        push_bus_stop(parts, offset, straight_ew);
    }

    if high
        && (has_adjacent_kind(game, pos, TileKind::Park)
            || has_adjacent_kind(game, pos, TileKind::Empty))
    {
        if straight_ns && seed % 4 == 2 {
            push_tree(parts, Vec3::new(0.36, 0.08, -0.30), 0.90);
        } else if straight_ew && seed % 4 == 3 {
            push_tree(parts, Vec3::new(-0.30, 0.08, 0.36), 0.90);
        }
    }

    if medium && (straight_ns || straight_ew) && seed % 3 == 1 {
        let offset = if straight_ns {
            Vec3::new(-0.34, 0.08, 0.28)
        } else {
            Vec3::new(0.28, 0.08, -0.34)
        };
        push_shrub_patch(parts, offset, Vec3::new(0.22, 0.12, 0.16));
    }
}

pub(super) fn append_empty_lot_props(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    prop_density: PropDensity,
) {
    if matches!(prop_density, PropDensity::Off) {
        return;
    }

    let seed = tile_seed(pos);
    let low = density_at_least(prop_density, PropDensity::Low);
    let medium = density_at_least(prop_density, PropDensity::Medium);
    let high = density_at_least(prop_density, PropDensity::High);

    if low {
        for (delta, offset, horizontal) in [
            (IVec2::new(0, -1), Vec3::new(0.0, 0.09, -0.34), true),
            (IVec2::new(0, 1), Vec3::new(0.0, 0.09, 0.34), true),
            (IVec2::new(1, 0), Vec3::new(0.34, 0.09, 0.0), false),
            (IVec2::new(-1, 0), Vec3::new(-0.34, 0.09, 0.0), false),
        ] {
            if tile_kind_at(game, pos + delta) != Some(TileKind::Road) {
                push_fence(parts, offset, horizontal, 0.72);
            }
        }
    }

    if medium && seed % 2 == 0 {
        push_grass_patch(
            parts,
            Vec3::new(-0.20, 0.07, -0.18),
            Vec3::new(0.34, 0.02, 0.26),
        );
        push_grass_patch(
            parts,
            Vec3::new(0.18, 0.07, 0.20),
            Vec3::new(0.28, 0.02, 0.22),
        );
    }

    if high && has_adjacent_kind(game, pos, TileKind::Road) {
        push_parking_markings(parts, 0.18);
        push_barrier(parts, Vec3::new(0.0, 0.10, -0.24), true);
    }
}

pub(super) fn append_building_props(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
    roof_y: f32,
    dense_roofline: bool,
) {
    if matches!(prop_density, PropDensity::Off) {
        return;
    }

    if !matches!(mesh_detail, MeshDetail::Low) {
        push_hvac(
            parts,
            Vec3::new(-0.22, roof_y, 0.18),
            Vec3::new(0.32, 0.16, 0.22),
        );
        if dense_roofline && density_at_least(prop_density, PropDensity::High) {
            push_hvac(
                parts,
                Vec3::new(0.28, roof_y + 0.02, -0.12),
                Vec3::new(0.26, 0.14, 0.20),
            );
        }
    }

    if density_at_least(prop_density, PropDensity::Medium)
        && has_adjacent_kind(game, pos, TileKind::Road)
    {
        push_dumpster(parts, Vec3::new(0.28, 0.17, -0.28));
    }

    if density_at_least(prop_density, PropDensity::High) {
        push_parking_markings(parts, -0.08);
        push_tree(parts, Vec3::new(-0.34, 0.08, 0.32), 0.76);
    }
}

pub(super) fn append_industrial_props(
    parts: &mut Vec<BlockoutPart>,
    game: &GameState,
    pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    if matches!(prop_density, PropDensity::Off) {
        return;
    }

    push_dumpster(parts, Vec3::new(-0.30, 0.17, 0.30));
    push_transformer(parts, Vec3::new(0.34, 0.17, 0.26));

    if density_at_least(prop_density, PropDensity::Medium) {
        push_fence(parts, Vec3::new(0.0, 0.09, 0.34), true, 0.76);
        push_fence(parts, Vec3::new(-0.34, 0.09, 0.0), false, 0.76);
    }

    if density_at_least(prop_density, PropDensity::High) && !matches!(mesh_detail, MeshDetail::Low)
    {
        push_hvac(
            parts,
            Vec3::new(0.0, 1.36, -0.04),
            Vec3::new(0.42, 0.18, 0.24),
        );
        push_barrier(parts, Vec3::new(0.0, 0.10, -0.34), true);
    }

    if has_adjacent_kind(game, pos, TileKind::Road)
        && density_at_least(prop_density, PropDensity::High)
    {
        push_sign(parts, Vec3::new(0.30, 0.10, -0.30));
    }
}

pub(super) fn append_utility_props(
    parts: &mut Vec<BlockoutPart>,
    _game: &GameState,
    _pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    if matches!(prop_density, PropDensity::Off) {
        return;
    }

    push_transformer(parts, Vec3::new(-0.34, 0.17, -0.30));
    push_transformer(parts, Vec3::new(0.34, 0.17, -0.30));
    push_barrier(parts, Vec3::new(0.0, 0.10, 0.34), true);

    if density_at_least(prop_density, PropDensity::Medium) {
        parts.push(part(
            Vec3::new(0.92, 0.012, 0.18),
            Vec3::new(0.0, 0.095, 0.14),
            Color::srgb(0.82, 0.90, 0.98),
            SurfaceKind::LotPaint,
        ));
    }

    if density_at_least(prop_density, PropDensity::High) && !matches!(mesh_detail, MeshDetail::Low)
    {
        push_hvac(
            parts,
            Vec3::new(0.0, 1.30, 0.0),
            Vec3::new(0.30, 0.16, 0.22),
        );
    }
}

pub(super) fn append_control_center_props(
    parts: &mut Vec<BlockoutPart>,
    _game: &GameState,
    _pos: IVec2,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) {
    if matches!(prop_density, PropDensity::Off) {
        return;
    }

    push_bench(parts, Vec3::new(-0.34, 0.10, 0.36));
    push_sign(parts, Vec3::new(0.34, 0.10, 0.32));

    if density_at_least(prop_density, PropDensity::Medium) {
        push_tree(parts, Vec3::new(0.34, 0.08, -0.34), 0.84);
        push_tree(parts, Vec3::new(-0.36, 0.08, -0.34), 0.84);
    }

    if density_at_least(prop_density, PropDensity::High) && !matches!(mesh_detail, MeshDetail::Low)
    {
        push_barrier(parts, Vec3::new(0.0, 0.10, 0.34), true);
    }
}

pub(super) fn append_park_props(
    parts: &mut Vec<BlockoutPart>,
    _game: &GameState,
    _pos: IVec2,
    prop_density: PropDensity,
) {
    if matches!(prop_density, PropDensity::Off) {
        return;
    }

    push_tree(parts, Vec3::new(0.30, 0.08, -0.26), 1.08);
    if density_at_least(prop_density, PropDensity::Medium) {
        push_tree(parts, Vec3::new(-0.28, 0.08, 0.24), 0.96);
        push_bench(parts, Vec3::new(0.06, 0.10, 0.32));
        push_shrub_patch(
            parts,
            Vec3::new(-0.04, 0.08, -0.10),
            Vec3::new(0.26, 0.14, 0.18),
        );
    }
    if density_at_least(prop_density, PropDensity::High) {
        push_grass_patch(
            parts,
            Vec3::new(-0.28, 0.07, -0.30),
            Vec3::new(0.30, 0.02, 0.26),
        );
        push_grass_patch(
            parts,
            Vec3::new(0.28, 0.07, 0.28),
            Vec3::new(0.30, 0.02, 0.24),
        );
    }
}

pub(super) fn asset_prop_parts(
    asset: &Asset,
    asset_size: Vec3,
    mesh_detail: MeshDetail,
    prop_density: PropDensity,
) -> Vec<BlockoutPart> {
    if matches!(prop_density, PropDensity::Off) {
        return Vec::new();
    }

    let mut parts = Vec::with_capacity(8);
    match asset.kind {
        AssetKind::Bridge => {
            let rail_y = 0.04;
            let rail_length = asset_size.x.max(TILE_WORLD_SIZE * 1.42);
            parts.push(part(
                Vec3::new(rail_length, 0.08, 0.06),
                Vec3::new(0.0, rail_y, 0.22),
                Color::srgb(0.84, 0.78, 0.52),
                SurfaceKind::PaintedMetal,
            ));
            parts.push(part(
                Vec3::new(rail_length, 0.08, 0.06),
                Vec3::new(0.0, rail_y, -0.22),
                Color::srgb(0.84, 0.78, 0.52),
                SurfaceKind::PaintedMetal,
            ));
            if density_at_least(prop_density, PropDensity::High) {
                parts.push(part(
                    Vec3::new(0.12, 0.12, 0.12),
                    Vec3::new(-rail_length * 0.42, 0.12, 0.0),
                    Color::srgb(0.98, 0.26, 0.20),
                    SurfaceKind::LotPaint,
                ));
                parts.push(part(
                    Vec3::new(0.12, 0.12, 0.12),
                    Vec3::new(rail_length * 0.42, 0.12, 0.0),
                    Color::srgb(0.98, 0.26, 0.20),
                    SurfaceKind::LotPaint,
                ));
            }
        }
        AssetKind::Sensor => {
            parts.push(part(
                Vec3::new(0.42, 0.08, 0.42),
                Vec3::new(0.0, -asset_size.y * 0.5 - 0.05, 0.0),
                Color::srgb(0.60, 0.64, 0.68),
                SurfaceKind::Concrete,
            ));
            parts.push(part(
                Vec3::new(0.18, 0.22, 0.18),
                Vec3::new(0.18, -asset_size.y * 0.22, -0.16),
                Color::srgb(0.70, 0.78, 0.84),
                SurfaceKind::PaintedMetal,
            ));
        }
        AssetKind::Plc => {
            push_hvac(
                &mut parts,
                Vec3::new(0.0, asset_size.y * 0.5 + 0.14, 0.0),
                Vec3::new(0.24, 0.14, 0.18),
            );
            if density_at_least(prop_density, PropDensity::Medium) {
                push_conduit(&mut parts, Vec3::new(0.0, -asset_size.y * 0.30, 0.20), true);
            }
        }
        AssetKind::Gateway => {
            parts.push(part(
                Vec3::new(0.52, 0.08, 0.52),
                Vec3::new(0.0, -asset_size.y * 0.5 - 0.05, 0.0),
                Color::srgb(0.58, 0.60, 0.66),
                SurfaceKind::Concrete,
            ));
            if !matches!(mesh_detail, MeshDetail::Low) {
                parts.push(part(
                    Vec3::new(0.34, 0.05, 0.05),
                    Vec3::new(0.0, asset_size.y * 0.44, 0.0),
                    Color::srgb(0.96, 0.78, 0.92),
                    SurfaceKind::PaintedMetal,
                ));
            }
        }
        AssetKind::Substation => {
            push_transformer(&mut parts, Vec3::new(-0.26, 0.16, 0.28));
            push_transformer(&mut parts, Vec3::new(0.28, 0.16, -0.18));
            if density_at_least(prop_density, PropDensity::Medium) {
                push_fence(&mut parts, Vec3::new(0.0, 0.08, 0.38), true, 0.92);
            }
        }
        AssetKind::PumpStation => {
            parts.push(part(
                Vec3::new(0.54, 0.22, 0.22),
                Vec3::new(-0.18, -asset_size.y * 0.18, 0.0),
                Color::srgb(0.56, 0.72, 0.90),
                SurfaceKind::PaintedMetal,
            ));
            parts.push(part(
                Vec3::new(0.20, 0.52, 0.20),
                Vec3::new(0.26, 0.10, 0.18),
                Color::srgb(0.70, 0.82, 0.96),
                SurfaceKind::PaintedMetal,
            ));
        }
    }

    parts
}

const NORTH: u8 = 1;
const EAST: u8 = 2;
const SOUTH: u8 = 4;
const WEST: u8 = 8;

fn push_streetlight(parts: &mut Vec<BlockoutPart>, offset: Vec3) {
    parts.push(part(
        Vec3::new(0.08, 1.36, 0.08),
        offset + Vec3::new(0.0, 0.68, 0.0),
        Color::srgb(0.72, 0.78, 0.84),
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.20, 0.06, 0.32),
        offset + Vec3::new(0.0, 1.34, 0.08),
        Color::srgb(0.94, 0.92, 0.76),
        SurfaceKind::PaintedMetal,
    ));
}

fn push_traffic_light(parts: &mut Vec<BlockoutPart>, offset: Vec3, horizontal_arm: bool) {
    parts.push(part(
        Vec3::new(0.08, 1.18, 0.08),
        offset + Vec3::new(0.0, 0.60, 0.0),
        Color::srgb(0.72, 0.78, 0.84),
        SurfaceKind::PaintedMetal,
    ));
    let arm_size = if horizontal_arm {
        Vec3::new(0.36, 0.05, 0.05)
    } else {
        Vec3::new(0.05, 0.05, 0.36)
    };
    let arm_offset = if horizontal_arm {
        Vec3::new(0.12, 1.08, 0.0)
    } else {
        Vec3::new(0.0, 1.08, 0.12)
    };
    parts.push(part(
        arm_size,
        offset + arm_offset,
        Color::srgb(0.72, 0.78, 0.84),
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.12, 0.20, 0.12),
        offset
            + if horizontal_arm {
                Vec3::new(0.26, 0.98, 0.0)
            } else {
                Vec3::new(0.0, 0.98, 0.26)
            },
        Color::srgb(0.16, 0.18, 0.20),
        SurfaceKind::PaintedMetal,
    ));
}

fn push_bench(parts: &mut Vec<BlockoutPart>, offset: Vec3) {
    parts.push(part(
        Vec3::new(0.34, 0.06, 0.12),
        offset + Vec3::new(0.0, 0.10, 0.0),
        Color::srgb(0.58, 0.44, 0.26),
        SurfaceKind::Brick,
    ));
    parts.push(part(
        Vec3::new(0.34, 0.16, 0.05),
        offset + Vec3::new(0.0, 0.18, -0.06),
        Color::srgb(0.50, 0.38, 0.24),
        SurfaceKind::Brick,
    ));
}

fn push_sign(parts: &mut Vec<BlockoutPart>, offset: Vec3) {
    parts.push(part(
        Vec3::new(0.05, 0.62, 0.05),
        offset + Vec3::new(0.0, 0.34, 0.0),
        Color::srgb(0.74, 0.80, 0.88),
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.22, 0.18, 0.04),
        offset + Vec3::new(0.0, 0.58, 0.0),
        Color::srgb(0.34, 0.64, 0.94),
        SurfaceKind::Glass,
    ));
}

fn push_bus_stop(parts: &mut Vec<BlockoutPart>, offset: Vec3, along_x: bool) {
    let roof_size = if along_x {
        Vec3::new(0.42, 0.06, 0.22)
    } else {
        Vec3::new(0.22, 0.06, 0.42)
    };
    parts.push(part(
        roof_size,
        offset + Vec3::new(0.0, 0.96, 0.0),
        Color::srgb(0.50, 0.62, 0.72),
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.05, 0.92, 0.05),
        offset + Vec3::new(-0.14, 0.46, 0.0),
        Color::srgb(0.72, 0.78, 0.84),
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.05, 0.92, 0.05),
        offset + Vec3::new(0.14, 0.46, 0.0),
        Color::srgb(0.72, 0.78, 0.84),
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.28, 0.06, 0.10),
        offset + Vec3::new(0.0, 0.16, 0.0),
        Color::srgb(0.56, 0.42, 0.24),
        SurfaceKind::Brick,
    ));
}

fn push_barrier(parts: &mut Vec<BlockoutPart>, offset: Vec3, horizontal: bool) {
    let size = if horizontal {
        Vec3::new(0.52, 0.10, 0.06)
    } else {
        Vec3::new(0.06, 0.10, 0.52)
    };
    parts.push(part(
        size,
        offset,
        Color::srgb(0.82, 0.82, 0.84),
        SurfaceKind::Concrete,
    ));
}

fn push_tree(parts: &mut Vec<BlockoutPart>, offset: Vec3, height_scale: f32) {
    parts.push(part(
        Vec3::new(0.10, 0.38 * height_scale, 0.10),
        offset + Vec3::new(0.0, 0.20 * height_scale, 0.0),
        Color::srgb(0.46, 0.30, 0.18),
        SurfaceKind::Brick,
    ));
    parts.push(part(
        Vec3::new(
            0.40 * height_scale,
            0.34 * height_scale,
            0.40 * height_scale,
        ),
        offset + Vec3::new(0.0, 0.52 * height_scale, 0.0),
        Color::srgb(0.16, 0.40, 0.18),
        SurfaceKind::Vegetation,
    ));
}

fn push_shrub_patch(parts: &mut Vec<BlockoutPart>, offset: Vec3, size: Vec3) {
    parts.push(part(
        size,
        offset,
        Color::srgb(0.18, 0.44, 0.20),
        SurfaceKind::Vegetation,
    ));
}

fn push_grass_patch(parts: &mut Vec<BlockoutPart>, offset: Vec3, size: Vec3) {
    parts.push(part(
        size,
        offset,
        Color::srgb(0.20, 0.48, 0.22),
        SurfaceKind::Vegetation,
    ));
}

fn push_fence(parts: &mut Vec<BlockoutPart>, offset: Vec3, horizontal: bool, length: f32) {
    let size = if horizontal {
        Vec3::new(length, 0.10, 0.04)
    } else {
        Vec3::new(0.04, 0.10, length)
    };
    parts.push(part(
        size,
        offset,
        Color::srgb(0.76, 0.80, 0.84),
        SurfaceKind::PaintedMetal,
    ));
}

fn push_hvac(parts: &mut Vec<BlockoutPart>, offset: Vec3, size: Vec3) {
    parts.push(part(
        size,
        offset,
        Color::srgb(0.76, 0.80, 0.84),
        SurfaceKind::PaintedMetal,
    ));
}

fn push_dumpster(parts: &mut Vec<BlockoutPart>, offset: Vec3) {
    parts.push(part(
        Vec3::new(0.26, 0.22, 0.18),
        offset,
        Color::srgb(0.24, 0.44, 0.26),
        SurfaceKind::PaintedMetal,
    ));
}

fn push_transformer(parts: &mut Vec<BlockoutPart>, offset: Vec3) {
    parts.push(part(
        Vec3::new(0.28, 0.24, 0.22),
        offset,
        Color::srgb(0.72, 0.82, 0.56),
        SurfaceKind::PaintedMetal,
    ));
    parts.push(part(
        Vec3::new(0.12, 0.40, 0.12),
        offset + Vec3::new(0.0, 0.28, 0.0),
        Color::srgb(0.82, 0.88, 0.96),
        SurfaceKind::PaintedMetal,
    ));
}

fn push_conduit(parts: &mut Vec<BlockoutPart>, offset: Vec3, horizontal: bool) {
    let size = if horizontal {
        Vec3::new(0.36, 0.08, 0.08)
    } else {
        Vec3::new(0.08, 0.08, 0.36)
    };
    parts.push(part(
        size,
        offset,
        Color::srgb(0.56, 0.60, 0.66),
        SurfaceKind::PaintedMetal,
    ));
}

fn push_parking_markings(parts: &mut Vec<BlockoutPart>, z_offset: f32) {
    for x in [-0.24, 0.0, 0.24] {
        parts.push(part(
            Vec3::new(0.10, 0.012, 0.42),
            Vec3::new(x, 0.094, z_offset),
            Color::srgb(0.90, 0.88, 0.74),
            SurfaceKind::RoadPaint,
        ));
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

fn density_at_least(current: PropDensity, threshold: PropDensity) -> bool {
    density_rank(current) >= density_rank(threshold)
}

fn density_rank(prop_density: PropDensity) -> u8 {
    match prop_density {
        PropDensity::Off => 0,
        PropDensity::Low => 1,
        PropDensity::Medium => 2,
        PropDensity::High => 3,
    }
}

fn tile_seed(pos: IVec2) -> i32 {
    (pos.x * 37 + pos.y * 19).rem_euclid(11)
}

fn adjacent_building_demand(game: &GameState, pos: IVec2) -> bool {
    [
        TileKind::Building,
        TileKind::Industrial,
        TileKind::ControlCenter,
        TileKind::Utility,
    ]
    .into_iter()
    .any(|kind| has_adjacent_kind(game, pos, kind))
}

fn has_adjacent_kind(game: &GameState, pos: IVec2, kind: TileKind) -> bool {
    [
        IVec2::new(0, -1),
        IVec2::new(1, 0),
        IVec2::new(0, 1),
        IVec2::new(-1, 0),
    ]
    .into_iter()
    .any(|delta| tile_kind_at(game, pos + delta) == Some(kind))
}

fn road_mask(game: &GameState, pos: IVec2) -> u8 {
    let mut mask = 0;
    for (delta, bit) in [
        (IVec2::new(0, -1), NORTH),
        (IVec2::new(1, 0), EAST),
        (IVec2::new(0, 1), SOUTH),
        (IVec2::new(-1, 0), WEST),
    ] {
        if tile_kind_at(game, pos + delta) == Some(TileKind::Road) {
            mask |= bit;
        }
    }
    mask
}

fn tile_kind_at(game: &GameState, pos: IVec2) -> Option<TileKind> {
    if (0..GRID_W).contains(&pos.x) && (0..GRID_H).contains(&pos.y) {
        Some(game.tile(pos).kind)
    } else {
        None
    }
}
