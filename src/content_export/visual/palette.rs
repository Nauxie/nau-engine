use super::types::VisualPaletteSummary;
use crate::generated_content::{biome_detail_color_set, terrain_biome_palette};
use bevy::prelude::*;

pub(super) fn visual_content_palette_summary(index: usize) -> VisualPaletteSummary {
    let terrain = terrain_biome_palette(index);
    let detail = biome_detail_color_set(index);

    VisualPaletteSummary {
        index,
        terrain_key: visual_content_vec3_key(terrain.grass),
        foliage_key: visual_content_rgba_key(detail.foliage_primary),
        stone_key: visual_content_rgba_key(detail.stone_primary),
    }
}

fn visual_content_vec3_key(color: Vec3) -> [u8; 3] {
    [
        (color.x.clamp(0.0, 1.0) * 31.0).round() as u8,
        (color.y.clamp(0.0, 1.0) * 31.0).round() as u8,
        (color.z.clamp(0.0, 1.0) * 31.0).round() as u8,
    ]
}

fn visual_content_rgba_key(color: [u8; 4]) -> [u8; 3] {
    [color[0] / 8, color[1] / 8, color[2] / 8]
}

pub(super) fn visual_content_json_u8_triplet(value: [u8; 3]) -> String {
    format!("[{}, {}, {}]", value[0], value[1], value[2])
}
