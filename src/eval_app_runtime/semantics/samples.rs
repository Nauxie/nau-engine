use crate::environment_visuals::{
    PlayerAirflowVisualKind, crosswind_ribbon_scene_sample_positions,
    player_airflow_scene_sample_positions, updraft_ribbon_scene_sample_positions,
};
use crate::eval_app_runtime::scene::EvalScene;
use crate::generated_content::{
    FloraMaterialRole, IslandArtifactMaterial, IslandWaterVisualKind, TERRAIN_BIOME_PALETTE_COUNT,
    WaterDetailKind, WaterDetailMaterialRole, island_artifact_visual_specs,
    island_flora_visual_specs, island_lake_basin_visual_specs, island_playable_normalized_offset,
    island_rock_formation_specs, island_ruin_complex_specs, island_visual_surface_position,
    island_water_detail_specs, island_water_visual_specs,
};
use bevy::prelude::*;
use nau_engine::world::{IslandPlateauRegion, SkyIsland};

#[derive(Clone, Copy, Debug)]
pub(super) struct SemanticSceneSample {
    pub(super) island_name: Option<&'static str>,
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
const MAX_FLORA_CLUSTER_SAMPLES_PER_ISLAND: usize = 4;
const MAX_RUIN_COMPLEX_SAMPLES_PER_ISLAND: usize = 2;
const MAX_ROCK_FORMATION_SAMPLES_PER_ISLAND: usize = 2;
const MAX_PLATEAU_WATER_DETAIL_SAMPLES: usize = 12;

pub(super) fn semantic_scene_samples(scene: &EvalScene) -> Vec<SemanticSceneSample> {
    let mut samples = Vec::new();

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        for world_position in island_terrain_surface_sample_positions(island) {
            samples.push(SemanticSceneSample {
                island_name: Some(island.name),
                kind: "terrain_surface",
                label: island.name,
                expected_material: "terrain",
                material_variant: terrain_material_variant(island_index),
                world_position,
            });
        }
        samples.push(SemanticSceneSample {
            island_name: Some(island.name),
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
                island_name: Some(island.name),
                kind: "tree_canopy",
                label: island.name,
                expected_material: "foliage",
                material_variant: "foliage",
                world_position: canopy_position,
            });
        }

        samples.extend(generated_content_scene_samples(island_index, island));
    }

    if let Ok((player_transform, ..)) = scene.player.single()
        && let Some((island_index, island)) =
            nearest_island_to_position(scene.route.islands(), player_transform.translation)
    {
        for world_position in player_local_terrain_sample_positions(island, player_transform) {
            samples.push(SemanticSceneSample {
                island_name: Some(island.name),
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
                    island_name: Some(island.name),
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
            island_name: None,
            kind: "weather_cloud",
            label: "weather cloud",
            expected_material: "cloud",
            material_variant: "cloud",
            world_position: cloud_transform.translation,
        });
    }

    for (_, _, transform) in scene.updraft_guides.iter().step_by(7).take(30) {
        samples.push(SemanticSceneSample {
            island_name: None,
            kind: "updraft_wind_visual",
            label: "updraft wind mote",
            expected_material: "wind",
            material_variant: "wind_updraft",
            world_position: transform.translation,
        });
    }
    for (_, ribbon, transform) in scene.updraft_ribbons.iter().take(12) {
        for (label, world_position) in UPDRAFT_RIBBON_SAMPLE_LABELS
            .into_iter()
            .zip(updraft_ribbon_scene_sample_positions(ribbon, transform))
        {
            samples.push(SemanticSceneSample {
                island_name: None,
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
            island_name: None,
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
                island_name: None,
                kind: "crosswind_wind_visual",
                label,
                expected_material: "wind",
                material_variant: "wind_crosswind",
                world_position,
            });
        }
    }
    for (visual, _, global_transform, visibility) in scene.player_wind_shear_visuals.iter() {
        if !matches!(*visibility, Visibility::Visible) {
            continue;
        }
        let transform = global_transform.compute_transform();
        for world_position in player_airflow_scene_sample_positions(visual, &transform) {
            samples.push(SemanticSceneSample {
                island_name: None,
                kind: "player_wind_shear_visual",
                label: player_wind_shear_visual_label(visual.kind),
                expected_material: "wind",
                material_variant: "wind_player_shear",
                world_position,
            });
        }
    }

    samples
}

