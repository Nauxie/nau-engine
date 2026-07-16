use super::markers::SemanticRouteMarker;
use super::occlusion::{SemanticMarkerOcclusion, marker_occlusion_between};
use super::samples::SemanticSceneSample;
use crate::content_export::{
    terrain_export_json_number, terrain_export_json_string, terrain_export_json_vec3,
};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticMarkerVisibility {
    Visible,
    Occluded,
    Offscreen,
    BehindCamera,
}

impl SemanticMarkerVisibility {
    fn label(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Occluded => "occluded",
            Self::Offscreen => "offscreen",
            Self::BehindCamera => "behind_camera",
        }
    }
}

pub(super) fn checkpoint_marker_projection_json(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    markers: &[SemanticRouteMarker],
    islands: &[SkyIsland],
) -> (Option<Vec2>, Vec<String>, usize, usize, usize, bool) {
    let viewport_size = camera.logical_viewport_size();
    let camera_position = camera_transform.translation();
    let mut visible_count = 0usize;
    let mut in_viewport_count = 0usize;
    let mut occluded_count = 0usize;
    let mut current_objective_visible = false;
    let entries = markers
        .iter()
        .map(|marker| {
            let projected = camera
                .world_to_viewport_with_depth(camera_transform, marker.world_position)
                .ok();
            let in_viewport = projected
                .zip(viewport_size)
                .is_some_and(|(screen, viewport)| {
                    screen.x >= 0.0
                        && screen.y >= 0.0
                        && screen.x <= viewport.x
                        && screen.y <= viewport.y
                        && screen.z.is_finite()
                        && screen.z > 0.0
                });
            let behind_camera =
                projected.is_some_and(|screen| screen.z.is_finite() && screen.z <= 0.0);
            let occlusion = in_viewport
                .then(|| marker_occlusion_between(camera_position, marker.world_position, islands))
                .flatten();
            let visibility = marker_visibility(behind_camera, in_viewport, occlusion);
            if in_viewport {
                in_viewport_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Occluded {
                occluded_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Visible {
                visible_count += 1;
                current_objective_visible |= marker.current_objective;
            }

            let screen_json = projected
                .map(|screen| {
                    format!(
                        "{{\"x\": {}, \"y\": {}, \"depth_m\": {}}}",
                        terrain_export_json_number(screen.x),
                        terrain_export_json_number(screen.y),
                        terrain_export_json_number(screen.z)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let occluder_json = occluder_json(occlusion);
            let camera_distance_m = marker.world_position.distance(camera_position);

            format!(
                "{{\"kind\": {}, \"label\": {}, \"current_objective\": {}, \"world\": {}, \"screen\": {}, \"in_viewport\": {}, \"visibility\": {}, \"occluder\": {}, \"camera_distance_m\": {}}}",
                terrain_export_json_string(marker.kind),
                terrain_export_json_string(marker.label),
                marker.current_objective,
                terrain_export_json_vec3(marker.world_position),
                screen_json,
                in_viewport,
                terrain_export_json_string(visibility.label()),
                occluder_json,
                terrain_export_json_number(camera_distance_m)
            )
        })
        .collect();

    (
        viewport_size,
        entries,
        visible_count,
        in_viewport_count,
        occluded_count,
        current_objective_visible,
    )
}

pub(super) fn checkpoint_scene_sample_projection_json(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    samples: &[SemanticSceneSample],
    scene_islands: &[SkyIsland],
) -> (Vec<String>, usize, usize, usize, usize, usize) {
    let viewport_size = camera.logical_viewport_size();
    let camera_position = camera_transform.translation();
    let mut visible_count = 0usize;
    let mut in_viewport_count = 0usize;
    let mut occluded_count = 0usize;
    let mut visible_wind_count = 0usize;
    let mut visible_materials = HashSet::new();
    let entries = samples
        .iter()
        .map(|sample| {
            let projected = camera
                .world_to_viewport_with_depth(camera_transform, sample.world_position)
                .ok();
            let in_viewport = projected
                .zip(viewport_size)
                .is_some_and(|(screen, viewport)| {
                    screen.x >= 0.0
                        && screen.y >= 0.0
                        && screen.x <= viewport.x
                        && screen.y <= viewport.y
                        && screen.z.is_finite()
                        && screen.z > 0.0
                });
            let behind_camera =
                projected.is_some_and(|screen| screen.z.is_finite() && screen.z <= 0.0);
            let occlusion = in_viewport
                .then(|| {
                    marker_occlusion_between(camera_position, sample.world_position, scene_islands)
                })
                .flatten()
                .filter(|occlusion| {
                    sample.expected_material != "terrain" || occlusion.island_name != sample.label
                })
                .filter(|occlusion| {
                    !waterfall_attached_to_occluding_island(sample, *occlusion, scene_islands)
                });
            let visibility = marker_visibility(behind_camera, in_viewport, occlusion);
            if in_viewport {
                in_viewport_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Occluded {
                occluded_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Visible {
                visible_count += 1;
                visible_materials.insert(sample.expected_material);
                if sample.expected_material == "wind" {
                    visible_wind_count += 1;
                }
            }

            let screen_json = projected
                .map(|screen| {
                    format!(
                        "{{\"x\": {}, \"y\": {}, \"depth_m\": {}}}",
                        terrain_export_json_number(screen.x),
                        terrain_export_json_number(screen.y),
                        terrain_export_json_number(screen.z)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let occluder_json = occluder_json(occlusion);
            let camera_distance_m = sample.world_position.distance(camera_position);
            let island_json = sample
                .island_name
                .map(terrain_export_json_string)
                .unwrap_or_else(|| "null".to_string());

            format!(
                "{{\"island\": {}, \"kind\": {}, \"label\": {}, \"expected_material\": {}, \"material_variant\": {}, \"world\": {}, \"screen\": {}, \"in_viewport\": {}, \"visibility\": {}, \"occluder\": {}, \"camera_distance_m\": {}}}",
                island_json,
                terrain_export_json_string(sample.kind),
                terrain_export_json_string(sample.label),
                terrain_export_json_string(sample.expected_material),
                terrain_export_json_string(sample.material_variant),
                terrain_export_json_vec3(sample.world_position),
                screen_json,
                in_viewport,
                terrain_export_json_string(visibility.label()),
                occluder_json,
                terrain_export_json_number(camera_distance_m)
            )
        })
        .collect();

    (
        entries,
        visible_count,
        in_viewport_count,
        occluded_count,
        visible_materials.len(),
        visible_wind_count,
    )
}

fn marker_visibility(
    behind_camera: bool,
    in_viewport: bool,
    occlusion: Option<SemanticMarkerOcclusion>,
) -> SemanticMarkerVisibility {
    if behind_camera {
        SemanticMarkerVisibility::BehindCamera
    } else if !in_viewport {
        SemanticMarkerVisibility::Offscreen
    } else if occlusion.is_some() {
        SemanticMarkerVisibility::Occluded
    } else {
        SemanticMarkerVisibility::Visible
    }
}

fn occluder_json(occlusion: Option<SemanticMarkerOcclusion>) -> String {
    occlusion
        .map(|occlusion| {
            format!(
                "{{\"kind\": \"sky_island\", \"label\": {}, \"distance_m\": {}}}",
                terrain_export_json_string(occlusion.island_name),
                terrain_export_json_number(occlusion.distance_m)
            )
        })
        .unwrap_or_else(|| "null".to_string())
}

fn waterfall_attached_to_occluding_island(
    sample: &SemanticSceneSample,
    occlusion: SemanticMarkerOcclusion,
    islands: &[SkyIsland],
) -> bool {
    if sample.island_name != Some(occlusion.island_name) {
        return false;
    }
    if matches!(sample.kind, "water_surface" | "river_channel")
        || sample.kind.starts_with("water_detail_")
    {
        return true;
    }
    if sample.kind != "waterfall_water" {
        return false;
    }

    islands
        .iter()
        .min_by(|left, right| {
            normalized_island_radius(**left, sample.world_position)
                .total_cmp(&normalized_island_radius(**right, sample.world_position))
        })
        .is_some_and(|island| {
            island.name == occlusion.island_name
                && normalized_island_radius(*island, sample.world_position) >= 0.68
                && sample.world_position.y <= island.floor_y() + 2.0
        })
}

fn normalized_island_radius(island: SkyIsland, position: Vec3) -> f32 {
    Vec2::new(
        (position.x - island.center.x) / island.half_extents.x.max(0.001),
        (position.z - island.center.z) / island.half_extents.y.max(0.001),
    )
    .length()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owning_island_coarse_volume_does_not_hide_attached_water_samples() {
        let plateau = SkyIsland::new(
            "great sky plateau",
            Vec3::ZERO,
            Vec2::new(230.0, 155.0),
            72.0,
            false,
        );
        let sample = SemanticSceneSample {
            island_name: Some("great sky plateau"),
            kind: "waterfall_water",
            label: "broken edge waterfall",
            expected_material: "water",
            material_variant: "water",
            world_position: Vec3::new(180.0, -30.0, 42.0),
        };
        let owning_occlusion = SemanticMarkerOcclusion {
            island_name: "great sky plateau",
            distance_m: 120.0,
        };
        let foreign_occlusion = SemanticMarkerOcclusion {
            island_name: "other island",
            distance_m: 120.0,
        };

        assert!(waterfall_attached_to_occluding_island(
            &sample,
            owning_occlusion,
            &[plateau]
        ));
        for kind in ["water_detail_waterfall_lip", "water_detail_plunge_pool"] {
            let edge_detail = SemanticSceneSample { kind, ..sample };
            assert!(waterfall_attached_to_occluding_island(
                &edge_detail,
                owning_occlusion,
                &[plateau]
            ));
        }
        let lake_detail = SemanticSceneSample {
            kind: "water_detail_lily_pad",
            ..sample
        };
        assert!(waterfall_attached_to_occluding_island(
            &lake_detail,
            owning_occlusion,
            &[plateau]
        ));
        let lake_surface = SemanticSceneSample {
            kind: "water_surface",
            world_position: Vec3::ZERO,
            ..sample
        };
        assert!(waterfall_attached_to_occluding_island(
            &lake_surface,
            owning_occlusion,
            &[plateau]
        ));
        let terrain = SemanticSceneSample {
            kind: "terrain_surface",
            ..sample
        };
        assert!(!waterfall_attached_to_occluding_island(
            &terrain,
            owning_occlusion,
            &[plateau]
        ));
        assert!(!waterfall_attached_to_occluding_island(
            &sample,
            foreign_occlusion,
            &[plateau]
        ));
    }
}
