use super::{
    ActionRailAction, ActionRailButton, ActionRailButtonText, ActionSidebarDetailsPanel,
    ActionSidebarExpandedOnly, ActionSidebarHelpText, ActionSidebarRoot, ActionSidebarScrollArea,
    ActionSidebarToggleButton, ActionSidebarToggleText, CameraRig, CloseEventModalButton,
    EventIconButton, EventIconLabel, EventModalRoot, EventModalText, EventStripStatusText,
    GroundPlane, HoverMarker, HudText, MainCamera, MinimapLegendText, MinimapMode, MinimapTileCell,
    MinimapViewButton, SelectionMarker, SettingsText, TopBarClockButton, TopBarClockText,
    TopBarMetricsText, TopBarSpeedButton, TopBarSpeedText, ACTION_BAR_TOP,
    ACTION_BAR_WIDTH_EXPANDED, MAX_EVENT_BUTTONS, MINIMAP_CELL_SIZE, MINIMAP_GRID_HEIGHT,
    MINIMAP_GRID_WIDTH, MINIMAP_PANEL_HEIGHT, MINIMAP_PANEL_WIDTH, SETTINGS_PANEL_HEIGHT,
    SETTINGS_PANEL_TOP, TOP_BAR_HEIGHT,
};
use crate::constants::{GRID_H, GRID_W, HUD_MARGIN, HUD_PANEL_W, TILE_WORLD_SIZE};
use crate::flat_material::FlatMaterial;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FlatMaterial>>,
    camera_rig: Res<CameraRig>,
) {
    let world_w = GRID_W as f32 * TILE_WORLD_SIZE + TILE_WORLD_SIZE * 2.0;
    let world_h = GRID_H as f32 * TILE_WORLD_SIZE + TILE_WORLD_SIZE * 2.0;
    let terrain_w = world_w + TILE_WORLD_SIZE * 18.0;
    let terrain_h = world_h + TILE_WORLD_SIZE * 18.0;
    let backdrop_height = TILE_WORLD_SIZE * 4.8;
    let backdrop_thickness = TILE_WORLD_SIZE * 2.0;
    let backdrop_y = backdrop_height * 0.5 - 1.6;

    commands.spawn((
        Camera3d {
            screen_space_specular_transmission_steps: 0,
            ..default()
        },
        DistanceFog {
            color: Color::srgb(0.26, 0.31, 0.38),
            directional_light_color: Color::srgba(0.98, 0.94, 0.86, 0.35),
            directional_light_exponent: 18.0,
            falloff: FogFalloff::Linear {
                start: 128.0,
                end: 192.0,
            },
        },
        camera_rig.camera_transform(),
        MainCamera,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(terrain_w, 0.18, terrain_h))),
        MeshMaterial3d(materials.add(FlatMaterial::opaque(Color::srgb(0.08, 0.11, 0.09)))),
        Transform::from_xyz(0.0, -0.54, 0.0),
        NotShadowCaster,
        GroundPlane,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(world_w, 0.06, world_h))),
        MeshMaterial3d(materials.add(FlatMaterial::opaque(Color::srgb(0.12, 0.16, 0.13)))),
        Transform::from_xyz(0.0, -0.18, 0.0),
        NotShadowCaster,
        GroundPlane,
    ));

    for (size, translation, color) in [
        (
            Vec3::new(terrain_w, backdrop_height, backdrop_thickness),
            Vec3::new(
                0.0,
                backdrop_y,
                terrain_h * 0.5 + backdrop_thickness * 0.5 - TILE_WORLD_SIZE,
            ),
            Color::srgb(0.06, 0.08, 0.11),
        ),
        (
            Vec3::new(terrain_w, backdrop_height, backdrop_thickness),
            Vec3::new(
                0.0,
                backdrop_y,
                -terrain_h * 0.5 - backdrop_thickness * 0.5 + TILE_WORLD_SIZE,
            ),
            Color::srgb(0.06, 0.08, 0.11),
        ),
        (
            Vec3::new(backdrop_thickness, backdrop_height, terrain_h),
            Vec3::new(
                terrain_w * 0.5 + backdrop_thickness * 0.5 - TILE_WORLD_SIZE,
                backdrop_y,
                0.0,
            ),
            Color::srgb(0.05, 0.07, 0.10),
        ),
        (
            Vec3::new(backdrop_thickness, backdrop_height, terrain_h),
            Vec3::new(
                -terrain_w * 0.5 - backdrop_thickness * 0.5 + TILE_WORLD_SIZE,
                backdrop_y,
                0.0,
            ),
            Color::srgb(0.05, 0.07, 0.10),
        ),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(FlatMaterial::opaque(color))),
            Transform::from_translation(translation),
            NotShadowCaster,
            GroundPlane,
        ));
    }

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            TILE_WORLD_SIZE * 0.94,
            0.06,
            TILE_WORLD_SIZE * 0.94,
        ))),
        MeshMaterial3d(materials.add(FlatMaterial::transparent(Color::srgba(
            1.0, 0.94, 0.22, 0.55,
        )))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        NotShadowCaster,
        Visibility::Hidden,
        HoverMarker,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            TILE_WORLD_SIZE * 0.84,
            0.06,
            TILE_WORLD_SIZE * 0.84,
        ))),
        MeshMaterial3d(materials.add(FlatMaterial::transparent(Color::srgba(
            0.18, 1.0, 0.40, 0.55,
        )))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        NotShadowCaster,
        Visibility::Hidden,
        SelectionMarker,
    ));
}

