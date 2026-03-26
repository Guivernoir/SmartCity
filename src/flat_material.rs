use bevy::asset::Asset;
use bevy::color::{Color, LinearRgba};
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

const FLAT_COLOR_SHADER_PATH: &str = "shaders/flat_color_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FlatMaterial {
    #[uniform(0)]
    color: LinearRgba,
    alpha_mode: AlphaMode,
}

impl FlatMaterial {
    pub fn opaque(color: Color) -> Self {
        Self {
            color: color.to_linear(),
            alpha_mode: AlphaMode::Opaque,
        }
    }

    pub fn transparent(color: Color) -> Self {
        Self {
            color: color.to_linear(),
            alpha_mode: AlphaMode::Blend,
        }
    }
}

impl Material for FlatMaterial {
    fn fragment_shader() -> ShaderRef {
        FLAT_COLOR_SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}
