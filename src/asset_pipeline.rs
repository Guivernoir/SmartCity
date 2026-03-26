use crate::game::GameState;
use bevy::log::warn;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const ASSET_CATALOG_RELATIVE_PATH: &str = "catalog/asset_catalog.ron";

#[derive(Resource, Debug, Default)]
pub struct AssetCatalog {
    manifest_version: u32,
    pub entries: Vec<AssetCatalogEntry>,
    pub scene_handles: HashMap<String, Handle<Scene>>,
    ready_scene_count: usize,
    missing_scene_count: usize,
    fallback_only_count: usize,
    stats: AssetCatalogStats,
}

impl AssetCatalog {
    pub fn summary_line(&self) -> String {
        format!(
            "Catalog {} defs | roads {} | buildings {} | props {} | anchors {} | {} scene-ready | {} fallback-only | {} missing files",
            self.entries.len(),
            self.stats.road_count,
            self.stats.building_count,
            self.stats.prop_count,
            self.stats.anchor_count,
            self.ready_scene_count,
            self.fallback_only_count,
            self.missing_scene_count
        )
    }

    fn replace_from_manifest(
        &mut self,
        manifest: AssetCatalogManifest,
        scene_handles: HashMap<String, Handle<Scene>>,
        ready_scene_count: usize,
        missing_scene_count: usize,
        fallback_only_count: usize,
    ) {
        self.stats = AssetCatalogStats::from_entries(&manifest.entries);
        self.manifest_version = manifest.version;
        self.entries = manifest.entries;
        self.scene_handles = scene_handles;
        self.ready_scene_count = ready_scene_count;
        self.missing_scene_count = missing_scene_count;
        self.fallback_only_count = fallback_only_count;
    }
}

#[derive(Clone, Debug, Default)]
struct AssetCatalogStats {
    road_count: usize,
    building_count: usize,
    prop_count: usize,
    anchor_count: usize,
}

impl AssetCatalogStats {
    fn from_entries(entries: &[AssetCatalogEntry]) -> Self {
        let mut stats = Self::default();
        let mut _terrain_count = 0usize;
        let mut _zone_typed_count = 0usize;
        let mut _tag_count = 0usize;
        let mut _material_variant_count = 0usize;
        let mut _low_quality_rule_count = 0usize;
        let mut _total_footprint_area = 0.0f32;
        let mut _max_lod_distance = 0.0f32;
        let mut _landmark_count = 0usize;
        let mut _max_anchor_height = 0.0f32;

        for entry in entries {
            match entry.family {
                CatalogFamily::TerrainMaterial => _terrain_count += 1,
                CatalogFamily::RoadPiece => stats.road_count += 1,
                CatalogFamily::BuildingFamily => stats.building_count += 1,
                CatalogFamily::PropFamily => stats.prop_count += 1,
            }

            if entry.zone_class.is_some() {
                _zone_typed_count += 1;
            }

            _tag_count += entry.tags.len();
            stats.anchor_count += entry.prop_anchors.len();
            _material_variant_count += entry.material_variants.len();
            _total_footprint_area += entry.footprint[0] * entry.footprint[1];
            _max_lod_distance = _max_lod_distance.max(entry.lod_distances[2]);
            _low_quality_rule_count += usize::from(
                !entry.low_quality_fallback.mesh.is_empty()
                    && !entry.low_quality_fallback.material.is_empty(),
            );

            if matches!(entry.height_class, HeightClass::Landmark) {
                _landmark_count += 1;
            }

            for anchor in &entry.prop_anchors {
                if !anchor.name.is_empty() {
                    _max_anchor_height = _max_anchor_height.max(anchor.offset[1]);
                }
            }
        }

        stats
    }
}

#[derive(Clone, Debug, Deserialize)]
struct AssetCatalogManifest {
    version: u32,
    entries: Vec<AssetCatalogEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AssetCatalogEntry {
    pub id: String,
    pub family: CatalogFamily,
    pub zone_class: Option<CatalogZoneClass>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub footprint: [f32; 2],
    pub height_class: HeightClass,
    pub lod_distances: [f32; 3],
    pub low_quality_fallback: FallbackRule,
    #[serde(default)]
    pub prop_anchors: Vec<PropAnchor>,
    #[serde(default)]
    pub material_variants: Vec<String>,
    #[serde(default)]
    pub scene: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum CatalogFamily {
    TerrainMaterial,
    RoadPiece,
    BuildingFamily,
    PropFamily,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum CatalogZoneClass {
    Residential,
    MixedUse,
    Industrial,
    Utility,
    Civic,
    Streetscape,
    Nature,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum HeightClass {
    Surface,
    LowRise,
    MidRise,
    Landmark,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FallbackRule {
    pub mesh: String,
    pub material: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PropAnchor {
    pub name: String,
    pub offset: [f32; 3],
}

pub fn load_asset_catalog(
    asset_server: Res<AssetServer>,
    mut asset_catalog: ResMut<AssetCatalog>,
    mut game: ResMut<GameState>,
) {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let catalog_path = asset_root.join(ASSET_CATALOG_RELATIVE_PATH);
    let catalog_text = match fs::read_to_string(&catalog_path) {
        Ok(text) => text,
        Err(error) => {
            warn!(
                "Unable to read asset catalog at {}: {error}",
                catalog_path.display()
            );
            game.push_log("Asset catalog missing; using hardcoded blockout visuals.".to_string());
            return;
        }
    };

    let manifest = match ron::de::from_str::<AssetCatalogManifest>(&catalog_text) {
        Ok(manifest) => manifest,
        Err(error) => {
            warn!(
                "Unable to parse asset catalog at {}: {error}",
                catalog_path.display()
            );
            game.push_log(
                "Asset catalog parse failed; using hardcoded blockout visuals.".to_string(),
            );
            return;
        }
    };

    let mut scene_handles = HashMap::with_capacity(manifest.entries.len());
    let mut ready_scene_count = 0;
    let mut missing_scene_count = 0;
    let mut fallback_only_count = 0;

    for entry in &manifest.entries {
        match entry.scene.as_deref() {
            Some(scene_path) => {
                let source_path = scene_path.split('#').next().unwrap_or(scene_path);
                if asset_root.join(source_path).exists() {
                    scene_handles
                        .insert(entry.id.clone(), asset_server.load(scene_path.to_string()));
                    ready_scene_count += 1;
                } else {
                    missing_scene_count += 1;
                }
            }
            None => fallback_only_count += 1,
        }
    }

    asset_catalog.replace_from_manifest(
        manifest,
        scene_handles,
        ready_scene_count,
        missing_scene_count,
        fallback_only_count,
    );

    game.push_log(format!(
        "Asset pipeline ready. {}",
        asset_catalog.summary_line()
    ));
}
