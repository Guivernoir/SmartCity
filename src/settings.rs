use crate::constants::HUD_PANEL_W;
use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Debug)]
pub struct RenderCapability {
    pub software_fallback: bool,
}

impl RenderCapability {
    pub fn detect() -> Self {
        Self {
            software_fallback: matches!(
                std::env::var("SMARTCITY_FORCE_SOFTWARE_RENDERER").as_deref(),
                Ok("1" | "true" | "TRUE" | "yes" | "YES")
            ) || matches!(
                std::env::var("WGPU_FORCE_FALLBACK_ADAPTER").as_deref(),
                Ok("1" | "true" | "TRUE" | "yes" | "YES")
            ),
        }
    }

    pub const fn label(self) -> &'static str {
        if self.software_fallback {
            "Software Fallback"
        } else {
            "GPU Native"
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GraphicsPreset {
    Low,
    Balanced,
    High,
    Custom,
}

impl GraphicsPreset {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Balanced => "Balanced",
            Self::High => "High",
            Self::Custom => "Custom",
        }
    }

    pub const fn next_selectable(self) -> Self {
        match self {
            Self::Low => Self::Balanced,
            Self::Balanced => Self::High,
            Self::High => Self::Low,
            Self::Custom => Self::Low,
        }
    }

    pub const fn prev_selectable(self) -> Self {
        match self {
            Self::Low => Self::High,
            Self::Balanced => Self::Low,
            Self::High => Self::Balanced,
            Self::Custom => Self::High,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RenderTierChoice {
    Auto,
    FallbackDebug,
    ShippingLow,
    Full3d,
}

impl RenderTierChoice {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::FallbackDebug => "Fallback Debug",
            Self::ShippingLow => "Shipping Low",
            Self::Full3d => "Full 3D",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Auto => Self::FallbackDebug,
            Self::FallbackDebug => Self::ShippingLow,
            Self::ShippingLow => Self::Full3d,
            Self::Full3d => Self::Auto,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Auto => Self::Full3d,
            Self::FallbackDebug => Self::Auto,
            Self::ShippingLow => Self::FallbackDebug,
            Self::Full3d => Self::ShippingLow,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveRenderTier {
    FallbackDebug,
    ShippingLow,
    Full3d,
}

impl ActiveRenderTier {
    pub const fn label(self) -> &'static str {
        match self {
            Self::FallbackDebug => "Fallback Debug",
            Self::ShippingLow => "Shipping Low",
            Self::Full3d => "Full 3D",
        }
    }

    pub const fn uses_mesh_renderer(self) -> bool {
        !matches!(self, Self::FallbackDebug)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeshDetail {
    Low,
    Medium,
    High,
}

impl MeshDetail {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Low,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Low => Self::High,
            Self::Medium => Self::Low,
            Self::High => Self::Medium,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PropDensity {
    Off,
    Low,
    Medium,
    High,
}

impl PropDensity {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Off => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Off,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Off => Self::High,
            Self::Low => Self::Off,
            Self::Medium => Self::Low,
            Self::High => Self::Medium,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiScalePreset {
    Compact,
    Normal,
    Large,
}

impl UiScalePreset {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Normal => "Normal",
            Self::Large => "Large",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Compact => Self::Normal,
            Self::Normal => Self::Large,
            Self::Large => Self::Compact,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Compact => Self::Large,
            Self::Normal => Self::Compact,
            Self::Large => Self::Normal,
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct GraphicsSettings {
    pub preset: GraphicsPreset,
    pub render_tier: RenderTierChoice,
    pub mesh_detail: MeshDetail,
    pub prop_density: PropDensity,
    pub show_ground: bool,
    pub show_markers: bool,
    pub show_telemetry: bool,
    pub ui_scale: UiScalePreset,
}

impl GraphicsSettings {
    pub fn default_for(capability: RenderCapability) -> Self {
        let mut settings = Self {
            preset: GraphicsPreset::Balanced,
            render_tier: RenderTierChoice::Auto,
            mesh_detail: MeshDetail::Medium,
            prop_density: PropDensity::Medium,
            show_ground: true,
            show_markers: true,
            show_telemetry: true,
            ui_scale: UiScalePreset::Normal,
        };

        if capability.software_fallback {
            settings.apply_preset(GraphicsPreset::Low);
            settings.show_telemetry = true;
        }

        settings
    }

    pub fn apply_preset(&mut self, preset: GraphicsPreset) {
        match preset {
            GraphicsPreset::Low => {
                self.preset = GraphicsPreset::Low;
                self.render_tier = RenderTierChoice::ShippingLow;
                self.mesh_detail = MeshDetail::Low;
                self.prop_density = PropDensity::Low;
                self.show_ground = true;
                self.show_markers = true;
                self.show_telemetry = false;
                self.ui_scale = UiScalePreset::Normal;
            }
            GraphicsPreset::Balanced => {
                self.preset = GraphicsPreset::Balanced;
                self.render_tier = RenderTierChoice::Auto;
                self.mesh_detail = MeshDetail::Medium;
                self.prop_density = PropDensity::Medium;
                self.show_ground = true;
                self.show_markers = true;
                self.show_telemetry = true;
                self.ui_scale = UiScalePreset::Normal;
            }
            GraphicsPreset::High => {
                self.preset = GraphicsPreset::High;
                self.render_tier = RenderTierChoice::Full3d;
                self.mesh_detail = MeshDetail::High;
                self.prop_density = PropDensity::High;
                self.show_ground = true;
                self.show_markers = true;
                self.show_telemetry = true;
                self.ui_scale = UiScalePreset::Large;
            }
            GraphicsPreset::Custom => {}
        }
    }

    pub fn mark_custom(&mut self) {
        self.preset = GraphicsPreset::Custom;
    }

    pub const fn active_render_tier(&self, capability: RenderCapability) -> ActiveRenderTier {
        if capability.software_fallback {
            return ActiveRenderTier::FallbackDebug;
        }

        match self.render_tier {
            RenderTierChoice::Auto => ActiveRenderTier::Full3d,
            RenderTierChoice::FallbackDebug => ActiveRenderTier::FallbackDebug,
            RenderTierChoice::ShippingLow => ActiveRenderTier::ShippingLow,
            RenderTierChoice::Full3d => ActiveRenderTier::Full3d,
        }
    }

    pub const fn effective_mesh_detail(&self, capability: RenderCapability) -> MeshDetail {
        match self.active_render_tier(capability) {
            ActiveRenderTier::FallbackDebug => MeshDetail::Low,
            ActiveRenderTier::ShippingLow => match self.mesh_detail {
                MeshDetail::High => MeshDetail::Medium,
                other => other,
            },
            ActiveRenderTier::Full3d => self.mesh_detail,
        }
    }

    pub const fn effective_prop_density(&self, capability: RenderCapability) -> PropDensity {
        match self.active_render_tier(capability) {
            ActiveRenderTier::FallbackDebug => match self.prop_density {
                PropDensity::Off => PropDensity::Off,
                _ => PropDensity::Low,
            },
            ActiveRenderTier::ShippingLow => match self.prop_density {
                PropDensity::High => PropDensity::Medium,
                other => other,
            },
            ActiveRenderTier::Full3d => self.prop_density,
        }
    }

    pub const fn hud_font_size(&self) -> f32 {
        match self.ui_scale {
            UiScalePreset::Compact => 15.0,
            UiScalePreset::Normal => 17.0,
            UiScalePreset::Large => 20.0,
        }
    }

    pub const fn hud_panel_width(&self) -> f32 {
        match self.ui_scale {
            UiScalePreset::Compact => HUD_PANEL_W - 40.0,
            UiScalePreset::Normal => HUD_PANEL_W,
            UiScalePreset::Large => HUD_PANEL_W + 60.0,
        }
    }

    pub const fn settings_panel_width(&self) -> f32 {
        self.hud_panel_width() + 40.0
    }

    pub const fn gizmo_line_width(&self) -> f32 {
        match self.ui_scale {
            UiScalePreset::Compact => 2.0,
            UiScalePreset::Normal => 3.0,
            UiScalePreset::Large => 4.0,
        }
    }

    pub fn row_value_label(&self, row: SettingsMenuRow, capability: RenderCapability) -> String {
        match row {
            SettingsMenuRow::Preset => self.preset.label().to_string(),
            SettingsMenuRow::Renderer => format!(
                "{} -> {}",
                self.render_tier.label(),
                self.active_render_tier(capability).label()
            ),
            SettingsMenuRow::MeshDetail => {
                self.effective_mesh_detail(capability).label().to_string()
            }
            SettingsMenuRow::PropDensity => {
                self.effective_prop_density(capability).label().to_string()
            }
            SettingsMenuRow::Ground => on_off(self.show_ground).to_string(),
            SettingsMenuRow::Markers => on_off(self.show_markers).to_string(),
            SettingsMenuRow::Telemetry => on_off(self.show_telemetry).to_string(),
            SettingsMenuRow::UiScale => self.ui_scale.label().to_string(),
        }
    }

    pub fn adjust(&mut self, row: SettingsMenuRow, delta: i32) {
        let forward = delta >= 0;

        match row {
            SettingsMenuRow::Preset => {
                let next = if forward {
                    self.preset.next_selectable()
                } else {
                    self.preset.prev_selectable()
                };
                self.apply_preset(next);
            }
            SettingsMenuRow::Renderer => {
                self.render_tier = if forward {
                    self.render_tier.next()
                } else {
                    self.render_tier.prev()
                };
                self.mark_custom();
            }
            SettingsMenuRow::MeshDetail => {
                self.mesh_detail = if forward {
                    self.mesh_detail.next()
                } else {
                    self.mesh_detail.prev()
                };
                self.mark_custom();
            }
            SettingsMenuRow::PropDensity => {
                self.prop_density = if forward {
                    self.prop_density.next()
                } else {
                    self.prop_density.prev()
                };
                self.mark_custom();
            }
            SettingsMenuRow::Ground => {
                self.show_ground = !self.show_ground;
                self.mark_custom();
            }
            SettingsMenuRow::Markers => {
                self.show_markers = !self.show_markers;
                self.mark_custom();
            }
            SettingsMenuRow::Telemetry => {
                self.show_telemetry = !self.show_telemetry;
                self.mark_custom();
            }
            SettingsMenuRow::UiScale => {
                self.ui_scale = if forward {
                    self.ui_scale.next()
                } else {
                    self.ui_scale.prev()
                };
                self.mark_custom();
            }
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SettingsMenuState {
    pub open: bool,
    pub selected_row: usize,
}

impl SettingsMenuState {
    pub fn move_selection(&mut self, delta: i32) {
        let len = SettingsMenuRow::ALL.len() as i32;
        let next = (self.selected_row as i32 + delta).rem_euclid(len);
        self.selected_row = next as usize;
    }

    pub fn selected_row(&self) -> SettingsMenuRow {
        SettingsMenuRow::ALL[self.selected_row]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsMenuRow {
    Preset,
    Renderer,
    MeshDetail,
    PropDensity,
    Ground,
    Markers,
    Telemetry,
    UiScale,
}

impl SettingsMenuRow {
    pub const ALL: [Self; 8] = [
        Self::Preset,
        Self::Renderer,
        Self::MeshDetail,
        Self::PropDensity,
        Self::Ground,
        Self::Markers,
        Self::Telemetry,
        Self::UiScale,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Preset => "Preset",
            Self::Renderer => "Renderer",
            Self::MeshDetail => "Geometry",
            Self::PropDensity => "Props",
            Self::Ground => "Ground",
            Self::Markers => "Markers",
            Self::Telemetry => "Telemetry",
            Self::UiScale => "UI Scale",
        }
    }
}

fn on_off(value: bool) -> &'static str {
    if value {
        "On"
    } else {
        "Off"
    }
}