fn generated_content_scene_samples(
    island_index: usize,
    island: SkyIsland,
) -> Vec<SemanticSceneSample> {
    let mut samples = Vec::new();
    let water_features = island_water_visual_specs(island_index, island);

    for water_feature in water_features.iter().copied() {
        samples.push(SemanticSceneSample {
            island_name: Some(island.name),
            kind: water_feature_sample_kind(water_feature.kind),
            label: water_feature.label,
            expected_material: "water",
            material_variant: "water",
            world_position: water_feature.translation,
        });
    }

    for lake_basin in island_lake_basin_visual_specs(island_index, island) {
        let rim_sample = Quat::from_rotation_y(lake_basin.rotation_y)
            * Vec3::new(
                (lake_basin.radius_x - lake_basin.rim_width * 0.35).max(0.1),
                lake_basin.rim_height * 0.55,
                0.0,
            );
        samples.push(SemanticSceneSample {
            island_name: Some(island.name),
            kind: "lake_basin",
            label: lake_basin.label,
            expected_material: "stone",
            material_variant: "stone_ruin",
            world_position: lake_basin.translation + rim_sample,
        });
    }

    if island.is_great_plateau_anchor() {
        for flora in island_flora_visual_specs(island_index, island)
            .into_iter()
            .take(MAX_FLORA_CLUSTER_SAMPLES_PER_ISLAND)
        {
            let (expected_material, material_variant) = flora_sample_material(flora.material);
            samples.push(SemanticSceneSample {
                island_name: Some(island.name),
                kind: "flora_cluster",
                label: flora.label,
                expected_material,
                material_variant,
                world_position: flora.translation
                    + Vec3::Y * flora_sample_vertical_offset(flora.material),
            });
        }

        for ruin in island_ruin_complex_specs(island_index, island)
            .into_iter()
            .take(MAX_RUIN_COMPLEX_SAMPLES_PER_ISLAND)
        {
            samples.push(SemanticSceneSample {
                island_name: Some(island.name),
                kind: "ruin_complex",
                label: ruin.label,
                expected_material: "stone",
                material_variant: "stone_ruin",
                world_position: elevated_feature_sample_position(
                    ruin.translation,
                    ruin.rotation_y,
                    ruin.camera_half_extents,
                ),
            });
        }

        for formation in island_rock_formation_specs(island_index, island)
            .into_iter()
            .take(MAX_ROCK_FORMATION_SAMPLES_PER_ISLAND)
        {
            samples.push(SemanticSceneSample {
                island_name: Some(island.name),
                kind: "rock_formation",
                label: formation.label,
                expected_material: "stone",
                material_variant: "stone_ruin",
                world_position: elevated_feature_sample_position(
                    formation.translation,
                    formation.rotation_y,
                    formation.camera_half_extents,
                ),
            });
        }

        for detail in island_water_detail_specs(island_index, island, &water_features)
            .into_iter()
            .take(MAX_PLATEAU_WATER_DETAIL_SAMPLES)
        {
            let (expected_material, material_variant) =
                water_detail_sample_material(detail.material);
            samples.push(SemanticSceneSample {
                island_name: Some(island.name),
                kind: water_detail_sample_kind(detail.kind),
                label: detail.label,
                expected_material,
                material_variant,
                world_position: detail.translation
                    + Vec3::Y * water_detail_sample_vertical_offset(detail.material),
            });
        }
    }

    for artifact in island_artifact_visual_specs(island_index, island) {
        let (expected_material, material_variant) = artifact_sample_material(artifact.material);
        samples.push(SemanticSceneSample {
            island_name: Some(island.name),
            kind: "surface_artifact",
            label: artifact.label,
            expected_material,
            material_variant,
            world_position: artifact.translation
                + Vec3::Y * artifact_sample_vertical_offset(artifact.label),
        });
    }

    if island.is_great_plateau_anchor() {
        samples.extend(great_plateau_arrival_scene_samples(island));
    }

    samples
}

fn water_feature_sample_kind(kind: IslandWaterVisualKind) -> &'static str {
    match kind {
        IslandWaterVisualKind::PlateauWaterfallRibbon
        | IslandWaterVisualKind::PlateauWaterfallMist
        | IslandWaterVisualKind::RouteWaterfallRibbon
        | IslandWaterVisualKind::RouteWaterfallMist => "waterfall_water",
        IslandWaterVisualKind::PondSurface
        | IslandWaterVisualKind::PlateauLakeSurface
        | IslandWaterVisualKind::RouteLakeSurface => "water_surface",
        IslandWaterVisualKind::RiverChannel => "river_channel",
    }
}

fn flora_sample_material(material: FloraMaterialRole) -> (&'static str, &'static str) {
    match material {
        FloraMaterialRole::Foliage | FloraMaterialRole::GroundCover => ("foliage", "foliage"),
        FloraMaterialRole::Flower => ("flower", "flower"),
    }
}

fn flora_sample_vertical_offset(material: FloraMaterialRole) -> f32 {
    match material {
        FloraMaterialRole::Foliage => 0.65,
        FloraMaterialRole::GroundCover => 0.35,
        FloraMaterialRole::Flower => 0.50,
    }
}

