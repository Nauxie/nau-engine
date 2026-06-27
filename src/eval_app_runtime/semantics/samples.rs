use crate::environment_visuals::{
    crosswind_ribbon_scene_sample_positions, updraft_ribbon_scene_sample_positions,
};
use crate::eval_app_runtime::scene::EvalScene;
use crate::generated_content::{
    TERRAIN_BIOME_PALETTE_COUNT, island_playable_normalized_offset, island_visual_surface_position,
};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

#[derive(Clone, Copy, Debug)]
pub(super) struct SemanticSceneSample {
    pub(super) kind: &'static str,
    pub(super) label: &'static str,
    pub(super) expected_material: &'static str,
    pub(super) material_variant: &'static str,
    pub(super) world_position: Vec3,
}

const UPDRAFT_RIBBON_SAMPLE_LABELS: [&str; 3] = [
    "updraft wind ribbon lower",
    "updraft wind ribbon middle",
    "updraft wind ribbon upper",
];
const CROSSWIND_RIBBON_SAMPLE_LABELS: [&str; 3] = [
    "crosswind wind ribbon leading",
    "crosswind wind ribbon center",
    "crosswind wind ribbon trailing",
];

pub(super) fn semantic_scene_samples(scene: &EvalScene) -> Vec<SemanticSceneSample> {
    let mut samples = Vec::new();

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        for world_position in island_terrain_surface_sample_positions(island) {
            samples.push(SemanticSceneSample {
                kind: "terrain_surface",
                label: island.name,
                expected_material: "terrain",
                material_variant: terrain_material_variant(island_index),
                world_position,
            });
        }
        samples.push(SemanticSceneSample {
            kind: "distant_island",
            label: island.name,
            expected_material: "distant_island",
            material_variant: "distant_island",
            world_position: island_visual_surface_position(island, Vec2::new(0.0, 0.0))
                + Vec3::Y * 1.2,
        });

        for (sample_index, canopy_position) in tree_canopy_sample_positions(island_index, island)
            .into_iter()
            .enumerate()
        {
            if sample_index == 1 && island.is_target {
                continue;
            }
            samples.push(SemanticSceneSample {
                kind: "tree_canopy",
                label: island.name,
                expected_material: "foliage",
                material_variant: "foliage",
                world_position: canopy_position,
            });
        }
    }

    if let Ok((player_transform, ..)) = scene.player.single()
        && let Some((island_index, island)) =
            nearest_island_to_position(scene.route.islands(), player_transform.translation)
    {
        for world_position in player_local_terrain_sample_positions(island, player_transform) {
            samples.push(SemanticSceneSample {
                kind: "terrain_surface",
                label: island.name,
                expected_material: "terrain",
                material_variant: terrain_material_variant(island_index),
                world_position,
            });
        }
    }

    if let Ok((_, camera_global_transform)) = scene.camera_projection.single()
        && let Ok((player_transform, ..)) = scene.player.single()
    {
        let camera_transform = camera_global_transform.compute_transform();
        if let Some((island_index, island, focus_xz)) = camera_framed_island_terrain_focus(
            scene.route.islands(),
            &camera_transform,
            player_transform.translation,
        ) {
            for world_position in
                camera_framed_terrain_sample_positions(island, &camera_transform, focus_xz)
            {
                samples.push(SemanticSceneSample {
                    kind: "terrain_surface",
                    label: island.name,
                    expected_material: "terrain",
                    material_variant: terrain_material_variant(island_index),
                    world_position,
                });
            }
        }
    }

    for cloud_transform in scene.weather_clouds.iter().take(18) {
        samples.push(SemanticSceneSample {
            kind: "weather_cloud",
            label: "weather cloud",
            expected_material: "cloud",
            material_variant: "cloud",
            world_position: cloud_transform.translation,
        });
    }

    for (_, _, transform) in scene.updraft_guides.iter().step_by(7).take(14) {
        samples.push(SemanticSceneSample {
            kind: "updraft_wind_visual",
            label: "updraft wind mote",
            expected_material: "wind",
            material_variant: "wind_updraft",
            world_position: transform.translation,
        });
    }
    for (_, ribbon, transform) in scene.updraft_ribbons.iter().take(6) {
        for (label, world_position) in UPDRAFT_RIBBON_SAMPLE_LABELS
            .into_iter()
            .zip(updraft_ribbon_scene_sample_positions(ribbon, transform))
        {
            samples.push(SemanticSceneSample {
                kind: "updraft_wind_visual",
                label,
                expected_material: "wind",
                material_variant: "wind_updraft",
                world_position,
            });
        }
    }
    for (_, _, transform) in scene.crosswind_guides.iter().step_by(8).take(12) {
        samples.push(SemanticSceneSample {
            kind: "crosswind_wind_visual",
            label: "crosswind wind mote",
            expected_material: "wind",
            material_variant: "wind_crosswind",
            world_position: transform.translation,
        });
    }
    for (_, ribbon, transform) in scene.crosswind_ribbons.iter().take(8) {
        for (label, world_position) in CROSSWIND_RIBBON_SAMPLE_LABELS
            .into_iter()
            .zip(crosswind_ribbon_scene_sample_positions(ribbon, transform))
        {
            samples.push(SemanticSceneSample {
                kind: "crosswind_wind_visual",
                label,
                expected_material: "wind",
                material_variant: "wind_crosswind",
                world_position,
            });
        }
    }

    samples
}

fn terrain_material_variant(island_index: usize) -> &'static str {
    match island_index % TERRAIN_BIOME_PALETTE_COUNT {
        1 => "terrain_gold_meadow",
        2 => "terrain_copper_clay",
        3 => "terrain_alpine_mist",
        4 => "terrain_highland_grass",
        _ => "terrain_lush_meadow",
    }
}