pub fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                height: Val::Px(TOP_BAR_HEIGHT),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.05, 0.08, 0.96)),
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            min_width: Val::Px(190.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("SMARTCITY FORGE"),
                                TextFont {
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.94, 0.97, 1.0)),
                            ));
                            parent.spawn((
                                Text::new("Command center // live city operations"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.62, 0.74, 0.84)),
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(176.0),
                                height: Val::Px(56.0),
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                row_gap: Val::Px(2.0),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(18.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.08, 0.12, 0.17, 0.96)),
                            BorderColor::all(Color::srgba(0.28, 0.46, 0.58, 0.62)),
                            TopBarClockButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("SIM TIME"),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.58, 0.76, 0.84)),
                            ));
                            parent.spawn((
                                Text::new("loading date and time..."),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                TopBarClockText,
                            ));
                        });

                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(108.0),
                                height: Val::Px(56.0),
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                row_gap: Val::Px(2.0),
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(18.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.08, 0.12, 0.17, 0.96)),
                            BorderColor::all(Color::srgba(0.28, 0.46, 0.58, 0.62)),
                            TopBarSpeedButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("SPEED"),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.58, 0.76, 0.84)),
                            ));
                            parent.spawn((
                                Text::new("1x"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                TopBarSpeedText,
                            ));
                        });
                });

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        max_width: Val::Px(760.0),
                        min_width: Val::Px(300.0),
                        height: Val::Px(62.0),
                        margin: UiRect::axes(Val::Px(18.0), Val::Px(0.0)),
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(9.0)),
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(4.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.07, 0.11, 0.15, 0.94)),
                    BorderColor::all(Color::srgba(0.32, 0.48, 0.58, 0.48)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("CITY SNAPSHOT"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.56, 0.74, 0.82)),
                    ));
                    parent.spawn((
                        Text::new("loading metrics..."),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.94, 0.97, 1.0)),
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        TopBarMetricsText,
                    ));
                });

            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(6.0),
                    ..default()
                },))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Events stable"),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.78, 0.83, 0.92)),
                        EventStripStatusText,
                    ));

                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            for slot in 0..MAX_EVENT_BUTTONS {
                                parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(74.0),
                                            height: Val::Px(30.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            border_radius: BorderRadius::MAX,
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.16, 0.18, 0.22, 0.75)),
                                        BorderColor::all(Color::srgba(0.95, 0.97, 1.0, 0.10)),
                                        Visibility::Hidden,
                                        EventIconButton { slot },
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Text::new(""),
                                            TextFont {
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                            EventIconLabel { slot },
                                        ));
                                    });
                            }
                        });
                });
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(HUD_MARGIN),
                top: Val::Px(ACTION_BAR_TOP),
                bottom: Val::Px(HUD_MARGIN),
                width: Val::Px(ACTION_BAR_WIDTH_EXPANDED),
                padding: UiRect::all(Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(26.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.08, 0.11, 0.92)),
            BorderColor::all(Color::srgba(0.34, 0.50, 0.60, 0.42)),
            ActionSidebarRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(42.0),
                                height: Val::Px(42.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                border_radius: BorderRadius::all(Val::Px(14.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.10, 0.16, 0.22, 0.96)),
                            BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.36)),
                            ActionSidebarToggleButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("<<"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                ActionSidebarToggleText,
                            ));
                        });

                    parent
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(2.0),
                                ..default()
                            },
                            ActionSidebarExpandedOnly,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("ACTION RAIL"),
                                TextFont {
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.94, 0.97, 1.0)),
                            ));
                            parent.spawn((
                                Text::new("Build, inspect, and tune the city"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.58, 0.74, 0.82)),
                            ));
                        });
                });

            parent
                .spawn((
                    Node {
                        flex_grow: 1.0,
                        min_height: Val::Px(0.0),
                        overflow: Overflow::scroll_y(),
                        padding: UiRect::right(Val::Px(4.0)),
                        ..default()
                    },
                    ScrollPosition::default(),
                    ActionSidebarScrollArea,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(12.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("TOOLS"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.54, 0.72, 0.82)),
                                ActionSidebarExpandedOnly,
                            ));

                            for action in ActionRailAction::TOOLS {
                                parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(40.0),
                                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            border_radius: BorderRadius::all(Val::Px(16.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.10, 0.16, 0.22, 0.96)),
                                        BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.12)),
                                        ActionRailButton { action },
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Text::new("..."),
                                            TextFont {
                                                font_size: 13.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                            ActionRailButtonText { action },
                                        ));
                                    });
                            }

                            parent.spawn((
                                Text::new("SYSTEM"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.54, 0.72, 0.82)),
                                ActionSidebarExpandedOnly,
                            ));

                            for action in ActionRailAction::SYSTEMS {
                                parent
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(40.0),
                                            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            border_radius: BorderRadius::all(Val::Px(16.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.10, 0.16, 0.22, 0.96)),
                                        BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.12)),
                                        ActionRailButton { action },
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            Text::new("..."),
                                            TextFont {
                                                font_size: 13.0,
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                            ActionRailButtonText { action },
                                        ));
                                    });
                            }

                            parent
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::all(Val::Px(12.0)),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(8.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(18.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.07, 0.11, 0.16, 0.96)),
                                    BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.16)),
                                    ActionSidebarExpandedOnly,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("HELP"),
                                        TextFont {
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.56, 0.74, 0.82)),
                                    ));
                                    parent.spawn((
                                        Text::new("loading help..."),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.94, 0.97, 1.0)),
                                        ActionSidebarHelpText,
                                    ));
                                });

                            parent
                                .spawn((
                                    Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::all(Val::Px(12.0)),
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(8.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(18.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.08, 0.12, 0.17, 0.96)),
                                    BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.16)),
                                    ActionSidebarDetailsPanel,
                                    ActionSidebarExpandedOnly,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("CITY CONTEXT"),
                                        TextFont {
                                            font_size: 11.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.56, 0.74, 0.82)),
                                    ));
                                    parent.spawn((
                                        Text::new("loading city panel..."),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                        HudText,
                                    ));
                                });
                        });
                });
        });

    commands.spawn((
        Text::new("opening configuration..."),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(HUD_MARGIN),
            top: Val::Px(SETTINGS_PANEL_TOP),
            width: Val::Px(HUD_PANEL_W + 40.0),
            height: Val::Px(SETTINGS_PANEL_HEIGHT),
            padding: UiRect::all(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(24.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.08, 0.11, 0.94)),
        BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.22)),
        Visibility::Hidden,
        SettingsText,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(HUD_MARGIN),
                bottom: Val::Px(HUD_MARGIN),
                width: Val::Px(MINIMAP_PANEL_WIDTH),
                height: Val::Px(MINIMAP_PANEL_HEIGHT),
                padding: UiRect::all(Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(26.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.11, 0.92)),
            BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.22)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("MINIMAP // REAL VIEW"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MinimapLegendText,
            ));

            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },))
                .with_children(|parent| {
                    for mode in MinimapMode::ALL {
                        parent
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Px(58.0),
                                    height: Val::Px(28.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.16, 0.18, 0.22, 0.72)),
                                BorderColor::all(Color::srgba(0.90, 0.96, 1.0, 0.12)),
                                MinimapViewButton { mode },
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(mode.short_label()),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Px(MINIMAP_GRID_WIDTH),
                        height: Val::Px(MINIMAP_GRID_HEIGHT),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(18.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.025, 0.035, 0.95)),
                    BorderColor::all(Color::srgba(0.90, 0.96, 1.0, 0.08)),
                ))
                .with_children(|parent| {
                    for y in 0..GRID_H {
                        for x in 0..GRID_W {
                            parent.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(x as f32 * MINIMAP_CELL_SIZE),
                                    top: Val::Px(y as f32 * MINIMAP_CELL_SIZE),
                                    width: Val::Px(MINIMAP_CELL_SIZE - 1.0),
                                    height: Val::Px(MINIMAP_CELL_SIZE - 1.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.18, 0.20, 0.24)),
                                MinimapTileCell {
                                    pos: IVec2::new(x, y),
                                },
                            ));
                        }
                    }
                });
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.015, 0.02, 0.72)),
            Visibility::Hidden,
            EventModalRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(460.0),
                        padding: UiRect::all(Val::Px(18.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(24.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.08, 0.11, 0.98)),
                    BorderColor::all(Color::srgba(0.50, 0.72, 0.80, 0.18)),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            width: Val::Percent(100.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("EVENT"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.88, 0.92, 0.98)),
                            ));
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(34.0),
                                        height: Val::Px(34.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.18, 0.20, 0.24, 0.9)),
                                    CloseEventModalButton,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("X"),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    parent.spawn((
                        Text::new("waiting for event data..."),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        EventModalText,
                    ));
                });
        });
}
