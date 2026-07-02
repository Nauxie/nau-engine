use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use std::{fs::File, io::Write, path::Path};

pub(super) fn write_mesh_obj(path: &Path, mesh: &Mesh, object_name: &str) -> std::io::Result<()> {
    let positions = mesh_positions(mesh);
    let normals = mesh_normals(mesh).filter(|normals| normals.len() == positions.len());
    let uvs = mesh_uv0(mesh).filter(|uvs| uvs.len() == positions.len());
    let colors = mesh_colors(mesh).filter(|colors| colors.len() == positions.len());
    let indices = mesh_index_values(mesh);
    let mut file = File::create(path)?;

    writeln!(file, "# NAU terrain export")?;
    writeln!(file, "o {}", terrain_export_slug(object_name))?;
    for (index, position) in positions.iter().enumerate() {
        if let Some(colors) = colors {
            let color = colors[index];
            writeln!(
                file,
                "v {:.6} {:.6} {:.6} {:.4} {:.4} {:.4}",
                position[0], position[1], position[2], color[0], color[1], color[2]
            )?;
        } else {
            writeln!(
                file,
                "v {:.6} {:.6} {:.6}",
                position[0], position[1], position[2]
            )?;
        }
    }
    if let Some(uvs) = uvs {
        for uv in uvs {
            writeln!(file, "vt {:.6} {:.6}", uv[0], uv[1])?;
        }
    }
    if let Some(normals) = normals {
        for normal in normals {
            writeln!(
                file,
                "vn {:.6} {:.6} {:.6}",
                normal[0], normal[1], normal[2]
            )?;
        }
    }

    let has_uvs = uvs.is_some();
    let has_normals = normals.is_some();
    for triangle in indices.chunks_exact(3) {
        writeln!(
            file,
            "f {} {} {}",
            obj_face_index(triangle[0], has_uvs, has_normals),
            obj_face_index(triangle[1], has_uvs, has_normals),
            obj_face_index(triangle[2], has_uvs, has_normals)
        )?;
    }

    Ok(())
}

pub(super) fn write_terrain_material_weights_csv(path: &Path, mesh: &Mesh) -> std::io::Result<()> {
    let Some(weights) = mesh_terrain_material_weights(mesh) else {
        return Ok(());
    };
    let mut file = File::create(path)?;
    writeln!(file, "vertex,lush_highland,exposed_edge")?;
    for (index, weight) in weights.iter().enumerate() {
        writeln!(file, "{index},{:.4},{:.4}", weight[0], weight[1])?;
    }
    Ok(())
}

fn obj_face_index(index: u32, has_uvs: bool, has_normals: bool) -> String {
    let obj_index = index + 1;
    match (has_uvs, has_normals) {
        (true, true) => format!("{obj_index}/{obj_index}/{obj_index}"),
        (true, false) => format!("{obj_index}/{obj_index}"),
        (false, true) => format!("{obj_index}//{obj_index}"),
        (false, false) => obj_index.to_string(),
    }
}

pub(super) fn mesh_positions(mesh: &Mesh) -> &[[f32; 3]] {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => &[],
    }
}

fn mesh_normals(mesh: &Mesh) -> Option<&[[f32; 3]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => Some(values),
        _ => None,
    }
}

pub(crate) fn mesh_uv0(mesh: &Mesh) -> Option<&[[f32; 2]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values),
        _ => None,
    }
}

fn mesh_colors(mesh: &Mesh) -> Option<&[[f32; 4]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => Some(values),
        _ => None,
    }
}

fn mesh_terrain_material_weights(mesh: &Mesh) -> Option<&[[f32; 2]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_1) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values),
        _ => None,
    }
}

pub(super) fn mesh_index_values(mesh: &Mesh) -> Vec<u32> {
    match mesh.indices() {
        Some(Indices::U16(values)) => values.iter().map(|index| u32::from(*index)).collect(),
        Some(Indices::U32(values)) => values.clone(),
        None => (0..mesh.count_vertices() as u32).collect(),
    }
}

pub(super) fn terrain_export_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;

    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }

    if last_was_separator {
        slug.pop();
    }
    if slug.is_empty() {
        "unnamed".to_string()
    } else {
        slug
    }
}

pub(crate) fn terrain_export_json_vec3(value: Vec3) -> String {
    format!(
        "[{}, {}, {}]",
        terrain_export_json_number(value.x),
        terrain_export_json_number(value.y),
        terrain_export_json_number(value.z)
    )
}

pub(super) fn terrain_export_json_vec2(value: Vec2) -> String {
    format!(
        "[{}, {}]",
        terrain_export_json_number(value.x),
        terrain_export_json_number(value.y)
    )
}

pub(crate) fn terrain_export_json_number(value: f32) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "0.0000".to_string()
    }
}

pub(crate) fn terrain_export_json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => output.push_str(&format!("\\u{:04x}", value as u32)),
            value => output.push(value),
        }
    }
    output.push('"');
    output
}
