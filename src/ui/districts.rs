use crate::constants::{GRID_H, GRID_W};
use crate::game::GameState;
use crate::model::TileKind;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DistrictTheme {
    Core,
    Riverside,
    Garden,
    Innovation,
    Logistics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum BuildingDensity {
    Low,
    Mid,
    High,
    Tower,
}

impl BuildingDensity {
    pub const fn is_dense(self) -> bool {
        !matches!(self, Self::Low)
    }

    pub const fn downgrade(self) -> Self {
        match self {
            Self::Low => Self::Low,
            Self::Mid => Self::Low,
            Self::High => Self::Mid,
            Self::Tower => Self::High,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BuildingStyle {
    pub theme: DistrictTheme,
    pub density: BuildingDensity,
    pub wealth_tier: u8,
    pub tech_tier: u8,
    pub road_frontage: u8,
    pub lot_span: u8,
    pub near_park: bool,
    pub near_river: bool,
    pub near_industrial: bool,
    pub seed: u32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BuildingPalette {
    pub lot: Color,
    pub podium: Color,
    pub primary: Color,
    pub secondary: Color,
    pub glass: Color,
    pub accent: Color,
    pub courtyard: Color,
}

pub(super) fn district_theme(game: &GameState, pos: IVec2) -> DistrictTheme {
    let center = center_weight(pos);
    let river = river_proximity(game, pos);
    let park = proximity_score(game, pos, TileKind::Park, 3);
    let industrial = proximity_score(game, pos, TileKind::Industrial, 4);
    let control = control_center_weight(pos);
    let noise = noise01(game, pos, 0x91e1_0da5);

    if industrial > 0.58 && center < 0.52 {
        DistrictTheme::Logistics
    } else if control + center * 0.34 + noise * 0.18 > 0.82 {
        DistrictTheme::Innovation
    } else if river > 0.56 {
        DistrictTheme::Riverside
    } else if park > 0.34 || noise < 0.16 {
        DistrictTheme::Garden
    } else {
        DistrictTheme::Core
    }
}

pub(super) fn building_style(game: &GameState, pos: IVec2) -> BuildingStyle {
    let theme = district_theme(game, pos);
    let center = center_weight(pos);
    let river = river_proximity(game, pos);
    let park = proximity_score(game, pos, TileKind::Park, 3);
    let industrial = proximity_score(game, pos, TileKind::Industrial, 4);
    let control = control_center_weight(pos);
    let road_frontage = orthogonal_neighbor_count(game, pos, TileKind::Road);
    let arterial = proximity_score(game, pos, TileKind::Road, 2);
    let lot_span = lot_span(game, pos, TileKind::Building);
    let noise = noise01(game, pos, 0x5bd1_e995);

    let wealth_score =
        center * 0.28 + river * 0.20 + park * 0.18 + noise * 0.18 - industrial * 0.24;
    let tech_score = control * 0.34
        + center * 0.18
        + arterial * 0.18
        + if matches!(theme, DistrictTheme::Innovation) {
            0.18
        } else {
            0.0
        }
        + noise * 0.12
        - park * 0.08;

    let skyline = center * 0.46
        + arterial * 0.22
        + road_frontage as f32 * 0.06
        + tech_score * 0.16
        + wealth_score * 0.08
        - park * 0.18
        - if matches!(theme, DistrictTheme::Riverside | DistrictTheme::Garden) {
            0.10
        } else {
            0.0
        };

    let mut density = if skyline > 0.90 {
        BuildingDensity::Tower
    } else if skyline > 0.62 {
        BuildingDensity::High
    } else if skyline > 0.34 {
        BuildingDensity::Mid
    } else {
        BuildingDensity::Low
    };

    if lot_span <= 1 && density > BuildingDensity::Mid {
        density = BuildingDensity::Mid;
    }
    if industrial > 0.48 && center < 0.44 {
        density = density.downgrade();
    }
    if park > 0.42 && !matches!(density, BuildingDensity::Low) && noise < 0.60 {
        density = density.downgrade();
    }

    BuildingStyle {
        theme,
        density,
        wealth_tier: tier_from_score(wealth_score),
        tech_tier: tier_from_score(tech_score),
        road_frontage,
        lot_span,
        near_park: park > 0.28,
        near_river: river > 0.32,
        near_industrial: industrial > 0.36,
        seed: noise_u32(game, pos, 0x7f4a_7c15),
    }
}

pub(super) fn building_palette(style: BuildingStyle) -> BuildingPalette {
    let (mut lot, mut podium, mut primary, mut secondary, mut glass, mut accent, courtyard) =
        match style.theme {
            DistrictTheme::Core => (
                Color::srgb(0.42, 0.45, 0.44),
                Color::srgb(0.58, 0.61, 0.64),
                Color::srgb(0.72, 0.75, 0.78),
                Color::srgb(0.48, 0.54, 0.60),
                Color::srgb(0.50, 0.70, 0.82),
                Color::srgb(0.92, 0.72, 0.40),
                Color::srgb(0.24, 0.34, 0.24),
            ),
            DistrictTheme::Riverside => (
                Color::srgb(0.52, 0.48, 0.42),
                Color::srgb(0.76, 0.70, 0.62),
                Color::srgb(0.88, 0.84, 0.76),
                Color::srgb(0.66, 0.72, 0.78),
                Color::srgb(0.56, 0.78, 0.88),
                Color::srgb(0.24, 0.62, 0.78),
                Color::srgb(0.26, 0.42, 0.32),
            ),
            DistrictTheme::Garden => (
                Color::srgb(0.30, 0.40, 0.28),
                Color::srgb(0.52, 0.40, 0.34),
                Color::srgb(0.66, 0.52, 0.44),
                Color::srgb(0.42, 0.54, 0.42),
                Color::srgb(0.58, 0.74, 0.82),
                Color::srgb(0.76, 0.88, 0.52),
                Color::srgb(0.30, 0.50, 0.28),
            ),
            DistrictTheme::Innovation => (
                Color::srgb(0.34, 0.40, 0.44),
                Color::srgb(0.50, 0.58, 0.62),
                Color::srgb(0.66, 0.76, 0.82),
                Color::srgb(0.40, 0.54, 0.64),
                Color::srgb(0.56, 0.84, 0.88),
                Color::srgb(0.98, 0.86, 0.52),
                Color::srgb(0.22, 0.40, 0.32),
            ),
            DistrictTheme::Logistics => (
                Color::srgb(0.42, 0.36, 0.28),
                Color::srgb(0.58, 0.50, 0.38),
                Color::srgb(0.70, 0.62, 0.46),
                Color::srgb(0.46, 0.40, 0.30),
                Color::srgb(0.56, 0.66, 0.74),
                Color::srgb(0.92, 0.60, 0.24),
                Color::srgb(0.26, 0.32, 0.22),
            ),
        };

    let wealth_lift = style.wealth_tier as f32 * 0.08;
    let tech_cool = style.tech_tier as f32 * 0.10;
    lot = lift(lot, wealth_lift * 0.35);
    podium = lift(podium, wealth_lift * 0.55);
    primary = lift(primary, wealth_lift);
    secondary = mix(secondary, Color::srgb(0.52, 0.76, 0.88), tech_cool * 0.55);
    glass = mix(glass, Color::srgb(0.72, 0.92, 0.98), tech_cool);
    accent = mix(accent, glass, tech_cool * 0.28);

    BuildingPalette {
        lot,
        podium,
        primary,
        secondary,
        glass,
        accent,
        courtyard,
    }
}

fn lot_span(game: &GameState, pos: IVec2, kind: TileKind) -> u8 {
    let same_neighbors = orthogonal_neighbor_count(game, pos, kind);
    match same_neighbors {
        0 => 1,
        1 | 2 => 2,
        _ => 3,
    }
}

fn center_weight(pos: IVec2) -> f32 {
    let cx = (GRID_W - 1) as f32 * 0.5;
    let cy = (GRID_H - 1) as f32 * 0.5;
    let nx = (pos.x as f32 - cx) / (GRID_W as f32 * 0.5);
    let ny = (pos.y as f32 - cy) / (GRID_H as f32 * 0.5);
    let distance = Vec2::new(nx, ny).length();
    (1.0 - distance / 1.05).clamp(0.0, 1.0)
}

fn control_center_weight(pos: IVec2) -> f32 {
    let distance = Vec2::new((pos.x - 2) as f32, (pos.y - 2) as f32).length();
    (1.0 - distance / 16.0).clamp(0.0, 1.0)
}

fn river_proximity(game: &GameState, pos: IVec2) -> f32 {
    (1.0 - (pos.x - game.river_x).abs() as f32 / 6.5).clamp(0.0, 1.0)
}

fn proximity_score(game: &GameState, pos: IVec2, kind: TileKind, radius: i32) -> f32 {
    let mut best: f32 = 0.0;
    for y in -radius..=radius {
        for x in -radius..=radius {
            let check = pos + IVec2::new(x, y);
            if !in_bounds(check) || game.tile(check).kind != kind {
                continue;
            }
            let distance = Vec2::new(x as f32, y as f32).length();
            let score = (1.0 - distance / (radius as f32 + 0.35)).clamp(0.0, 1.0);
            best = best.max(score);
        }
    }
    best
}

fn orthogonal_neighbor_count(game: &GameState, pos: IVec2, kind: TileKind) -> u8 {
    [
        IVec2::new(0, -1),
        IVec2::new(1, 0),
        IVec2::new(0, 1),
        IVec2::new(-1, 0),
    ]
    .into_iter()
    .filter(|delta| {
        let neighbor = pos + *delta;
        in_bounds(neighbor) && game.tile(neighbor).kind == kind
    })
    .count() as u8
}

fn noise01(game: &GameState, pos: IVec2, salt: u32) -> f32 {
    let value = noise_u32(game, pos, salt);
    (value & 0xffff) as f32 / 65535.0
}

fn noise_u32(game: &GameState, pos: IVec2, salt: u32) -> u32 {
    let mut value = pos.x as u32 ^ salt;
    value = value.wrapping_mul(0x9e37_79b1);
    value ^= (pos.y as u32).wrapping_mul(0x85eb_ca6b);
    value = value.wrapping_add((game.river_x as u32).wrapping_mul(0xc2b2_ae35));
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value
}

fn tier_from_score(score: f32) -> u8 {
    if score > 0.72 {
        2
    } else if score > 0.42 {
        1
    } else {
        0
    }
}

fn in_bounds(pos: IVec2) -> bool {
    (0..GRID_W).contains(&pos.x) && (0..GRID_H).contains(&pos.y)
}

fn lift(color: Color, amount: f32) -> Color {
    mix(color, Color::srgb(0.96, 0.97, 0.98), amount.clamp(0.0, 0.4))
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgb(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
    )
}
