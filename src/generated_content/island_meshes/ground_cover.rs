use super::super::random_unit;
use super::constants::{
    GROUND_COVER_BLADES_PER_PATCH, INDICES_PER_GROUND_BLADE, VERTICES_PER_GROUND_BLADE,
};
use super::shape::island_playable_normalized_offset;
use crate::generated_content::detail_meshes::{IslandWaterFootprint, island_water_visual_specs};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::{
    IslandArtDirection, IslandPaletteFamily, IslandSurfacePattern, IslandWaterStory, SkyIsland,
    authored_island_art_direction,
};

pub(crate) fn island_ground_cover_mesh(
    island_index: usize,
    island: SkyIsland,
    patch_count: usize,
) -> Mesh {
    let blade_count = patch_count * GROUND_COVER_BLADES_PER_PATCH;
    let mut positions = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut normals = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut uvs = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut indices = Vec::with_capacity(blade_count * INDICES_PER_GROUND_BLADE);
    let art_direction = authored_island_art_direction(island.name);
    let seed = island_index as u32 * 41
        + 503
        + art_direction.map_or(0, |profile| profile.signature_seed.rotate_left(9));
    let (blade_width_scale, blade_height_scale, blade_lean_scale) = art_direction
        .map(ground_cover_form_scales)
        .unwrap_or((1.0, 1.0, 1.0));
    let water_footprints = island_water_visual_specs(island_index, island)
        .into_iter()
        .filter_map(|spec| spec.horizontal_footprint())
        .collect::<Vec<_>>();

    for patch in 0..patch_count {
        let base_angle = random_unit(seed, patch as u32, 3) * std::f32::consts::TAU;
        let normalized_offset = art_direction.map_or_else(
            || {
                let radius = random_unit(seed, patch as u32, 11).sqrt() * 0.90;
                let jitter = Vec2::new(
                    (random_unit(seed, patch as u32, 17) - 0.5) * 0.08,
                    (random_unit(seed, patch as u32, 23) - 0.5) * 0.08,
                );
                island_playable_normalized_offset(
                    island,
                    Vec2::new(base_angle.cos(), base_angle.sin()) * radius + jitter,
                )
            },
            |profile| ground_cover_patch_offset(island, patch, patch_count, seed, profile),
        );
        let normalized_offset =
            ground_cover_offset_clear_of_water(island, normalized_offset, patch, &water_footprints);
        let x = island.center.x + normalized_offset.x * island.half_extents.x;
        let z = island.center.z + normalized_offset.y * island.half_extents.y;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) + 0.08;

        for blade in 0..GROUND_COVER_BLADES_PER_PATCH {
            let blade_phase = base_angle
                + blade as f32 * std::f32::consts::TAU / GROUND_COVER_BLADES_PER_PATCH as f32;
            let width = (0.14 + random_unit(seed, patch as u32, 31 + blade as u32) * 0.15)
                * blade_width_scale;
            let height = (0.72 + random_unit(seed, patch as u32, 43 + blade as u32) * 0.86)
                * blade_height_scale;
            let lean = Vec3::new(blade_phase.cos(), 0.0, blade_phase.sin())
                * (0.1 + random_unit(seed, patch as u32, 53 + blade as u32) * 0.24)
                * blade_lean_scale;
            push_ground_cover_blade(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                Vec3::new(x, surface_y, z),
                blade_phase,
                width,
                height,
                lean,
                patch,
            );
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

fn ground_cover_offset_clear_of_water(
    island: SkyIsland,
    original: Vec2,
    patch: usize,
    water_footprints: &[IslandWaterFootprint],
) -> Vec2 {
    const SHORELINE_MARGIN_M: f32 = 1.35;
    if water_footprints.is_empty()
        || ground_cover_offset_clears_water(island, original, water_footprints, SHORELINE_MARGIN_M)
    {
        return original;
    }

    let phase = patch as f32 * 2.399_963_1;
    let mut best = None::<(f32, Vec2)>;
    for ring in 0..14 {
        let radius = 0.16 + ring as f32 * 0.055;
        for angle_step in 0..64 {
            let angle =
                phase + angle_step as f32 * std::f32::consts::TAU / 64.0 + ring as f32 * 0.071;
            let candidate = island_playable_normalized_offset(
                island,
                Vec2::new(angle.cos(), angle.sin()) * radius,
            );
            if !ground_cover_offset_clears_water(
                island,
                candidate,
                water_footprints,
                SHORELINE_MARGIN_M,
            ) {
                continue;
            }
            let distance = candidate.distance_squared(original);
            if best.is_none_or(|(best_distance, _)| distance < best_distance) {
                best = Some((distance, candidate));
            }
        }
    }

    best.map_or(original, |(_, candidate)| candidate)
}

fn ground_cover_offset_clears_water(
    island: SkyIsland,
    normalized_offset: Vec2,
    water_footprints: &[IslandWaterFootprint],
    margin_m: f32,
) -> bool {
    let world_xz = Vec2::new(
        island.center.x + normalized_offset.x * island.half_extents.x,
        island.center.z + normalized_offset.y * island.half_extents.y,
    );
    water_footprints
        .iter()
        .all(|footprint| !footprint.contains_world_xz(world_xz, margin_m))
}

fn ground_cover_patch_offset(
    island: SkyIsland,
    patch: usize,
    patch_count: usize,
    seed: u32,
    profile: IslandArtDirection,
) -> Vec2 {
    let progress = (patch as f32 + 0.5) / patch_count.max(1) as f32;
    let centered = progress * 2.0 - 1.0;
    let side = if patch.is_multiple_of(2) { -1.0 } else { 1.0 };
    let angle = progress * std::f32::consts::TAU;
    let row = patch % 5;
    let column_count = patch_count.div_ceil(5).max(1);
    let column = patch / 5;
    let column_progress = if column_count <= 1 {
        0.0
    } else {
        column as f32 / (column_count - 1) as f32 * 2.0 - 1.0
    };
    let pattern = match profile.surface_pattern {
        IslandSurfacePattern::TerracedCourt => {
            Vec2::new(centered * 0.78, side * (0.18 + 0.08 * (patch % 4) as f32))
        }
        IslandSurfacePattern::BraidedCauseway => Vec2::new(
            centered * 0.82,
            (centered * std::f32::consts::PI * 2.0).sin() * 0.24 + side * 0.08,
        ),
        IslandSurfacePattern::RadialGarden => {
            Vec2::new(angle.cos(), angle.sin()) * (0.32 + 0.42 * ((patch % 4) as f32 / 3.0))
        }
        IslandSurfacePattern::CrownedRidge => {
            Vec2::new(centered * 0.76, 0.26 + (angle * 1.5).sin() * 0.24)
        }
        IslandSurfacePattern::WindwardRibs => {
            Vec2::new(centered * 0.82, side * (0.18 + 0.09 * (patch % 4) as f32))
        }
        IslandSurfacePattern::OrchardRows => {
            Vec2::new(column_progress * 0.80, (row as f32 - 2.0) * 0.18)
        }
        IslandSurfacePattern::NeedleHalo => {
            Vec2::new(angle.cos(), angle.sin()) * (0.50 + 0.19 * (patch % 3) as f32 / 2.0)
        }
        IslandSurfacePattern::BasinRings => {
            let radius = 0.28 + 0.18 * (patch % 3) as f32;
            Vec2::new(angle.cos(), angle.sin()) * radius
        }
        IslandSurfacePattern::ProcessionalAxis => {
            Vec2::new(centered * 0.82, side * (0.11 + 0.07 * (patch % 3) as f32))
        }
        IslandSurfacePattern::PortalCourt => {
            Vec2::new(side * (0.26 + 0.11 * (patch % 4) as f32), centered * 0.72)
        }
        IslandSurfacePattern::UnderhangThreshold => {
            Vec2::new(angle.cos() * 0.72, angle.sin() * 0.54 - 0.12)
        }
        IslandSurfacePattern::ThermalSpiral => {
            let radius = 0.18 + progress * 0.66;
            let spiral_angle = progress * std::f32::consts::TAU * 3.2;
            Vec2::new(spiral_angle.cos(), spiral_angle.sin()) * radius
        }
        IslandSurfacePattern::CascadeTerraces => Vec2::new(
            centered * 0.74,
            0.62 - (patch % 6) as f32 * 0.23 + side * 0.035,
        ),
        IslandSurfacePattern::PlateauDistricts => {
            let district_angle = angle + (patch % 4) as f32 * 0.22;
            Vec2::new(district_angle.cos() * 0.76, district_angle.sin() * 0.68)
        }
        IslandSurfacePattern::SummitSanctum => {
            Vec2::new(angle.cos(), angle.sin()) * (0.42 + 0.28 * (patch % 3) as f32 / 2.0)
        }
    };
    let rotation = profile.hero_rotation_degrees as f32 * std::f32::consts::PI / 180.0;
    let rotated = Vec2::new(
        pattern.x * rotation.cos() - pattern.y * rotation.sin(),
        pattern.x * rotation.sin() + pattern.y * rotation.cos(),
    );
    let random_radius = random_unit(seed, patch as u32, 11).sqrt() * 0.86;
    let random_angle = random_unit(seed, patch as u32, 13) * std::f32::consts::TAU;
    let random_disk = Vec2::new(random_angle.cos(), random_angle.sin()) * random_radius;
    let jitter = Vec2::new(
        random_unit(seed, patch as u32, 17) - 0.5,
        random_unit(seed, patch as u32, 23) - 0.5,
    ) * 0.07;
    let mut offset = rotated * 0.68
        + random_disk * 0.26
        + Vec2::from_array(profile.flora_anchor) * 0.12
        + jitter;
    let hero_anchor = Vec2::from_array(profile.hero_anchor);
    let hero_delta = offset - hero_anchor;
    if hero_delta.length() < 0.12 {
        offset = hero_anchor + hero_delta.normalize_or(Vec2::Y) * 0.12;
    }

    island_playable_normalized_offset(island, offset * 0.96)
}

fn ground_cover_form_scales(profile: IslandArtDirection) -> (f32, f32, f32) {
    let palette = match profile.palette_family {
        IslandPaletteFamily::VerdantSun | IslandPaletteFamily::PlateauBloom => (1.08, 1.10, 0.95),
        IslandPaletteFamily::CopperOrchard => (1.02, 1.04, 0.92),
        IslandPaletteFamily::StormSlate => (0.88, 0.90, 1.34),
        IslandPaletteFamily::MistJade => (0.94, 1.08, 0.86),
        IslandPaletteFamily::SapphireWetland => (0.92, 1.18, 1.08),
        IslandPaletteFamily::AlpineFrost => (0.80, 0.90, 1.28),
        IslandPaletteFamily::RuinOchre => (0.86, 0.90, 1.10),
        IslandPaletteFamily::CloudSilver => (0.90, 0.92, 1.18),
    };
    let water_height = match profile.water_story {
        IslandWaterStory::DryWindCarved => 0.94,
        IslandWaterStory::SpringPond | IslandWaterStory::ReflectingBasin => 1.04,
        IslandWaterStory::ReedyLake
        | IslandWaterStory::CascadeRun
        | IslandWaterStory::WaterfallGarden
        | IslandWaterStory::MistPool
        | IslandWaterStory::CaveSeep => 1.10,
    };
    (palette.0, palette.1 * water_height, palette.2)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_ground_cover_blade(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    origin: Vec3,
    angle: f32,
    width: f32,
    height: f32,
    lean: Vec3,
    patch: usize,
) {
    let right = Vec3::new(angle.cos(), 0.0, angle.sin());
    let cross = Vec3::new(-angle.sin(), 0.0, angle.cos());
    let side = right * (width * 0.5);
    let mid_side = right * (width * 0.26);
    let mid = origin + Vec3::Y * (height * 0.54) + lean * 0.42;
    let tip = origin + Vec3::Y * height + lean;
    let leaflet_sign = if patch.is_multiple_of(2) { 1.0 } else { -1.0 };
    let leaflet = mid
        + cross * (leaflet_sign * width * 0.95)
        + right * (width * 0.10)
        + Vec3::Y * (height * 0.08);
    let blade_normal = Vec3::new(right.z * 0.35, 0.8, -right.x * 0.35).normalize();
    let leaflet_normal = (blade_normal + cross * (leaflet_sign * 0.18)).normalize();
    let start = positions.len() as u32;

    positions.extend([
        (origin - side).to_array(),
        (origin + side).to_array(),
        (mid - mid_side).to_array(),
        (mid + mid_side).to_array(),
        tip.to_array(),
        leaflet.to_array(),
    ]);
    normals.extend([
        blade_normal.to_array(),
        blade_normal.to_array(),
        blade_normal.to_array(),
        blade_normal.to_array(),
        blade_normal.to_array(),
        leaflet_normal.to_array(),
    ]);
    let uv_offset = if patch.is_multiple_of(2) { 0.0 } else { 0.5 };
    uvs.extend([
        [uv_offset, 1.0],
        [uv_offset + 0.42, 1.0],
        [uv_offset + 0.10, 0.46],
        [uv_offset + 0.32, 0.46],
        [uv_offset + 0.21, 0.0],
        [uv_offset + 0.44, 0.34],
    ]);
    indices.extend([
        start,
        start + 1,
        start + 2,
        start + 1,
        start + 3,
        start + 2,
        start + 2,
        start + 3,
        start + 4,
        start + 2,
        start + 3,
        start + 5,
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use nau_engine::world::SkyRoute;

    #[test]
    fn authored_ground_cover_blade_height_ranges_meet_export_contract() {
        const ACCEPTED_BLADE_HEIGHT_RANGE_M: f32 = 0.70;
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let patch_count =
                crate::generated_content::island_detail_budget(island).ground_cover_patch_count;
            let mesh = island_ground_cover_mesh(island_index, island, patch_count);
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("ground cover should expose Float32x3 positions");
            };
            let mut min_height_m = f32::INFINITY;
            let mut max_height_m = f32::NEG_INFINITY;
            for blade in positions.chunks_exact(VERTICES_PER_GROUND_BLADE) {
                let base_y = blade[0][1].min(blade[1][1]);
                let height_m = blade[4][1] - base_y;
                min_height_m = min_height_m.min(height_m);
                max_height_m = max_height_m.max(height_m);
            }

            assert!(
                max_height_m - min_height_m >= ACCEPTED_BLADE_HEIGHT_RANGE_M,
                "{} blade-height range was {:.3}m",
                island.name,
                max_height_m - min_height_m
            );
        }
    }

    #[test]
    fn ground_cover_origins_clear_horizontal_water_footprints() {
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let footprints = island_water_visual_specs(island_index, island)
                .into_iter()
                .filter_map(|spec| spec.horizontal_footprint())
                .collect::<Vec<_>>();
            if footprints.is_empty() {
                continue;
            }
            let mesh = island_ground_cover_mesh(island_index, island, 104);
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("ground cover should expose Float32x3 positions");
            };

            for blade in positions.chunks_exact(VERTICES_PER_GROUND_BLADE) {
                let origin = (Vec3::from_array(blade[0]) + Vec3::from_array(blade[1])) * 0.5;
                assert!(
                    footprints
                        .iter()
                        .all(|footprint| !footprint.contains_world_xz(origin.xz(), 1.30)),
                    "{} ground cover overlaps authored water at {:?}",
                    island.name,
                    origin
                );
            }
        }
    }
}
