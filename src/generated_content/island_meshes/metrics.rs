use super::palette::terrain_material_region_id;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use std::collections::HashSet;

pub(crate) fn mesh_y_range(mesh: &Mesh) -> f32 {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return 0.0;
    };
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for position in positions {
        min_y = min_y.min(position[1]);
        max_y = max_y.max(position[1]);
    }
    if min_y.is_finite() && max_y.is_finite() {
        max_y - min_y
    } else {
        0.0
    }
}

pub(crate) fn mesh_vertical_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return 0;
    };
    let min_y = positions
        .iter()
        .map(|position| position[1])
        .fold(f32::INFINITY, f32::min);
    if !min_y.is_finite() {
        return 0;
    }

    positions
        .iter()
        .map(|position| ((position[1] - min_y) / 0.05).round() as i32)
        .collect::<HashSet<_>>()
        .len()
}

pub(crate) fn mesh_normal_slope_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    else {
        return 0;
    };

    normals
        .iter()
        .filter(|normal| normal[1] > 0.0)
        .map(|normal| {
            let horizontal = Vec2::new(normal[0], normal[2]).length();
            let slope_degrees = horizontal.atan2(normal[1].max(0.0001)).to_degrees();
            (slope_degrees * 2.0).round() as i32
        })
        .collect::<HashSet<_>>()
        .len()
}

pub(crate) fn mesh_vertex_color_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        return 0;
    };
    let mut bands = HashSet::new();
    for color in colors {
        bands.insert([
            (color[0].clamp(0.0, 1.0) * 31.0).round() as u8,
            (color[1].clamp(0.0, 1.0) * 31.0).round() as u8,
            (color[2].clamp(0.0, 1.0) * 31.0).round() as u8,
        ]);
    }
    bands.len()
}

pub(crate) fn mesh_terrain_material_weight_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    let mut bands = HashSet::new();
    for weight in weights {
        bands.insert([
            (weight[0].clamp(0.0, 1.0) * 15.0).round() as u8,
            (weight[1].clamp(0.0, 1.0) * 15.0).round() as u8,
        ]);
    }
    bands.len()
}

pub(crate) fn mesh_terrain_material_channel_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    let base = weights
        .iter()
        .any(|weight| weight[0] < 0.18 && weight[1] < 0.18);
    let lush = weights.iter().any(|weight| weight[0] > 0.18);
    let exposed = weights.iter().any(|weight| weight[1] > 0.18);
    usize::from(base) + usize::from(lush) + usize::from(exposed)
}

pub(crate) fn mesh_terrain_material_region_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    weights
        .iter()
        .map(|weight| terrain_material_region_id(*weight))
        .collect::<HashSet<_>>()
        .len()
}
