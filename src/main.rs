mod asset_pipeline;
mod constants;
mod flat_material;
mod game;
mod model;
mod settings;
mod ui;

use crate::asset_pipeline::{load_asset_catalog, AssetCatalog};
use crate::constants::{WINDOW_H, WINDOW_W};
use crate::flat_material::FlatMaterial;
use crate::game::GameState;
use crate::settings::{GraphicsSettings, RenderCapability, SettingsMenuState};
use crate::ui::{
    advance_simulation, apply_graphics_settings, configure_gizmos, draw_city_gizmos,
    handle_camera_input, handle_keyboard_input, handle_mouse_clicks, handle_ui_buttons,
    rebuild_city_visuals, scroll_action_sidebar, setup_atmosphere, setup_hud, setup_scene,
    sync_atmosphere, update_action_rail_button_labels, update_action_rail_button_styles,
    update_action_sidebar_help_text, update_action_sidebar_layout, update_camera_transform,
    update_event_badge_buttons, update_event_badge_labels, update_event_modal_panel,
    update_event_strip_status_text, update_hover_tile, update_hud_panel,
    update_minimap_button_styles, update_minimap_cell_colors, update_minimap_legend_text,
    update_settings_panel, update_tile_markers, update_top_bar_clock_text,
    update_top_bar_control_styles, update_top_bar_metrics_text, update_top_bar_speed_text,
    ActionSidebarState, CameraRig, EventModalState, HoverState, MinimapMode, RenderAssetCache,
    SceneSyncState, TimeControl,
};
use bevy::asset::AssetPlugin;
use bevy::audio::AudioPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::pbr::{MaterialPlugin, PbrPlugin};
use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuLimits, WgpuSettings, WgpuSettingsPriority},
    RenderPlugin,
};
use bevy::window::WindowResolution;

fn main() {
    configure_runtime_overrides();
    let render_capability = RenderCapability::detect();
    let graphics_settings = GraphicsSettings::default_for(render_capability);

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.025, 0.03, 0.04)))
        .insert_resource(GameState::new())
        .insert_resource(AssetCatalog::default())
        .insert_resource(render_capability)
        .insert_resource(graphics_settings)
        .insert_resource(SettingsMenuState::default())
        .insert_resource(TimeControl::default())
        .insert_resource(HoverState::default())
        .insert_resource(ActionSidebarState::default())
        .insert_resource(CameraRig::default())
        .insert_resource(MinimapMode::default())
        .insert_resource(EventModalState::default())
        .insert_resource(SceneSyncState::default())
        .init_resource::<RenderAssetCache>()
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<AudioPlugin>()
                .set(LogPlugin {
                    filter: log_filter().to_string(),
                    level: Level::INFO,
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SmartCity Forge - Phase 2 3D".to_string(),
                        resolution: WindowResolution::new(WINDOW_W as u32, WINDOW_H as u32),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(PbrPlugin {
                    prepass_enabled: false,
                    add_default_deferred_lighting_plugin: false,
                    use_gpu_instance_buffer_builder: false,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(render_settings()),
                    ..default()
                }),
        )
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(MaterialPlugin::<FlatMaterial>::default())
        .add_systems(
            Startup,
            (setup_scene, setup_hud, configure_gizmos, load_asset_catalog),
        )
        .add_systems(
            Update,
            (
                (
                    handle_ui_buttons,
                    handle_keyboard_input,
                    scroll_action_sidebar,
                    handle_camera_input,
                    update_camera_transform,
                    update_hover_tile,
                    handle_mouse_clicks,
                    advance_simulation,
                )
                    .chain(),
                (
                    apply_graphics_settings,
                    rebuild_city_visuals,
                    update_tile_markers,
                    configure_gizmos,
                    draw_city_gizmos,
                ),
                (
                    update_top_bar_clock_text,
                    update_top_bar_speed_text,
                    update_top_bar_metrics_text,
                    update_top_bar_control_styles,
                    update_event_strip_status_text,
                    update_minimap_legend_text,
                    update_minimap_button_styles,
                    update_minimap_cell_colors,
                    update_event_badge_buttons,
                    update_event_badge_labels,
                    update_event_modal_panel,
                    update_action_sidebar_layout,
                    update_action_sidebar_help_text,
                    update_action_rail_button_styles,
                    update_action_rail_button_labels,
                    update_hud_panel,
                    update_settings_panel,
                ),
            ),
        );

    if !render_capability.software_fallback {
        app.add_systems(Startup, setup_atmosphere)
            .add_systems(Update, sync_atmosphere);
    }

    app.run();
}

fn configure_runtime_overrides() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", log_filter());
    }

    let force_software = matches!(
        std::env::var("SMARTCITY_FORCE_SOFTWARE_RENDERER").as_deref(),
        Ok("1" | "true" | "TRUE" | "yes" | "YES")
    );

    if !force_software {
        return;
    }

    if std::env::var_os("WGPU_BACKEND").is_none() {
        std::env::set_var("WGPU_BACKEND", "gl");
    }

    if cfg!(target_os = "linux") && std::env::var_os("LIBGL_ALWAYS_SOFTWARE").is_none() {
        std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
    }
}

fn render_settings() -> WgpuSettings {
    let mut settings = WgpuSettings::default();
    let using_gl_backend = matches!(
        std::env::var("WGPU_BACKEND").as_deref().map(str::to_ascii_lowercase),
        Ok(value) if value.split(',').any(|backend| matches!(backend.trim(), "gl" | "gles"))
    ) || cfg!(target_os = "linux") && std::env::var("WGPU_BACKEND").is_err();

    if cfg!(target_os = "linux") && std::env::var("WGPU_BACKEND").is_err() {
        settings.backends = Some(Backends::GL);
    }

    if using_gl_backend {
        // Treat native GL like a WebGL2-class target on this project so Bevy
        // avoids higher-end texture/view assumptions that old Mesa software
        // paths cannot honor cleanly.
        settings.priority = WgpuSettingsPriority::WebGL2;
        settings.constrained_limits = Some(WgpuLimits {
            max_storage_textures_per_shader_stage: 4,
            ..WgpuLimits::default()
        });
    }

    if matches!(
        std::env::var("SMARTCITY_FORCE_SOFTWARE_RENDERER").as_deref(),
        Ok("1" | "true" | "TRUE" | "yes" | "YES")
    ) || matches!(
        std::env::var("WGPU_FORCE_FALLBACK_ADAPTER").as_deref(),
        Ok("1" | "true" | "TRUE" | "yes" | "YES")
    ) {
        settings.force_fallback_adapter = true;
    }

    settings
}

fn log_filter() -> &'static str {
    "info,wgpu_hal::gles=off"
}
