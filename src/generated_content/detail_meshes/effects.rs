use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) fn updraft_ribbon_mesh(radius: f32, height: f32, phase: f32) -> Mesh {
    const SEGMENTS: usize = 44;
    const STRANDS: f32 = 1.45;

    let width = (radius * 0.03).clamp(0.32, 0.65);
    let ribbon_radius = radius * 0.42;
    let mut positions = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut normals = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut uvs = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut indices = Vec::with_capacity(SEGMENTS * 6);

    for segment in 0..=SEGMENTS {
        let t = segment as f32 / SEGMENTS as f32;
        let angle = phase + t * std::f32::consts::TAU * STRANDS;
        let y = -height * 0.5 + t * height;
        let breathing = 1.0 + 0.08 * (angle * 2.0 + phase).sin();
        let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
        let center = radial * ribbon_radius * breathing + Vec3::Y * y;
        let side = radial * width;
        let normal = Vec3::new(radial.x * 0.32, 0.78, radial.z * 0.32).normalize();

        positions.extend([(center - side).to_array(), (center + side).to_array()]);
        normals.extend([normal.to_array(), normal.to_array()]);
        uvs.extend([[0.0, t], [1.0, t]]);

        if segment < SEGMENTS {
            let start = (segment * 2) as u32;
            indices.extend([start, start + 1, start + 2, start + 1, start + 3, start + 2]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn crosswind_flow_ribbon_mesh(length: f32, phase: f32) -> Mesh {
    const SEGMENTS: usize = 24;

    let length = length.max(1.0);
    let base_width = (length * 0.018).clamp(0.28, 0.74);
    let mut positions = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut normals = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut uvs = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut indices = Vec::with_capacity(SEGMENTS * 6);

    for segment in 0..=SEGMENTS {
        let t = segment as f32 / SEGMENTS as f32;
        let x = (t - 0.5) * length;
        let center = Vec3::X * x + crosswind_flow_ribbon_centerline_offset(length, phase, t);
        let taper = (std::f32::consts::PI * t).sin().max(0.0).powf(0.55);
        let flutter = (phase + t * std::f32::consts::TAU * 1.7).sin() * 0.08;
        let width = base_width * (0.28 + taper * (0.88 + flutter));
        let side = Vec3::Z * width;
        let normal =
            Vec3::new(0.0, 1.0, (phase + t * std::f32::consts::TAU).cos() * 0.14).normalize();

        positions.extend([(center - side).to_array(), (center + side).to_array()]);
        normals.extend([normal.to_array(), normal.to_array()]);
        uvs.extend([[0.0, t], [1.0, t]]);

        if segment < SEGMENTS {
            let start = (segment * 2) as u32;
            indices.extend([start, start + 1, start + 2, start + 1, start + 3, start + 2]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn crosswind_flow_ribbon_centerline_offset(length: f32, phase: f32, t: f32) -> Vec3 {
    let t = t.clamp(0.0, 1.0);
    let length = length.max(1.0);
    let envelope = (std::f32::consts::PI * t).sin().max(0.0);
    let wave = phase + t * std::f32::consts::TAU * 1.15;
    Vec3::new(
        0.0,
        wave.cos() * length * 0.012 * envelope,
        wave.sin() * length * 0.025 * envelope,
    )
}

pub(crate) fn player_airflow_streamline_mesh() -> Mesh {
    const SEGMENTS: usize = 14;

    let mut positions = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut normals = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut uvs = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut indices = Vec::with_capacity(SEGMENTS * 6);

    for segment in 0..=SEGMENTS {
        let t = segment as f32 / SEGMENTS as f32;
        let centered = t - 0.5;
        let taper = (std::f32::consts::PI * t).sin().max(0.0).powf(0.6);
        let curl = (t * std::f32::consts::TAU * 0.85).sin();
        let center = Vec3::new(
            curl * 0.055 * taper,
            (t * std::f32::consts::TAU).cos() * 0.012,
            centered,
        );
        let width = 0.24 * (0.10 + taper * 0.90);
        let side = Vec3::X * width;
        let normal = Vec3::new(curl * 0.08, 0.96, 0.18).normalize();

        positions.extend([(center - side).to_array(), (center + side).to_array()]);
        normals.extend([normal.to_array(), normal.to_array()]);
        uvs.extend([[0.0, t], [1.0, t]]);

        if segment < SEGMENTS {
            let start = (segment * 2) as u32;
            indices.extend([start, start + 1, start + 2, start + 1, start + 3, start + 2]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
