use crate::constants::{GRID_H, GRID_W, HUD_MARGIN};
use crate::model::Tool;
use bevy::prelude::*;

mod atmosphere;
mod blockout;
mod camera;
mod districts;
mod helpers;
mod input;
mod materials;
mod props;
mod render;
mod setup;
mod update;

pub use atmosphere::{setup_atmosphere, sync_atmosphere};
pub use camera::{handle_camera_input, update_camera_transform};
pub use input::{
    advance_simulation, handle_keyboard_input, handle_mouse_clicks, handle_ui_buttons,
    scroll_action_sidebar, update_hover_tile,
};
pub use materials::RenderAssetCache;
pub use render::{
    apply_graphics_settings, configure_gizmos, draw_city_gizmos, rebuild_city_visuals,
    update_tile_markers,
};
pub use setup::{setup_hud, setup_scene};
pub use update::{
    update_action_rail_button_labels, update_action_rail_button_styles,
    update_action_sidebar_help_text, update_action_sidebar_layout, update_event_badge_buttons,
    update_event_badge_labels, update_event_modal_panel, update_event_strip_status_text,
    update_hud_panel, update_minimap_button_styles, update_minimap_cell_colors,
    update_minimap_legend_text, update_settings_panel, update_top_bar_clock_text,
    update_top_bar_control_styles, update_top_bar_metrics_text, update_top_bar_speed_text,
};

pub(super) const TOP_BAR_HEIGHT: f32 = 88.0;
pub(super) const ACTION_BAR_TOP: f32 = TOP_BAR_HEIGHT + HUD_MARGIN;
pub(super) const ACTION_BAR_WIDTH_EXPANDED: f32 = 304.0;
pub(super) const ACTION_BAR_WIDTH_COLLAPSED: f32 = 92.0;
pub(super) const SETTINGS_PANEL_TOP: f32 = TOP_BAR_HEIGHT + HUD_MARGIN;
pub(super) const SETTINGS_PANEL_HEIGHT: f32 = 312.0;
pub(super) const MINIMAP_PANEL_WIDTH: f32 = 356.0;
pub(super) const MINIMAP_PANEL_HEIGHT: f32 = 286.0;
pub(super) const MINIMAP_CELL_SIZE: f32 = 10.0;
pub(super) const MINIMAP_GRID_WIDTH: f32 = GRID_W as f32 * MINIMAP_CELL_SIZE;
pub(super) const MINIMAP_GRID_HEIGHT: f32 = GRID_H as f32 * MINIMAP_CELL_SIZE;
pub(super) const MAX_EVENT_BUTTONS: usize = 4;

#[derive(Resource, Default)]
pub struct HoverState {
    pub hovered_tile: Option<IVec2>,
}

#[derive(Resource, Default)]
pub struct SceneSyncState {
    pub rendered_visual_revision: u64,
}

#[derive(Resource, Default)]
pub struct ActionSidebarState {
    pub collapsed: bool,
}

#[derive(Resource, Clone, Debug)]
pub struct CameraRig {
    pub target_pivot: Vec3,
    pub current_pivot: Vec3,
    pub target_yaw: f32,
    pub current_yaw: f32,
    pub target_pitch: f32,
    pub current_pitch: f32,
    pub target_distance: f32,
    pub current_distance: f32,
    pub preset: CameraPreset,
}

#[derive(Resource)]
pub struct TimeControl {
    paused: bool,
    speed_index: usize,
    accumulator: f32,
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MinimapMode {
    #[default]
    Real,
    Services,
    Electrical,
    Water,
    Risk,
}

impl MinimapMode {
    pub const ALL: [Self; 5] = [
        Self::Real,
        Self::Services,
        Self::Electrical,
        Self::Water,
        Self::Risk,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Real => "Real View",
            Self::Services => "Services Coverage",
            Self::Electrical => "Electrical",
            Self::Water => "Water",
            Self::Risk => "Risk",
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Real => "REAL",
            Self::Services => "SERV",
            Self::Electrical => "ELEC",
            Self::Water => "H2O",
            Self::Risk => "RISK",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CameraPreset {
    #[default]
    Strategic,
    CityView,
    Cinematic,
}

impl CameraPreset {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Strategic => "Strategic",
            Self::CityView => "City View",
            Self::Cinematic => "Cinematic",
        }
    }
}

#[derive(Resource, Default)]
pub struct EventModalState {
    pub selected_incident: Option<usize>,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self::for_preset(CameraPreset::Strategic)
    }
}

impl CameraRig {
    pub fn for_preset(preset: CameraPreset) -> Self {
        let mut rig = Self {
            target_pivot: Vec3::ZERO,
            current_pivot: Vec3::ZERO,
            target_yaw: 0.0,
            current_yaw: 0.0,
            target_pitch: 0.0,
            current_pitch: 0.0,
            target_distance: 0.0,
            current_distance: 0.0,
            preset,
        };
        rig.apply_preset(preset, true);
        rig
    }

