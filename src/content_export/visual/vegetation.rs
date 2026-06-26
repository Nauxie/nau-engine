use super::{metrics::finite_ratio, types::GroundCoverBladeStats};
use crate::{
    content_export::shared::mesh_positions,
    generated_content::{
        TREE_BRANCH_COUNT, TREE_ROOT_FLARE_COUNT, TREE_TRUNK_RING_COUNT, TREE_TRUNK_SEGMENTS,
        VERTICES_PER_GROUND_BLADE,
    },
};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

#[derive(Debug)]
pub(super) struct VisualTreeSpec {
    pub(super) label: String,
    pub(super) trunk_radius_m: f32,
    pub(super) trunk_height_m: f32,
    pub(super) seed: u32,
    pub(super) canopy_radius_m: f32,
    pub(super) canopy_seed: u32,
}

pub(super) fn visual_content_tree_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<VisualTreeSpec> {
    let mut specs = Vec::new();

    for tree_index in 0..3 {
        if island.is_target && tree_index == 1 {
            continue;
        }
        specs.push(VisualTreeSpec {
            label: format!("detail tree {tree_index}"),
            trunk_radius_m: 0.22,
            trunk_height_m: 2.1 + tree_index as f32 * 0.25,
            seed: 5_000 + island_index as u32 * 97 + tree_index as u32 * 13,
            canopy_radius_m: 1.05 + tree_index as f32 * 0.08,
            canopy_seed: 6_000 + island_index as u32 * 101 + tree_index as u32 * 17,
        });
    }

    if island.name == "launch mesa" {
        specs.push(VisualTreeSpec {
            label: "launch tree".to_string(),
            trunk_radius_m: 0.35,
            trunk_height_m: 4.4,
            seed: 7_000 + island_index as u32 * 97,
            canopy_radius_m: 1.55,
            canopy_seed: 8_000 + island_index as u32 * 101,
        });
    }

    specs
}
pub(super) fn ground_cover_blade_stats(mesh: &Mesh) -> GroundCoverBladeStats {
    let positions = mesh_positions(mesh);
    let mut blade_count = 0usize;
    let mut min_height_m = f32::INFINITY;
    let mut max_height_m = 0.0f32;

    for blade in positions.chunks_exact(VERTICES_PER_GROUND_BLADE) {
        let base_y = blade[0][1].min(blade[1][1]);
        let tip_y = blade[4][1];
        let height = (tip_y - base_y).max(0.0);
        min_height_m = min_height_m.min(height);
        max_height_m = max_height_m.max(height);
        blade_count += 1;
    }

    if blade_count == 0 {
        return GroundCoverBladeStats::default();
    }

    GroundCoverBladeStats {
        blade_count,
        min_height_m,
        max_height_m,
        height_range_m: max_height_m - min_height_m,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TreeTrunkShapeMetrics {
    pub(super) taper_ratio: f32,
    pub(super) branch_reach_ratio: f32,
    pub(super) branch_count: usize,
    pub(super) root_flare_count: usize,
    pub(super) trunk_ring_count: usize,
}

pub(super) fn tree_trunk_shape_metrics(mesh: &Mesh) -> TreeTrunkShapeMetrics {
    let positions = mesh_positions(mesh);
    let top_ring_start = TREE_TRUNK_SEGMENTS * (TREE_TRUNK_RING_COUNT - 1);
    let branch_vertices_start = TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT + 2;
    if positions.len() <= branch_vertices_start {
        return TreeTrunkShapeMetrics::default();
    }

    let bottom_radius = average_xz_radius(&positions[0..TREE_TRUNK_SEGMENTS]);
    let top_radius =
        average_xz_radius(&positions[top_ring_start..top_ring_start + TREE_TRUNK_SEGMENTS]);
    let branch_reach = positions[branch_vertices_start..]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);

    let taper_ratio = finite_ratio(bottom_radius, top_radius);
    let branch_reach_ratio = finite_ratio(branch_reach, bottom_radius);

    TreeTrunkShapeMetrics {
        taper_ratio,
        branch_reach_ratio,
        branch_count: TREE_BRANCH_COUNT,
        root_flare_count: TREE_ROOT_FLARE_COUNT,
        trunk_ring_count: TREE_TRUNK_RING_COUNT,
    }
}

fn average_xz_radius(points: &[[f32; 3]]) -> f32 {
    if points.is_empty() {
        return 0.0;
    }
    let center = points
        .iter()
        .map(|position| Vec2::new(position[0], position[2]))
        .sum::<Vec2>()
        / points.len() as f32;

    points
        .iter()
        .map(|position| (Vec2::new(position[0], position[2]) - center).length())
        .sum::<f32>()
        / points.len() as f32
}

pub(super) fn tree_canopy_lobe_count() -> usize {
    1 + 5
}
