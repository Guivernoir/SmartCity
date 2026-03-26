use crate::settings::ActiveRenderTier;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(super) enum SurfaceKind {
    Asphalt,
    Concrete,
    Glass,
    PaintedMetal,
    Brick,
    Vegetation,
    Water,
    RoadPaint,
    LotPaint,
    Wear,
}

#[derive(Resource, Default)]
pub struct RenderAssetCache {
    cuboids: HashMap<MeshKey, Handle<Mesh>>,
    materials: HashMap<MaterialKey, Handle<StandardMaterial>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MeshKey {
    x: u32,
    y: u32,
    z: u32,
}

impl MeshKey {
    fn from_size(size: Vec3) -> Self {
        Self {
            x: quantize_dimension(size.x),
            y: quantize_dimension(size.y),
            z: quantize_dimension(size.z),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MaterialKey {
    surface: SurfaceKind,
    color: [u8; 4],
    tier: u8,
}

pub(super) fn cuboid_mesh(
    cache: &mut RenderAssetCache,
    meshes: &mut Assets<Mesh>,
    size: Vec3,
) -> Handle<Mesh> {
    let key = MeshKey::from_size(size);
    cache
        .cuboids
        .entry(key)
        .or_insert_with(|| meshes.add(Cuboid::new(size.x, size.y, size.z)))
        .clone()
}

pub(super) fn surface_material(
    cache: &mut RenderAssetCache,
    materials: &mut Assets<StandardMaterial>,
    surface: SurfaceKind,
    color: Color,
    render_tier: ActiveRenderTier,
) -> Handle<StandardMaterial> {
    let key = MaterialKey {
        surface,
        color: color.to_srgba().to_u8_array(),
        tier: render_tier_bucket(render_tier),
    };

    cache
        .materials
        .entry(key)
        .or_insert_with(|| materials.add(build_surface_material(surface, color, render_tier)))
        .clone()
}

fn build_surface_material(
    surface: SurfaceKind,
    color: Color,
    render_tier: ActiveRenderTier,
) -> StandardMaterial {
    let mut material = StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.82,
        metallic: 0.0,
        reflectance: 0.42,
        ..default()
    };

    match surface {
        SurfaceKind::Asphalt => {
            material.perceptual_roughness = 0.95;
            material.reflectance = 0.18;
        }
        SurfaceKind::Concrete => {
            material.perceptual_roughness = 0.88;
            material.reflectance = 0.28;
        }
        SurfaceKind::Glass => {
            material.perceptual_roughness = match render_tier {
                ActiveRenderTier::ShippingLow => 0.28,
                ActiveRenderTier::Full3d => 0.12,
                ActiveRenderTier::FallbackDebug => 0.35,
            };
            material.reflectance = 0.92;
        }
        SurfaceKind::PaintedMetal => {
            material.perceptual_roughness = match render_tier {
                ActiveRenderTier::ShippingLow => 0.44,
                ActiveRenderTier::Full3d => 0.28,
                ActiveRenderTier::FallbackDebug => 0.52,
            };
            material.metallic = 0.74;
            material.reflectance = 0.86;
        }
        SurfaceKind::Brick => {
            material.perceptual_roughness = 0.93;
            material.reflectance = 0.22;
        }
        SurfaceKind::Vegetation => {
            material.perceptual_roughness = 0.98;
            material.reflectance = 0.16;
            material.diffuse_transmission = if matches!(render_tier, ActiveRenderTier::Full3d) {
                0.10
            } else {
                0.0
            };
        }
        SurfaceKind::Water => {
            material.base_color = if matches!(render_tier, ActiveRenderTier::Full3d) {
                color.with_alpha(0.84)
            } else {
                color
            };
            material.alpha_mode = if matches!(render_tier, ActiveRenderTier::Full3d) {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
            material.perceptual_roughness = if matches!(render_tier, ActiveRenderTier::Full3d) {
                0.08
            } else {
                0.18
            };
            material.reflectance = 0.92;
        }
        SurfaceKind::RoadPaint | SurfaceKind::LotPaint => {
            material.perceptual_roughness = 0.62;
            material.reflectance = 0.52;
        }
        SurfaceKind::Wear => {
            material.perceptual_roughness = 1.0;
            material.reflectance = 0.12;
        }
    }

    material
}

fn quantize_dimension(value: f32) -> u32 {
    (value.max(0.0) * 1000.0).round() as u32
}

fn render_tier_bucket(render_tier: ActiveRenderTier) -> u8 {
    match render_tier {
        ActiveRenderTier::FallbackDebug => 0,
        ActiveRenderTier::ShippingLow => 1,
        ActiveRenderTier::Full3d => 2,
    }
}
