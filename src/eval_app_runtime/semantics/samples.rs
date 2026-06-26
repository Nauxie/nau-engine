use crate::eval_app_runtime::scene::EvalScene;
use crate::generated_content::island_visual_surface_position;
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

#[derive(Clone, Copy, Debug)]
pub(super) struct SemanticSceneSample {
    pub(super) kind: &'static str,
    pub(super) label: &'static str,
    pub(super) expected_material: &'static str,
    pub(super) world_position: Vec3,
}

pub(super) fn semantic_scene_samples(scene: &EvalScene) -> Vec<SemanticSceneSample> {
    let mut samples = Vec::new();

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        samples.push(SemanticSceneSample {
            kind: "terrain_surface",
            label: island.name,
            expected_material: "terrain",
            world_position: island_visual_surface_position(island, Vec2::new(0.16, -0.14))
                + Vec3::Y * 0.08,
        });
        samples.push(SemanticSceneSample {
            kind: "distant_island",
            label: island.name,
            expected_material: "distant_island",
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
                world_position: canopy_position,
            });
        }
    }

    for cloud_transform in scene.weather_clouds.iter().take(18) {
        samples.push(SemanticSceneSample {
            kind: "weather_cloud",
            label: "weather cloud",
            expected_material: "cloud",
            world_position: cloud_transform.translation,
        });
    }

    samples
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