fn island_terrain_surface_sample_positions(island: SkyIsland) -> [Vec3; 4] {
    [
        Vec2::new(0.16, -0.14),
        Vec2::new(-0.28, 0.20),
        Vec2::new(0.42, 0.18),
        Vec2::new(0.0, 0.38),
    ]
    .map(|offset| island_visual_surface_position(island, offset) + Vec3::Y * 0.08)
}

fn nearest_island_to_position(islands: &[SkyIsland], position: Vec3) -> Option<(usize, SkyIsland)> {
    islands
        .iter()
        .copied()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.center
                .xz()
                .distance_squared(position.xz())
                .total_cmp(&b.center.xz().distance_squared(position.xz()))
        })
}

fn player_local_terrain_sample_positions(
    island: SkyIsland,
    player_transform: &Transform,
) -> [Vec3; 3] {
    let forward = (player_transform.rotation * -Vec3::Z)
        .xz()
        .normalize_or_zero();
    let forward = if forward.length_squared() > 0.0001 {
        forward
    } else {
        Vec2::NEG_Y
    };
    let right = Vec2::new(forward.y, -forward.x);
    let player_xz = player_transform.translation.xz();

    [
        player_xz + forward * 4.0,
        player_xz + forward * 8.0 + right * 3.5,
        player_xz + forward * 8.0 - right * 3.5,
    ]
    .map(|xz| terrain_sample_position_on_island(island, xz))
}

fn camera_framed_island_terrain_focus(
    islands: &[SkyIsland],
    camera_transform: &Transform,
    fallback_position: Vec3,
) -> Option<(usize, SkyIsland, Vec2)> {
    let camera_position = camera_transform.translation;
    let camera_forward = camera_transform.rotation * -Vec3::Z;
    if camera_forward.y.abs() <= 0.03 {
        return nearest_island_to_position(islands, fallback_position)
            .map(|(index, island)| (index, island, fallback_position.xz()));
    }

    islands
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, island)| {
            let distance_to_floor = (island.floor_y() - camera_position.y) / camera_forward.y;
            if !distance_to_floor.is_finite() || distance_to_floor <= 1.0 {
                return None;
            }
            let focus = camera_position + camera_forward * distance_to_floor;
            let normalized = island_normalized_offset(island, focus.xz());
            let radius = normalized.length();
            let silhouette = island
                .playable_silhouette_scale(normalized.y.atan2(normalized.x))
                .max(0.001);
            let normalized_distance = radius / silhouette;

            Some((index, island, focus.xz(), normalized_distance))
        })
        .filter(|(_, _, _, normalized_distance)| *normalized_distance <= 1.28)
        .min_by(|(_, _, _, a), (_, _, _, b)| a.total_cmp(b))
        .map(|(index, island, focus_xz, _)| (index, island, focus_xz))
        .or_else(|| {
            nearest_island_to_position(islands, fallback_position)
                .map(|(index, island)| (index, island, fallback_position.xz()))
        })
}

fn camera_framed_terrain_sample_positions(
    island: SkyIsland,
    camera_transform: &Transform,
    focus_xz: Vec2,
) -> [Vec3; 5] {
    let camera_right = (camera_transform.rotation * Vec3::X)
        .xz()
        .normalize_or_zero();
    let right = if camera_right.length_squared() > 0.0001 {
        camera_right
    } else {
        Vec2::X
    };
    let camera_forward = (camera_transform.rotation * -Vec3::Z)
        .xz()
        .normalize_or_zero();
    let forward = if camera_forward.length_squared() > 0.0001 {
        camera_forward
    } else {
        Vec2::NEG_Y
    };

    [
        focus_xz,
        focus_xz + right * 3.5,
        focus_xz - right * 3.5,
        focus_xz + forward * 5.0,
        focus_xz - forward * 5.0,
    ]
    .map(|xz| terrain_sample_position_on_island(island, xz))
}

fn terrain_sample_position_on_island(island: SkyIsland, xz: Vec2) -> Vec3 {
    let normalized =
        island_playable_normalized_offset(island, island_normalized_offset(island, xz));
    let clamped_xz = Vec2::new(
        island.center.x + normalized.x * island.half_extents.x,
        island.center.z + normalized.y * island.half_extents.y,
    );
    let probe = Vec3::new(clamped_xz.x, island.center.y, clamped_xz.y);
    let surface_y = island.mesh_top_y_at(probe);
    Vec3::new(clamped_xz.x, surface_y + 0.16, clamped_xz.y)
}

fn island_normalized_offset(island: SkyIsland, xz: Vec2) -> Vec2 {
    Vec2::new(
        (xz.x - island.center.x) / island.half_extents.x.max(0.001),
        (xz.y - island.center.z) / island.half_extents.y.max(0.001),
    )
}

pub(super) fn tree_canopy_sample_positions(island_index: usize, island: SkyIsland) -> Vec<Vec3> {
    if island.name == "launch mesa" {
        let launch_tree_height = 4.4;
        let launch_tree_surface_y =
            island.mesh_top_y_at(Vec3::new(island.center.x, island.center.y, 8.0));
        return vec![Vec3::new(
            island.center.x,
            launch_tree_surface_y + launch_tree_height + 0.85,
            8.0,
        )];
    }

    let detail_phase = island_index as f32 * 0.77;
    [
        Vec2::new(-0.42, -0.24),
        Vec2::new(0.34, -0.36),
        Vec2::new(0.24, 0.32),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, offset)| {
        let sway = (detail_phase + index as f32).sin() * 0.08;
        let surface = island_visual_surface_position(island, Vec2::new(offset.x + sway, offset.y));
        let trunk_height = 2.1 + index as f32 * 0.25;
        surface + Vec3::Y * (trunk_height + 0.72)
    })
    .collect()
}
