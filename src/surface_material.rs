use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

const SURFACE_SHADER_PATH: &str = "shaders/surface_material.wgsl";

pub(crate) type SurfaceMaterial = ExtendedMaterial<StandardMaterial, SurfaceExtension>;

#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
pub(crate) struct SurfaceMaterialUniform {
    pub(crate) primary_tint: Vec4,
    pub(crate) secondary_tint: Vec4,
    pub(crate) accent_tint: Vec4,
    /// x: surface kind, y: deterministic phase, z: macro scale, w: detail strength.
    pub(crate) parameters: Vec4,
    /// x: first octave scale, y: wave strength, z: flow speed, w: foam strength.
    pub(crate) motion: Vec4,
}

#[derive(Asset, AsBindGroup, Clone, Debug, Reflect)]
pub(crate) struct SurfaceExtension {
    #[uniform(100)]
    pub(crate) uniform: SurfaceMaterialUniform,
    #[texture(101)]
    #[sampler(102)]
    #[dependency]
    pub(crate) detail_normal_texture: Option<Handle<Image>>,
}

impl MaterialExtension for SurfaceExtension {
    fn fragment_shader() -> ShaderRef {
        SURFACE_SHADER_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SURFACE_SHADER_PATH.into()
    }
}

impl SurfaceExtension {
    pub(crate) fn terrain(
        primary_tint: Vec4,
        secondary_tint: Vec4,
        accent_tint: Vec4,
        phase: f32,
    ) -> Self {
        Self {
            uniform: SurfaceMaterialUniform {
                primary_tint,
                secondary_tint,
                accent_tint,
                parameters: Vec4::new(0.0, phase, 0.034, 0.68),
                motion: Vec4::ZERO,
            },
            detail_normal_texture: None,
        }
    }

    pub(crate) fn water(detail_normal_texture: Handle<Image>, phase: f32) -> Self {
        Self {
            uniform: SurfaceMaterialUniform {
                primary_tint: Vec4::new(0.045, 0.200, 0.260, 1.0),
                secondary_tint: Vec4::new(0.070, 0.380, 0.480, 1.0),
                accent_tint: Vec4::new(0.72, 0.95, 0.99, 1.0),
                parameters: Vec4::new(1.0, phase, 0.037, 0.82),
                motion: Vec4::new(2.4, 0.44, 0.055, 0.84),
            },
            detail_normal_texture: Some(detail_normal_texture),
        }
    }

    pub(crate) fn foam(detail_normal_texture: Handle<Image>, phase: f32) -> Self {
        Self {
            uniform: SurfaceMaterialUniform {
                primary_tint: Vec4::new(0.68, 0.86, 0.91, 1.0),
                secondary_tint: Vec4::new(0.82, 0.96, 0.98, 1.0),
                accent_tint: Vec4::new(0.98, 1.0, 1.0, 1.0),
                parameters: Vec4::new(2.0, phase, 0.055, 0.74),
                motion: Vec4::new(3.2, 0.32, 0.090, 1.0),
            },
            detail_normal_texture: Some(detail_normal_texture),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SURFACE_SHADER: &str = include_str!("../assets/shaders/surface_material.wgsl");

    #[test]
    fn surface_kinds_share_one_extension_without_pipeline_keys() {
        let image = Handle::<Image>::default();
        let terrain = SurfaceExtension::terrain(Vec4::ONE, Vec4::ONE, Vec4::ONE, 0.25);
        let water = SurfaceExtension::water(image.clone(), 0.5);
        let foam = SurfaceExtension::foam(image, 0.75);

        assert_eq!(terrain.uniform.parameters.x, 0.0);
        assert_eq!(water.uniform.parameters.x, 1.0);
        assert_eq!(foam.uniform.parameters.x, 2.0);
        assert!(terrain.detail_normal_texture.is_none());
        assert!(water.detail_normal_texture.is_some());
        assert!(foam.detail_normal_texture.is_some());
    }

    #[test]
    fn surface_shader_preserves_water_orientation_and_unlit_foam_contracts() {
        assert!(SURFACE_SHADER.contains("uv = in.uv;"));
        assert!(SURFACE_SHADER.contains("detail_normal * face_sign"));
        assert!(SURFACE_SHADER.contains("vec2<f32>(0.012, -0.110)"));
        assert!(SURFACE_SHADER.contains("STANDARD_MATERIAL_FLAGS_UNLIT_BIT"));
        assert!(SURFACE_SHADER.contains("out.color = pbr_input.material.base_color;"));
    }
}
