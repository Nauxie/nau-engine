use super::types::VisualPaletteSummary;
use crate::generated_content::{biome_detail_color_set, terrain_biome_palette};
use bevy::prelude::*;
use nau_engine::world::authored_island_art_direction_at;
use std::collections::HashSet;

const MIN_PERCEPTUAL_PALETTE_DISTANCE: f32 = 0.004;

type VisualPaletteKey = [[u8; 3]; 3];

pub(super) fn visual_content_palette_summary(index: usize) -> VisualPaletteSummary {
    let terrain = terrain_biome_palette(index);
    let detail = biome_detail_color_set(index);
    let art = authored_island_art_direction_at(index)
        .unwrap_or_else(|| panic!("island palette index {index} must have art direction"));

    VisualPaletteSummary {
        index,
        island_name: art.island_name,
        epithet: art.epithet,
        palette_family: art.palette_family.label(),
        surface_pattern: art.surface_pattern.label(),
        hero_landmark: art.hero_landmark.label(),
        water_story: art.water_story.label(),
        art_direction_signature: art.signature(),
        flora_kinds: art.flora_kinds.map(|kind| kind.label()),
        flora_count: usize::from(art.flora_count),
        formation_kinds: art.formation_kinds.map(|kind| kind.label()),
        formation_count: usize::from(art.formation_count),
        ruin_kinds: art.ruin_kinds.map(|kind| kind.label()),
        ruin_count: usize::from(art.ruin_count),
        terrain_key: visual_content_vec3_key(terrain.grass),
        foliage_key: visual_content_rgba_key(detail.foliage_primary),
        stone_key: visual_content_rgba_key(detail.stone_primary),
    }
}

fn visual_content_vec3_key(color: Vec3) -> [u8; 3] {
    [
        (color.x.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.y.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.z.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

fn visual_content_rgba_key(color: [u8; 4]) -> [u8; 3] {
    [color[0], color[1], color[2]]
}

pub(super) fn visual_content_coarse_color_count(colors: impl Iterator<Item = [u8; 3]>) -> usize {
    colors
        .map(|color| color.map(|channel| channel / 8))
        .collect::<HashSet<_>>()
        .len()
}

pub(super) fn visual_content_distinct_palette_count(palettes: &[VisualPaletteSummary]) -> usize {
    visual_content_distinct_palette_key_count(palettes.iter().map(visual_content_palette_key))
}

fn visual_content_distinct_palette_key_count(
    palettes: impl Iterator<Item = VisualPaletteKey>,
) -> usize {
    let mut distinct = Vec::<VisualPaletteKey>::new();
    for palette in palettes {
        if distinct.iter().all(|other| {
            visual_content_palette_distance(palette, *other) >= MIN_PERCEPTUAL_PALETTE_DISTANCE
        }) {
            distinct.push(palette);
        }
    }
    distinct.len()
}

fn visual_content_palette_key(palette: &VisualPaletteSummary) -> VisualPaletteKey {
    [palette.terrain_key, palette.foliage_key, palette.stone_key]
}

fn visual_content_palette_distance(left: VisualPaletteKey, right: VisualPaletteKey) -> f32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| visual_content_oklab(left).distance(visual_content_oklab(right)))
        .fold(0.0, f32::max)
}

fn visual_content_oklab(color: [u8; 3]) -> Vec3 {
    let [red, green, blue] = color.map(srgb_channel_to_linear);
    let light = (0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue).cbrt();
    let medium = (0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue).cbrt();
    let short = (0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue).cbrt();

    Vec3::new(
        0.210_454_26 * light + 0.793_617_8 * medium - 0.004_072_047 * short,
        1.977_998_5 * light - 2.428_592_2 * medium + 0.450_593_7 * short,
        0.025_904_037 * light + 0.782_771_77 * medium - 0.808_675_77 * short,
    )
}

fn srgb_channel_to_linear(channel: u8) -> f32 {
    let channel = f32::from(channel) / 255.0;
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

pub(super) fn visual_content_json_u8_triplet(value: [u8; 3]) -> String {
    format!("[{}, {}, {}]", value[0], value[1], value[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::world::SKY_ROUTE_ISLAND_COUNT;

    #[test]
    fn one_channel_one_value_delta_is_not_a_distinct_palette() {
        let authored = visual_content_palette_summary(0);
        let original = visual_content_palette_key(&authored);
        let mut near_identical = original;
        near_identical[0][0] = near_identical[0][0].saturating_add(1);

        assert_eq!(
            visual_content_distinct_palette_key_count([original, near_identical].into_iter()),
            1
        );
    }

    #[test]
    fn all_authored_visual_palettes_remain_perceptually_distinct() {
        let palettes = (0..SKY_ROUTE_ISLAND_COUNT)
            .map(visual_content_palette_summary)
            .collect::<Vec<_>>();

        assert_eq!(visual_content_distinct_palette_count(&palettes), 41);
    }
}
