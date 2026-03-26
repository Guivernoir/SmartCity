use super::blockout::{spawn_city_terrain, tile_blockout_parts};
use super::helpers::{
    asset_style, spawn_asset_visual, spawn_tile_visual, tile_to_world, tile_top_y,
};
use super::props::asset_prop_parts;
use super::{
    CityVisual, GroundPlane, HoverMarker, HoverState, RenderAssetCache, SceneSyncState,
    SelectionMarker,
};
use crate::game::GameState;
use crate::settings::{ActiveRenderTier, GraphicsSettings, MeshDetail, RenderCapability};
use bevy::gizmos::config::{DefaultGizmoConfigGroup, GizmoConfigStore};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub fn configure_gizmos(
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    if !graphics_settings.is_changed() && !render_capability.is_changed() {
        return;
    }

    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line.width = graphics_settings.gizmo_line_width();
    config.depth_bias = if matches!(
        graphics_settings.active_render_tier(*render_capability),
        ActiveRenderTier::FallbackDebug
    ) {
        -1.0
    } else {
        0.0
    };
}

pub fn apply_graphics_settings(
    graphics_settings: Res<GraphicsSettings>,
    mut ground_planes: Query<&mut Visibility, With<GroundPlane>>,
) {
    if !graphics_settings.is_changed() {
        return;
    }

    let visibility = if graphics_settings.show_ground {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    for mut ground_visibility in &mut ground_planes {
        *ground_visibility = visibility;
    }
}

pub fn rebuild_city_visuals(
    mut commands: Commands,
    game: Res<GameState>,
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    mut sync_state: ResMut<SceneSyncState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut render_assets: ResMut<RenderAssetCache>,
    existing_visuals: Query<Entity, With<CityVisual>>,
) {
    if sync_state.rendered_visual_revision == game.visual_revision()
        && !graphics_settings.is_changed()
    {
        return;
    }

    for entity in &existing_visuals {
        commands.entity(entity).despawn();
    }

    let active_render_tier = graphics_settings.active_render_tier(*render_capability);
    if !active_render_tier.uses_mesh_renderer() {
        sync_state.rendered_visual_revision = game.visual_revision();
        return;
    }

    spawn_city_terrain(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut render_assets,
        &game,
        active_render_tier,
    );

    for y in 0..crate::constants::GRID_H {
        for x in 0..crate::constants::GRID_W {
            let pos = IVec2::new(x, y);
            let tile = game.tile(pos);
            spawn_tile_visual(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut render_assets,
                &graphics_settings,
                *render_capability,
                &game,
                pos,
                tile.kind,
            );

            if let Some(asset_idx) = tile.asset {
                let asset = &game.assets[asset_idx];
                spawn_asset_visual(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut render_assets,
                    &graphics_settings,
                    *render_capability,
                    asset,
                    tile.kind,
                );
            }
        }
    }

    sync_state.rendered_visual_revision = game.visual_revision();
}

pub fn draw_city_gizmos(
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    game: Res<GameState>,
    hover_state: Res<HoverState>,
    mut gizmos: Gizmos,
) {
    if !matches!(
        graphics_settings.active_render_tier(*render_capability),
        ActiveRenderTier::FallbackDebug
    ) {
        return;
    }

    let simple_debug = matches!(
        graphics_settings.effective_mesh_detail(*render_capability),
        MeshDetail::Low
    );
    let mesh_detail = graphics_settings.effective_mesh_detail(*render_capability);
    let prop_density = graphics_settings.effective_prop_density(*render_capability);

    for y in 0..crate::constants::GRID_H {
        for x in 0..crate::constants::GRID_W {
            let pos = IVec2::new(x, y);
            let tile = game.tile(pos);
            for part in tile_blockout_parts(&game, pos, tile.kind, mesh_detail, prop_density) {
                let center = tile_to_world(pos, 0.0) + part.offset;
                if simple_debug && part.size.y <= 0.12 {
                    gizmos.rect(
                        Isometry3d::new(
                            center + Vec3::Y * (part.size.y * 0.5 + 0.01),
                            Quat::from_rotation_x(FRAC_PI_2),
                        ),
                        Vec2::new(part.size.x, part.size.z),
                        part.color,
                    );
                } else {
                    gizmos.cube(
                        Transform::from_translation(center).with_scale(part.size),
                        part.color,
                    );
                }
            }

            if let Some(asset_idx) = tile.asset {
                let asset = &game.assets[asset_idx];
                let (size, asset_color) = asset_style(asset.kind);
                let asset_y = tile_top_y(tile.kind) + size.y * 0.5 + 0.05;

                if simple_debug {
                    gizmos.rect(
                        Isometry3d::new(
                            tile_to_world(asset.pos, tile_top_y(tile.kind) + 0.10),
                            Quat::from_rotation_x(FRAC_PI_2),
                        ),
                        Vec2::new(size.x.max(0.34), size.z.max(0.34)),
                        asset_color,
                    );
                } else {
                    gizmos.cube(
                        Transform::from_translation(tile_to_world(asset.pos, asset_y))
                            .with_scale(size),
                        asset_color,
                    );

                    if !asset.compliant {
                        gizmos.cube(
                            Transform::from_translation(tile_to_world(
                                asset.pos,
                                asset_y + size.y * 0.5 + 0.45,
                            ))
                            .with_scale(Vec3::new(0.28, 0.9, 0.28)),
                            Color::srgb(0.96, 0.18, 0.20),
                        );
                    }
                }

                if !asset.compliant && simple_debug {
                    gizmos.rect(
                        Isometry3d::new(
                            tile_to_world(asset.pos, tile_top_y(tile.kind) + 0.18),
                            Quat::from_rotation_x(FRAC_PI_2),
                        ),
                        Vec2::new(size.x.max(0.42), size.z.max(0.42)) + Vec2::splat(0.16),
                        Color::srgb(0.96, 0.18, 0.20),
                    );
                }

                for part in asset_prop_parts(asset, size, mesh_detail, prop_density) {
                    let center = tile_to_world(asset.pos, asset_y) + part.offset;
                    if simple_debug && part.size.y <= 0.12 {
                        gizmos.rect(
                            Isometry3d::new(
                                center + Vec3::Y * (part.size.y * 0.5 + 0.01),
                                Quat::from_rotation_x(FRAC_PI_2),
                            ),
                            Vec2::new(part.size.x.max(0.08), part.size.z.max(0.08)),
                            part.color,
                        );
                    } else {
                        gizmos.cube(
                            Transform::from_translation(center).with_scale(part.size),
                            part.color,
                        );
                    }
                }
            }
        }
    }

    if graphics_settings.show_markers {
        if let Some(tile) = hover_state.hovered_tile {
            gizmos.rect(
                Isometry3d::new(
                    tile_to_world(tile, tile_top_y(game.tile(tile).kind) + 0.11),
                    Quat::from_rotation_x(FRAC_PI_2),
                ),
                Vec2::splat(crate::constants::TILE_WORLD_SIZE * 0.92),
                Color::srgb(1.0, 0.95, 0.2),
            );
        }

        if let Some(tile) = game.selected_tile {
            gizmos.rect(
                Isometry3d::new(
                    tile_to_world(tile, tile_top_y(game.tile(tile).kind) + 0.17),
                    Quat::from_rotation_x(FRAC_PI_2),
                ),
                Vec2::splat(crate::constants::TILE_WORLD_SIZE * 0.82),
                Color::srgb(0.18, 1.0, 0.4),
            );
        }
    }
}

pub fn update_tile_markers(
    game: Res<GameState>,
    hover_state: Res<HoverState>,
    graphics_settings: Res<GraphicsSettings>,
    render_capability: Res<RenderCapability>,
    hover_marker: Single<
        (&mut Transform, &mut Visibility),
        (With<HoverMarker>, Without<SelectionMarker>),
    >,
    selection_marker: Single<
        (&mut Transform, &mut Visibility),
        (With<SelectionMarker>, Without<HoverMarker>),
    >,
) {
    let (mut hover_transform, mut hover_visibility) = hover_marker.into_inner();
    let hide_markers = !graphics_settings.show_markers
        || matches!(
            graphics_settings.active_render_tier(*render_capability),
            ActiveRenderTier::FallbackDebug
        );

    if hide_markers {
        *hover_visibility = Visibility::Hidden;
    } else if let Some(tile) = hover_state.hovered_tile {
        let y = tile_top_y(game.tile(tile).kind) + 0.07;
        hover_transform.translation = tile_to_world(tile, y);
        *hover_visibility = Visibility::Visible;
    } else {
        *hover_visibility = Visibility::Hidden;
    }

    let (mut selection_transform, mut selection_visibility) = selection_marker.into_inner();
    if hide_markers {
        *selection_visibility = Visibility::Hidden;
    } else if let Some(tile) = game.selected_tile {
        let y = tile_top_y(game.tile(tile).kind) + 0.12;
        selection_transform.translation = tile_to_world(tile, y);
        *selection_visibility = Visibility::Visible;
    } else {
        *selection_visibility = Visibility::Hidden;
    }
}