    pub fn apply_preset(&mut self, preset: CameraPreset, snap: bool) {
        self.preset = preset;

        let (distance, yaw, pitch) = match preset {
            CameraPreset::Strategic => (88.0, 0.72, -0.98),
            CameraPreset::CityView => (62.0, 0.84, -0.76),
            CameraPreset::Cinematic => (38.0, 0.98, -0.50),
        };

        self.target_distance = distance;
        self.target_yaw = yaw;
        self.target_pitch = pitch;

        if snap {
            self.current_distance = distance;
            self.current_yaw = yaw;
            self.current_pitch = pitch;
        }
    }

    pub fn camera_transform(&self) -> Transform {
        let elevation = -self.current_pitch;
        let ground_distance = self.current_distance * elevation.cos();
        let offset = Vec3::new(
            ground_distance * self.current_yaw.sin(),
            self.current_distance * elevation.sin(),
            ground_distance * self.current_yaw.cos(),
        );

        Transform::from_translation(self.current_pivot + offset)
            .looking_at(self.current_pivot, Vec3::Y)
    }
}

impl Default for TimeControl {
    fn default() -> Self {
        Self {
            paused: false,
            speed_index: 0,
            accumulator: 0.0,
        }
    }
}

impl TimeControl {
    const SPEEDS: [u32; 4] = [1, 2, 4, 8];

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn slower(&mut self) {
        if self.speed_index > 0 {
            self.speed_index -= 1;
        }
    }

    pub fn faster(&mut self) {
        if self.speed_index + 1 < Self::SPEEDS.len() {
            self.speed_index += 1;
        }
    }

    pub fn cycle_speed(&mut self) {
        self.speed_index = (self.speed_index + 1) % Self::SPEEDS.len();
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn speed_multiplier(&self) -> f32 {
        Self::SPEEDS[self.speed_index] as f32
    }

    pub fn speed_label(&self) -> String {
        if self.paused {
            format!("Paused @ {}x", Self::SPEEDS[self.speed_index])
        } else {
            format!("{}x", Self::SPEEDS[self.speed_index])
        }
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CityVisual;

#[derive(Component)]
pub struct CitySkyShell;

#[derive(Component)]
pub struct SunLight;

#[derive(Component)]
pub struct HoverMarker;

#[derive(Component)]
pub struct SelectionMarker;

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct SettingsText;

#[derive(Component)]
pub struct GroundPlane;

#[derive(Component)]
pub struct TopBarClockButton;

#[derive(Component)]
pub struct TopBarClockText;

#[derive(Component)]
pub struct TopBarSpeedButton;

#[derive(Component)]
pub struct TopBarSpeedText;

#[derive(Component)]
pub struct TopBarMetricsText;

#[derive(Component)]
pub struct EventStripStatusText;

#[derive(Component)]
pub struct EventIconButton {
    pub slot: usize,
}

#[derive(Component)]
pub struct EventIconLabel {
    pub slot: usize,
}

#[derive(Component)]
pub struct MinimapLegendText;

#[derive(Component)]
pub struct MinimapViewButton {
    pub mode: MinimapMode,
}

#[derive(Component)]
pub struct MinimapTileCell {
    pub pos: IVec2,
}

#[derive(Component)]
pub struct EventModalRoot;

#[derive(Component)]
pub struct EventModalText;

#[derive(Component)]
pub struct CloseEventModalButton;

#[derive(Component)]
pub struct ActionSidebarRoot;

#[derive(Component)]
pub struct ActionSidebarScrollArea;

#[derive(Component)]
pub struct ActionSidebarExpandedOnly;

#[derive(Component)]
pub struct ActionSidebarToggleButton;

#[derive(Component)]
pub struct ActionSidebarToggleText;

#[derive(Component)]
pub struct ActionSidebarDetailsPanel;

#[derive(Component)]
pub struct ActionSidebarHelpText;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActionRailAction {
    Tool(Tool),
    Regenerate,
    CycleBridgeProfile,
    CycleLogicProfile,
    SpendResearch,
}

impl ActionRailAction {
    pub(super) const TOOLS: [Self; 7] = [
        Self::Tool(Tool::Inspect),
        Self::Tool(Tool::Bridge),
        Self::Tool(Tool::Sensor),
        Self::Tool(Tool::Plc),
        Self::Tool(Tool::Gateway),
        Self::Tool(Tool::Substation),
        Self::Tool(Tool::PumpStation),
    ];

    pub(super) const SYSTEMS: [Self; 4] = [
        Self::Regenerate,
        Self::CycleBridgeProfile,
        Self::CycleLogicProfile,
        Self::SpendResearch,
    ];
}

#[derive(Component)]
pub struct ActionRailButton {
    pub action: ActionRailAction,
}

#[derive(Component)]
pub struct ActionRailButtonText {
    pub action: ActionRailAction,
}