fn elevated_feature_sample_position(
    translation: Vec3,
    rotation_y: f32,
    camera_half_extents: Option<Vec3>,
) -> Vec3 {
    let half_extents = camera_half_extents.unwrap_or(Vec3::splat(0.5));
    translation
        + Quat::from_rotation_y(rotation_y)
            * Vec3::new(half_extents.x * 0.58, half_extents.y.max(0.5), 0.0)
}

fn water_detail_sample_material(material: WaterDetailMaterialRole) -> (&'static str, &'static str) {
    match material {
        WaterDetailMaterialRole::Water => ("water", "water"),
        WaterDetailMaterialRole::Stone => ("stone", "stone_ruin"),
        WaterDetailMaterialRole::Foliage => ("foliage", "foliage"),
        WaterDetailMaterialRole::Flower => ("flower", "flower"),
    }
}

fn water_detail_sample_kind(kind: WaterDetailKind) -> &'static str {
    match kind {
        WaterDetailKind::LilyPadColony => "water_detail_lily_pad",
        WaterDetailKind::ShoreReedArc => "water_detail_shore_reeds",
        WaterDetailKind::RiverbankCobbles => "water_detail_riverbank_cobbles",
        WaterDetailKind::WaterfallLipRocks => "water_detail_waterfall_lip",
        WaterDetailKind::PlungePoolRipples => "water_detail_plunge_pool",
        WaterDetailKind::MossySteppingStones => "water_detail_stepping_stones",
    }
}

fn water_detail_sample_vertical_offset(material: WaterDetailMaterialRole) -> f32 {
    match material {
        WaterDetailMaterialRole::Water => 0.08,
        WaterDetailMaterialRole::Stone => 0.20,
        WaterDetailMaterialRole::Foliage => 0.65,
        WaterDetailMaterialRole::Flower => 0.12,
    }
}

fn artifact_sample_material(material: IslandArtifactMaterial) -> (&'static str, &'static str) {
    match material {
        IslandArtifactMaterial::Stone => ("stone", "stone_ruin"),
        IslandArtifactMaterial::Foliage => ("foliage", "foliage"),
        IslandArtifactMaterial::Trunk => ("wood", "wood"),
    }
}

fn artifact_sample_vertical_offset(label: &str) -> f32 {
    match label {
        "glyph stone slab" => 1.2,
        "retaining wall fragment" | "reed patch" => 0.6,
        "broken bridge fragment" => 0.25,
        "ancient stair run" | "pebble field" => 0.12,
        _ => 0.0,
    }
}

fn great_plateau_arrival_scene_samples(island: SkyIsland) -> Vec<SemanticSceneSample> {
    let mut samples = Vec::with_capacity(2);

    if let Some(meadow) = island.plateau_region_position(IslandPlateauRegion::MeadowPlateau) {
        let shelf_radius = (island.half_extents.min_element() * 0.17).clamp(22.0, 34.0);
        samples.push(SemanticSceneSample {
            island_name: Some(island.name),
            kind: "plateau_arrival_shelf",
            label: "plateau meadow landing shelf",
            expected_material: "flower",
            material_variant: "flower",
            world_position: meadow
                + Vec3::Y * 0.42
                + Quat::from_rotation_y(0.18) * Vec3::X * shelf_radius,
        });
    }

    let ruin_surface = island_visual_surface_position(island, Vec2::new(-0.16, 0.12));
    let ruin_width = (island.half_extents.min_element() * 0.14).clamp(22.0, 30.0);
    let ruin_height = (island.thickness * 0.26).clamp(18.0, 26.0);
    let ruin_center = ruin_surface + Vec3::Y * (ruin_height * 0.46);
    samples.push(SemanticSceneSample {
        island_name: Some(island.name),
        kind: "plateau_arrival_ruin",
        label: "plateau arrival ruin marker",
        expected_material: "stone",
        material_variant: "stone_ruin",
        world_position: ruin_center
            + Quat::from_rotation_y(-0.42) * Vec3::new(ruin_width * 0.35, -ruin_height * 0.12, 0.0),
    });

    samples
}

fn player_wind_shear_visual_label(kind: PlayerAirflowVisualKind) -> &'static str {
    match kind {
        PlayerAirflowVisualKind::FrontPressure => "player wind front pressure",
        PlayerAirflowVisualKind::BodyWrap => "player wind body wrap",
        PlayerAirflowVisualKind::SideShear => "player wind side shear",
        PlayerAirflowVisualKind::ShoulderVortex => "player wind shoulder vortex",
        PlayerAirflowVisualKind::WingtipVortex => "player wind wingtip vortex",
        PlayerAirflowVisualKind::WakeTurbulence => "player wind wake turbulence",
    }
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

fn island_terrain_surface_sample_positions(island: SkyIsland) -> [Vec3; 5] {
    [
        Vec2::ZERO,
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
