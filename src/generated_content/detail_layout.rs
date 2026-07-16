use super::island_playable_normalized_offset;
use super::random_unit;
use bevy::prelude::*;
use nau_engine::world::{IslandBiome, IslandLandmarkRole, SkyIsland};

const GOLDEN_ANGLE_RADIANS: f32 = 2.399_963_1;
const MAX_TREE_COUNT: usize = 12;
const GREAT_PLATEAU_TREE_OFFSETS: [[f32; 2]; 14] = [
    [-0.64, 0.16],
    [-0.58, 0.34],
    [-0.49, 0.14],
    [0.18, -0.34],
    [0.32, -0.50],
    [0.45, -0.32],
    [-0.22, 0.78],
    [0.20, 0.78],
    [0.52, 0.60],
    [-0.38, 0.30],
    [0.40, 0.10],
    [-0.34, -0.28],
    [0.46, -0.14],
    [0.02, 0.46],
];
const ROCK_FOOTPRINT_INSET_SCALE: f32 = 0.95;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IslandDetailBudget {
    pub(crate) ground_cover_patch_count: usize,
    pub(crate) tree_count: usize,
    pub(crate) rock_count: usize,
    pub(crate) ruin_arch_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IslandTreeSpec {
    pub(crate) normalized_offset: Vec2,
    pub(crate) trunk_radius_m: f32,
    pub(crate) trunk_height_m: f32,
    pub(crate) canopy_radius_m: f32,
    pub(crate) trunk_seed: u32,
    pub(crate) canopy_seed: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IslandRockSpec {
    pub(crate) normalized_offset: Vec2,
    pub(crate) scale_m: f32,
    pub(crate) seed: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IslandRuinSpec {
    pub(crate) normalized_offset: Vec2,
    pub(crate) width_m: f32,
    pub(crate) height_m: f32,
    pub(crate) depth_m: f32,
    pub(crate) rotation_y: f32,
    pub(crate) seed: u32,
}

pub(crate) fn island_detail_budget(island: SkyIsland) -> IslandDetailBudget {
    let area_m2 = island.half_extents.x * island.half_extents.y;
    let tier = area_tier(area_m2);
    let biome = island.world_tags.biome;
    let base_ground_cover: usize = [24, 36, 52, 72, 104, 160][tier];
    let base_trees: usize = [1, 2, 3, 4, 6, 10][tier];
    let base_rocks: usize = [2, 3, 5, 7, 10, 16][tier];

    let ground_cover_patch_count = base_ground_cover;
    let tree_count = if island.is_great_plateau_anchor() {
        GREAT_PLATEAU_TREE_OFFSETS.len()
    } else {
        match biome {
            IslandBiome::Garden => base_trees + 2,
            IslandBiome::Orchard => base_trees + 3,
            IslandBiome::Meadow | IslandBiome::Lake | IslandBiome::Mist => base_trees,
            IslandBiome::Storm | IslandBiome::Alpine | IslandBiome::Ruin => {
                base_trees.saturating_sub(1).max(1)
            }
        }
        .min(MAX_TREE_COUNT)
    };
    let rock_count = match biome {
        IslandBiome::Storm | IslandBiome::Alpine | IslandBiome::Ruin => base_rocks + 2,
        IslandBiome::Meadow
        | IslandBiome::Garden
        | IslandBiome::Orchard
        | IslandBiome::Lake
        | IslandBiome::Mist => base_rocks,
    };
    let ruin_arch_count = if island.is_great_plateau_anchor()
        || (biome != IslandBiome::Ruin
            && island.world_tags.landmark_role != IslandLandmarkRole::RuinArch)
    {
        0
    } else if area_m2 < 1_200.0 {
        1
    } else if area_m2 < 6_000.0 {
        2
    } else {
        3
    };

    IslandDetailBudget {
        ground_cover_patch_count,
        tree_count,
        rock_count,
        ruin_arch_count,
    }
}

pub(crate) fn island_tree_specs(island_index: usize, island: SkyIsland) -> Vec<IslandTreeSpec> {
    let count = island_detail_budget(island).tree_count;
    let layout_seed = detail_seed(island_index, island, 0x51a7_3e2d);
    let (height_scale, canopy_scale, trunk_scale) = tree_biome_scales(island.world_tags.biome);

    (0..count)
        .map(|tree_index| {
            let sample = tree_index as u32;
            let normalized_offset = if island.is_great_plateau_anchor() {
                island_playable_normalized_offset(
                    island,
                    Vec2::from_array(GREAT_PLATEAU_TREE_OFFSETS[tree_index]),
                )
            } else {
                distributed_offset(
                    island,
                    tree_index,
                    count,
                    layout_seed,
                    if island.is_target { 0.48 } else { 0.30 },
                    0.82,
                )
            };
            let (plateau_height_scale, plateau_canopy_scale, plateau_trunk_scale) =
                if island.is_great_plateau_anchor() {
                    (1.48, 1.62, 1.34)
                } else {
                    (1.0, 1.0, 1.0)
                };
            let trunk_height_m = (2.10 + random_unit(layout_seed, sample, 41) * 1.45)
                * height_scale
                * plateau_height_scale;
            let trunk_radius_m = (0.19 + random_unit(layout_seed, sample, 53) * 0.09)
                * trunk_scale
                * plateau_trunk_scale;
            let canopy_radius_m = (0.95 + random_unit(layout_seed, sample, 67) * 0.50)
                * canopy_scale
                * plateau_canopy_scale;

            IslandTreeSpec {
                normalized_offset,
                trunk_radius_m,
                trunk_height_m,
                canopy_radius_m,
                trunk_seed: mixed_seed(layout_seed ^ sample.wrapping_mul(0x9e37_79b9) ^ 0x5000),
                canopy_seed: mixed_seed(layout_seed ^ sample.wrapping_mul(0x85eb_ca6b) ^ 0x6000),
            }
        })
        .collect()
}

pub(crate) fn island_rock_specs(island_index: usize, island: SkyIsland) -> Vec<IslandRockSpec> {
    let count = island_detail_budget(island).rock_count;
    let layout_seed = detail_seed(island_index, island, 0x8c63_19f5);
    let biome_scale = rock_biome_scale(island.world_tags.biome);

    (0..count)
        .map(|rock_index| {
            let sample = rock_index as u32;
            IslandRockSpec {
                normalized_offset: distributed_offset(
                    island,
                    rock_index,
                    count,
                    layout_seed,
                    0.48,
                    0.86,
                ) * ROCK_FOOTPRINT_INSET_SCALE,
                scale_m: ((0.42 + random_unit(layout_seed, sample, 79) * 0.38) * biome_scale)
                    .clamp(0.34, 1.10),
                seed: mixed_seed(layout_seed ^ sample.wrapping_mul(0xc2b2_ae35) ^ 0x9000),
            }
        })
        .collect()
}

pub(crate) fn island_ruin_specs(island_index: usize, island: SkyIsland) -> Vec<IslandRuinSpec> {
    let count = island_detail_budget(island).ruin_arch_count;
    if count == 0 {
        return Vec::new();
    }

    let layout_seed = detail_seed(island_index, island, 0xd47a_6c21);
    let cluster_angle = random_unit(layout_seed, 0, 89) * std::f32::consts::TAU;
    let cluster_radius = 0.34 + random_unit(layout_seed, 0, 97) * 0.18;
    let cluster_center = Vec2::new(cluster_angle.cos(), cluster_angle.sin()) * cluster_radius;
    let base_width = (island.half_extents.x * 0.24).clamp(5.5, 18.0);
    let base_height = (island.thickness * 0.38).clamp(4.8, 12.0);
    let base_depth = (island.half_extents.y * 0.08).clamp(1.2, 3.2);

    (0..count)
        .map(|ruin_index| {
            let sample = ruin_index as u32;
            let local_angle = cluster_angle
                + ruin_index as f32 * GOLDEN_ANGLE_RADIANS
                + (random_unit(layout_seed, sample, 101) - 0.5) * 0.22;
            let local_radius = if count == 1 {
                0.0
            } else {
                0.055 + random_unit(layout_seed, sample, 103) * 0.065
            };
            let normalized_offset = island_playable_normalized_offset(
                island,
                cluster_center + Vec2::new(local_angle.cos(), local_angle.sin()) * local_radius,
            );
            let size_scale = 0.82 + random_unit(layout_seed, sample, 107) * 0.26;

            IslandRuinSpec {
                normalized_offset,
                width_m: base_width * size_scale,
                height_m: base_height * (0.90 + random_unit(layout_seed, sample, 109) * 0.18),
                depth_m: base_depth * (0.88 + random_unit(layout_seed, sample, 113) * 0.22),
                rotation_y: cluster_angle
                    + std::f32::consts::FRAC_PI_2
                    + (random_unit(layout_seed, sample, 127) - 0.5) * 0.46,
                seed: mixed_seed(layout_seed ^ sample.wrapping_mul(0x27d4_eb2d) ^ 0x15000),
            }
        })
        .collect()
}

fn area_tier(area_m2: f32) -> usize {
    if area_m2 < 500.0 {
        0
    } else if area_m2 < 1_200.0 {
        1
    } else if area_m2 < 2_600.0 {
        2
    } else if area_m2 < 6_000.0 {
        3
    } else if area_m2 < 18_000.0 {
        4
    } else {
        5
    }
}

fn distributed_offset(
    island: SkyIsland,
    index: usize,
    count: usize,
    seed: u32,
    min_radius: f32,
    max_radius: f32,
) -> Vec2 {
    let sample = index as u32;
    let phase = random_unit(seed, 0, 7) * std::f32::consts::TAU;
    let angle =
        phase + index as f32 * GOLDEN_ANGLE_RADIANS + (random_unit(seed, sample, 11) - 0.5) * 0.24;
    let sequence_radius = ((index as f32 + 0.5) / count.max(1) as f32).sqrt();
    let radius_mix = sequence_radius * 0.42 + random_unit(seed, sample, 17).sqrt() * 0.58;
    let radius = min_radius + (max_radius - min_radius) * radius_mix;
    let direction = Vec2::new(angle.cos(), angle.sin());
    let tangent = Vec2::new(-direction.y, direction.x);
    let jitter = (random_unit(seed, sample, 23) - 0.5) * 0.07;

    island_playable_normalized_offset(island, direction * radius + tangent * jitter)
}

fn tree_biome_scales(biome: IslandBiome) -> (f32, f32, f32) {
    match biome {
        IslandBiome::Meadow => (1.0, 1.0, 1.0),
        IslandBiome::Garden => (0.96, 1.20, 0.94),
        IslandBiome::Storm => (0.80, 0.72, 1.12),
        IslandBiome::Orchard => (1.16, 1.25, 1.04),
        IslandBiome::Lake => (1.02, 1.10, 0.98),
        IslandBiome::Mist => (1.10, 1.02, 0.96),
        IslandBiome::Alpine => (0.88, 0.78, 1.10),
        IslandBiome::Ruin => (0.84, 0.70, 1.08),
    }
}

fn rock_biome_scale(biome: IslandBiome) -> f32 {
    match biome {
        IslandBiome::Meadow => 1.0,
        IslandBiome::Garden => 0.88,
        IslandBiome::Storm => 1.25,
        IslandBiome::Orchard => 0.94,
        IslandBiome::Lake => 1.02,
        IslandBiome::Mist => 0.96,
        IslandBiome::Alpine => 1.18,
        IslandBiome::Ruin => 1.14,
    }
}

fn detail_seed(island_index: usize, island: SkyIsland, salt: u32) -> u32 {
    let mut seed = (island_index as u32)
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(salt);
    for byte in island.name.bytes() {
        seed = (seed ^ byte as u32).wrapping_mul(0x0100_0193);
    }
    seed ^= island.half_extents.x.to_bits().rotate_left(7);
    seed ^= island.half_extents.y.to_bits().rotate_left(17);
    seed ^= island.thickness.to_bits().rotate_left(23);
    mixed_seed(seed)
}

fn mixed_seed(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::world::{IslandLandmarkRole, IslandPlateauRegion, SkyRoute};

    fn test_island(name: &'static str, half_extents: Vec2) -> SkyIsland {
        SkyIsland::new(name, Vec3::ZERO, half_extents, 18.0, false)
    }

    fn assert_playable_offset(island: SkyIsland, offset: Vec2) {
        let angle = offset.y.atan2(offset.x);
        assert!(
            offset.length() <= island.playable_silhouette_scale(angle) * 0.94 + 0.000_1,
            "{offset:?} should stay inside {}'s playable silhouette",
            island.name
        );
    }

    #[test]
    fn area_budgets_are_monotonic_across_all_tiers() {
        let islands = [
            test_island("midpoint shelf", Vec2::new(20.0, 20.0)),
            test_island("midpoint shelf", Vec2::new(30.0, 20.0)),
            test_island("midpoint shelf", Vec2::new(50.0, 30.0)),
            test_island("midpoint shelf", Vec2::new(80.0, 50.0)),
            test_island("midpoint shelf", Vec2::new(120.0, 80.0)),
            test_island("midpoint shelf", Vec2::new(160.0, 120.0)),
        ];
        let budgets = islands.map(island_detail_budget);

        for pair in budgets.windows(2) {
            assert!(pair[0].ground_cover_patch_count < pair[1].ground_cover_patch_count);
            assert!(pair[0].tree_count < pair[1].tree_count);
            assert!(pair[0].rock_count < pair[1].rock_count);
        }
    }

    #[test]
    fn biome_rules_enrich_growth_and_stone_scatter_differently() {
        let extents = Vec2::new(50.0, 40.0);
        let meadow = island_detail_budget(test_island("midpoint shelf", extents));
        let garden = island_detail_budget(test_island("landing garden", extents));
        let orchard = island_detail_budget(test_island("high orchard", extents));
        let storm = island_detail_budget(test_island("storm porch", extents));
        let alpine = island_detail_budget(test_island("far needle", extents));
        let ruin = island_detail_budget(test_island("broken stair", extents));
        let huge_orchard =
            island_detail_budget(test_island("high orchard", Vec2::new(160.0, 120.0)));

        assert!(garden.tree_count > meadow.tree_count);
        assert!(orchard.tree_count > garden.tree_count);
        assert!(storm.tree_count < meadow.tree_count);
        assert!(alpine.tree_count < meadow.tree_count);
        assert!(ruin.tree_count < meadow.tree_count);
        assert!(storm.rock_count > meadow.rock_count);
        assert!(alpine.rock_count > meadow.rock_count);
        assert!(ruin.rock_count > meadow.rock_count);
        assert_eq!(
            garden.ground_cover_patch_count,
            meadow.ground_cover_patch_count
        );
        assert_eq!(huge_orchard.tree_count, MAX_TREE_COUNT);
    }

    #[test]
    fn generated_detail_specs_are_deterministic() {
        let island = test_island("high orchard", Vec2::new(72.0, 46.0));

        assert_eq!(island_tree_specs(9, island), island_tree_specs(9, island));
        assert_eq!(island_rock_specs(9, island), island_rock_specs(9, island));
        assert_eq!(
            island_ruin_specs(9, test_island("mist arch", Vec2::new(78.0, 34.0))),
            island_ruin_specs(9, test_island("mist arch", Vec2::new(78.0, 34.0)))
        );
    }

    #[test]
    fn generated_offsets_stay_inside_playable_footprints() {
        let storm = test_island("storm porch", Vec2::new(42.0, 28.0));
        for spec in island_tree_specs(8, storm) {
            assert_playable_offset(storm, spec.normalized_offset);
        }
        for spec in island_rock_specs(8, storm) {
            assert_playable_offset(storm, spec.normalized_offset);
        }

        let ruin = test_island("mist arch", Vec2::new(78.0, 34.0));
        for spec in island_ruin_specs(13, ruin) {
            assert_playable_offset(ruin, spec.normalized_offset);
        }
    }

    #[test]
    fn great_plateau_trees_form_large_edge_groves_and_clear_the_arrival_lane() {
        let plateau = test_island("great sky plateau", Vec2::new(230.0, 155.0));
        let specs = island_tree_specs(31, plateau);
        let mut region_counts = [0usize; IslandPlateauRegion::COUNT];

        assert_eq!(specs.len(), GREAT_PLATEAU_TREE_OFFSETS.len());
        for spec in specs {
            assert_playable_offset(plateau, spec.normalized_offset);
            assert!(spec.trunk_height_m >= 3.1);
            assert!(spec.canopy_radius_m >= 1.5);
            assert!(spec.normalized_offset.length() >= 0.38);
            assert!(
                !(spec.normalized_offset.x > -0.44
                    && spec.normalized_offset.x < 0.32
                    && spec.normalized_offset.y.abs() < 0.18),
                "{:?} should not block the central arrival lane",
                spec.normalized_offset
            );

            let region = plateau
                .plateau_region_at_normalized_offset(spec.normalized_offset)
                .expect("authored plateau trees should stay in a plateau region");
            region_counts[region as usize] += 1;
        }

        assert!(region_counts[IslandPlateauRegion::MeadowPlateau as usize] >= 4);
        assert!(region_counts[IslandPlateauRegion::HighShelf as usize] >= 3);
        assert!(region_counts[IslandPlateauRegion::LowBasin as usize] >= 3);
        assert!(region_counts[IslandPlateauRegion::CliffRim as usize] >= 3);
    }

    #[test]
    fn ruin_clusters_require_ruin_biome_or_landmark_and_skip_plateau() {
        let mut biome_only = test_island("broken stair", Vec2::new(20.0, 20.0));
        biome_only.world_tags.landmark_role = IslandLandmarkRole::None;
        let mut landmark_only = test_island("midpoint shelf", Vec2::new(50.0, 40.0));
        landmark_only.world_tags.landmark_role = IslandLandmarkRole::RuinArch;
        let large_ruin = test_island("broken stair", Vec2::new(100.0, 80.0));
        let meadow = test_island("midpoint shelf", Vec2::new(50.0, 40.0));
        let plateau = test_island("great sky plateau", Vec2::new(230.0, 155.0));

        assert_eq!(island_ruin_specs(0, biome_only).len(), 1);
        assert_eq!(island_ruin_specs(1, landmark_only).len(), 2);
        assert_eq!(island_ruin_specs(2, large_ruin).len(), 3);
        assert!(island_ruin_specs(3, meadow).is_empty());
        assert!(island_ruin_specs(4, plateau).is_empty());
    }

    #[test]
    fn route_detail_budgets_clear_richer_density_floors() {
        let route = SkyRoute::default();
        let ground_cover_patches = route
            .islands()
            .iter()
            .copied()
            .map(island_detail_budget)
            .map(|budget| budget.ground_cover_patch_count)
            .sum::<usize>();
        let tree_count = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .map(|(index, island)| island_tree_specs(index, island).len())
            .sum::<usize>();
        let rock_count = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .map(|(index, island)| island_rock_specs(index, island).len())
            .sum::<usize>();
        let ruin_cluster_count = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, island)| !island_ruin_specs(*index, *island).is_empty())
            .count();

        assert!(ground_cover_patches >= 2_400);
        assert!(tree_count >= 160);
        assert!(rock_count >= 230);
        assert!(ruin_cluster_count >= 6);
    }
}
