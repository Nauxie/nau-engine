use crate::{
    artifact::{audit_obj_text, audit_weight_csv_text},
    manifest::audit_manifest,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn passing_visual_collision_coverage(island_count: u64) -> Value {
    json!({
        "schema": "nau_visual_collision_coverage.v2",
        "passed": true,
        "checked_visual_count": 500,
        "solid_visual_count": 340,
        "surface_supported_solid_proxy_count": 330,
        "footprint_bounded_solid_proxy_count": 330,
        "min_solid_proxy_edge_clearance_m": 0.25,
        "tree_solid_proxy_count": 110,
        "tree_footprint_bounded_proxy_count": 110,
        "rock_solid_proxy_count": 160,
        "rock_footprint_bounded_proxy_count": 160,
        "landmark_solid_proxy_count": 60,
        "landmark_footprint_bounded_proxy_count": 60,
        "obstacle_bounded_solid_proxy_count": 200,
        "terrain_rim_proxy_count": island_count * 16,
        "terrain_body_proxy_count": island_count * 4,
        "camera_only_allowance_count": island_count,
        "non_blocking_visual_count": 80,
        "failure_count": 0,
        "failures": []
    })
}

fn passing_seam_coverage(island_count: u64) -> Value {
    json!({
        "schema": "nau_terrain_seam_coverage.v1",
        "island_count": island_count,
        "max_terrain_cliff_top_gap_m": 0.0,
        "min_terrain_edge_skirt_depth_m": 0.32,
        "max_terrain_edge_skirt_horizontal_gap_m": 0.0
    })
}

fn layout_island(
    name: impl Into<String>,
    center: [f64; 3],
    half_extents: [f64; 2],
    thickness_m: f64,
) -> Value {
    json!({
        "name": name.into(),
        "center": center,
        "half_extents": half_extents,
        "thickness_m": thickness_m,
    })
}

fn layout_manifest(islands: Vec<Value>) -> Value {
    let island_count = islands.len() as u64;
    json!({
        "schema": "nau_terrain_export.v1",
        "island_count": island_count,
        "islands": islands,
    })
}

fn passing_layout_islands() -> Vec<Value> {
    let half_extents = [
        (10.0, 10.0),
        (10.0, 20.0),
        (5.0, 40.0),
        (15.0, 20.0),
        (18.0, 24.0),
        (20.0, 25.0),
        (20.0, 40.0),
        (25.0, 40.0),
        (30.0, 39.0),
        (30.0, 40.0),
        (30.0, 50.0),
        (35.0, 50.0),
        (40.0, 50.0),
        (40.0, 60.0),
        (45.0, 55.0),
        (35.0, 70.0),
        (30.0, 85.0),
        (50.0, 52.0),
        (50.0, 70.0),
        (60.0, 75.0),
        (50.0, 100.0),
        (70.0, 80.0),
        (40.0, 145.0),
        (60.0, 100.0),
        (80.0, 100.0),
        (75.0, 120.0),
        (70.0, 160.0),
        (100.0, 120.0),
        (100.0, 150.0),
        (120.0, 140.0),
        (80.0, 220.0),
        (100.0, 200.0),
    ];
    let sector_centers = [
        [1_000.0, 40.0, 200.0],
        [800.0, 80.0, 800.0],
        [-200.0, 1_050.0, 1_800.0],
        [-800.0, 120.0, 800.0],
        [-1_000.0, 160.0, -200.0],
        [-800.0, 200.0, -800.0],
        [200.0, 240.0, -1_800.0],
        [800.0, 280.0, -800.0],
    ];

    half_extents
        .into_iter()
        .enumerate()
        .map(|(index, (half_x, half_z))| {
            let name = if index == 0 {
                "launch mesa".to_string()
            } else {
                format!("layout island {index}")
            };
            let center = if index == 0 {
                [0.0, 0.0, 0.0]
            } else if index <= sector_centers.len() {
                sector_centers[index - 1]
            } else {
                [
                    (index as f64 % 7.0 - 3.0) * 80.0,
                    80.0 + (index as f64 % 11.0) * 35.0,
                    -300.0 - index as f64 * 40.0,
                ]
            };
            layout_island(name, center, [half_x, half_z], 10.0 + (index % 5) as f64)
        })
        .collect()
}

#[test]
fn obj_audit_counts_vertices_faces_and_vertex_colors() {
    let audit = audit_obj_text(
        "# sample\n\
             v 0.0 0.0 0.0 0.1 0.2 0.3\n\
             v 1.0 0.0 0.0 0.1 0.2 0.3\n\
             v 0.0 1.0 0.0\n\
             vn 0.0 1.0 0.0\n\
             f 1//1 2//1 3//1\n",
    );

    assert_eq!(audit.vertex_count, 3);
    assert_eq!(audit.face_count, 1);
    assert_eq!(audit.colored_vertex_count, 2);
    assert_eq!(audit.vertical_range_m, 1.0);
    assert_eq!(audit.vertical_band_count, 2);
    assert_eq!(audit.normal_slope_band_count, 1);
    assert_eq!(audit.horizontal_radius_bands, 2);
    assert_eq!(audit.silhouette_radius_bands, 1);
}

#[test]
fn obj_audit_tracks_vertical_mass_and_radius_variation() {
    let audit = audit_obj_text(
        "# sample\n\
             v 0.0 0.0 0.0 0.1 0.2 0.3\n\
             v 3.0 0.0 0.0 0.1 0.2 0.3\n\
             v -2.0 0.0 0.0 0.1 0.2 0.3\n\
             v 0.0 -9.0 0.0 0.1 0.2 0.3\n\
             v 0.0 -4.0 1.5 0.1 0.2 0.3\n\
             f 1 2 5\n\
             f 1 5 4\n",
    );

    assert_eq!(audit.vertical_range_m, 9.0);
    assert_eq!(audit.vertical_band_count, 3);
    assert_eq!(audit.normal_slope_band_count, 0);
    assert!(
        audit.horizontal_radius_bands >= 3,
        "radius bands should reflect broad, shoulder, and center mass"
    );
    assert!(
        audit.silhouette_radius_bands >= 2,
        "silhouette bands should track outer radius variation"
    );
}

#[test]
fn obj_audit_ignores_downward_normals_for_slope_bands() {
    let audit = audit_obj_text(
        "# flipped terrain normals\n\
             v 0.0 0.0 0.0 0.1 0.2 0.3\n\
             v 1.0 0.0 0.0 0.1 0.2 0.3\n\
             v 0.0 1.0 0.0 0.1 0.2 0.3\n\
             vn 0.0 -1.0 0.0\n\
             vn 0.5 -0.8660 0.0\n\
             vn 0.7071 -0.7071 0.0\n\
             f 1//1 2//2 3//3\n",
    );

    assert_eq!(audit.normal_slope_band_count, 0);
}

#[test]
fn audit_manifest_compares_terrain_band_metrics_to_obj_artifact() {
    let root = std::env::temp_dir().join(format!(
        "nau-terrain-audit-band-mismatch-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("temp audit dir should be created");
    fs::write(
        root.join("terrain.obj"),
        "# terrain\n\
             v 0.0 0.00 0.0 0.1 0.2 0.3\n\
             v 1.0 0.05 0.0 0.1 0.2 0.3\n\
             v 0.0 0.10 1.0 0.1 0.2 0.3\n\
             vn 0.0 1.0 0.0\n\
             vn 0.5 0.8660 0.0\n\
             vn 0.7071 0.7071 0.0\n\
             f 1//1 2//2 3//3\n",
    )
    .expect("terrain obj should be written");

    let manifest = json!({
        "schema": "nau_terrain_export.v1",
        "island_count": 1,
        "mesh_count": 4,
        "total_vertex_count": 2305,
        "total_triangle_count": 4000,
        "minimums": {
            "terrain_mesh_vertices": 2305,
            "terrain_color_bands": 32,
            "terrain_material_weight_bands": 24,
            "terrain_material_channels": 3,
            "terrain_material_regions": 4,
            "terrain_height_bands": 19,
            "terrain_normal_slope_bands": 10,
            "terrain_texture_detail_bands": 50,
            "terrain_texture_edge_promille": 590,
            "terrain_relief_range_m": 0.8,
            "cliff_color_bands": 9,
            "impostor_mesh_vertices": 140,
            "impostor_color_bands": 18
        },
        "islands": [{
            "name": "launch mesa",
            "terrain": {
                "obj": "terrain.obj",
                "material_weights_csv": "missing_weights.csv",
                "vertex_count": 3,
                "triangle_count": 1,
                "material_weight_bands": 24,
                "material_channels": 3,
                "material_regions": 4,
                "height_bands": 99,
                "normal_slope_bands": 99
            },
            "cliff": {"obj": "missing_cliff.obj", "vertex_count": 96, "triangle_count": 180},
            "underside": {"obj": "missing_underside.obj", "vertex_count": 96, "triangle_count": 180},
            "impostor": {"obj": "missing_impostor.obj", "vertex_count": 140, "triangle_count": 200}
        }]
    });

    let report = audit_manifest(&manifest, &root, "manifest.json");

    assert!(!audit_check_passed(
        &report,
        "obj_height_band_mismatch_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "obj_normal_slope_band_mismatch_count"
    ));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn audit_manifest_requires_impostor_entries_and_minimums() {
    let manifest = json!({
        "schema": "nau_terrain_export.v1",
        "island_count": 1,
        "mesh_count": 3,
        "total_vertex_count": 2305,
        "total_triangle_count": 4000,
        "minimums": {
            "terrain_mesh_vertices": 2305,
            "terrain_color_bands": 32,
            "terrain_material_weight_bands": 24,
            "terrain_material_channels": 3,
            "terrain_material_regions": 4,
            "terrain_height_bands": 8,
            "terrain_normal_slope_bands": 3,
            "terrain_texture_detail_bands": 50,
            "terrain_texture_edge_promille": 120,
            "terrain_relief_range_m": 0.8,
            "cliff_color_bands": 9,
            "impostor_mesh_vertices": 42,
            "impostor_color_bands": 4
        },
        "islands": [{
            "name": "launch mesa",
            "terrain": {
                "obj": "missing_terrain.obj",
                "material_weights_csv": "missing_weights.csv",
                "vertex_count": 2305,
                "triangle_count": 4000,
                "material_weight_bands": 24,
                "material_channels": 3,
                "material_regions": 4
            },
            "cliff": {
                "obj": "missing_cliff.obj",
                "vertex_count": 96,
                "triangle_count": 180
            },
            "underside": {
                "obj": "missing_underside.obj",
                "vertex_count": 96,
                "triangle_count": 180
            }
        }]
    });
    let report = audit_manifest(&manifest, Path::new("."), "manifest.json");

    assert!(
        !report
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    );
    assert!(!audit_check_passed(&report, "mesh_count"));
    assert!(!audit_check_passed(&report, "terrain_height_bands"));
    assert!(!audit_check_passed(&report, "terrain_normal_slope_bands"));
    assert!(!audit_check_passed(
        &report,
        "terrain_texture_edge_promille"
    ));
    assert!(!audit_check_passed(&report, "terrain_archetype_count"));
    assert!(!audit_check_passed(&report, "impostor_mesh_vertices"));
    assert!(!audit_check_passed(&report, "impostor_color_bands"));
    assert!(!audit_check_passed(&report, "impostor_vertical_range"));
    assert!(!audit_check_passed(
        &report,
        "impostor_horizontal_radius_bands"
    ));
    assert!(!audit_check_passed(
        &report,
        "terrain_silhouette_radius_bands"
    ));
    assert!(!audit_check_passed(&report, "terrain_obj_height_bands"));
    assert!(!audit_check_passed(
        &report,
        "terrain_obj_normal_slope_bands"
    ));
    assert!(!audit_check_passed(&report, "island_body_vertical_range"));
    assert!(!audit_check_passed(
        &report,
        "island_body_silhouette_radius_bands"
    ));
    assert!(!audit_check_passed(
        &report,
        "terrain_aggregate_base_region_promille"
    ));
    assert!(!audit_check_passed(
        &report,
        "terrain_aggregate_transition_region_promille"
    ));
    assert!(!audit_check_passed(
        &report,
        "terrain_aggregate_highland_region_promille"
    ));
    assert!(!audit_check_passed(
        &report,
        "terrain_aggregate_exposed_region_promille"
    ));
    assert!(!audit_check_passed(&report, "collision_truth_schema"));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_top_edge_probe_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_edge_traverse_probe_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_near_cliff_probe_count"
    ));
    assert!(
        report
            .get("artifacts")
            .and_then(Value::as_array)
            .expect("artifacts should be present")
            .iter()
            .any(
                |artifact| artifact.get("kind").and_then(Value::as_str) == Some("impostor")
                    && artifact.get("error").and_then(Value::as_str) == Some("missing obj path")
            )
    );
}

#[test]
fn audit_manifest_rejects_collision_truth_barriers_and_cliff_gaps() {
    let manifest = json!({
        "schema": "nau_terrain_export.v1",
        "island_count": 2,
        "terrain_archetype_count": 19,
        "mesh_count": 8,
        "total_vertex_count": 4610,
        "total_triangle_count": 8000,
        "minimums": {
            "terrain_mesh_vertices": 2305,
            "terrain_color_bands": 32,
            "terrain_material_weight_bands": 24,
            "terrain_material_channels": 3,
            "terrain_material_regions": 4,
            "terrain_height_bands": 19,
            "terrain_normal_slope_bands": 10,
            "terrain_texture_detail_bands": 50,
            "terrain_texture_edge_promille": 590,
            "terrain_relief_range_m": 0.8,
            "cliff_color_bands": 9,
            "impostor_mesh_vertices": 140,
            "impostor_color_bands": 18
        },
        "collision_truth": {
            "schema": "nau_terrain_collision_truth.v1",
            "island_count": 2,
            "contour_sample_count": 32,
            "top_edge_probe_count": 64,
            "top_edge_air_barrier_count": 1,
            "edge_traverse_probe_count": 1152,
            "edge_traverse_barrier_count": 1,
            "walkoff_shoulder_probe_count": 576,
            "walkoff_shoulder_barrier_count": 1,
            "far_field_probe_count": 64,
            "far_field_hit_count": 1,
            "near_cliff_probe_count": 64,
            "near_cliff_miss_count": 1,
            "excessive_near_cliff_push_count": 1,
            "max_top_edge_push_m": 0.0,
            "max_edge_traverse_push_m": 0.2,
            "max_walkoff_shoulder_push_m": 0.2,
            "max_far_field_push_m": 0.0,
            "max_near_cliff_push_m": 0.5
        },
        "visual_collision_coverage": passing_visual_collision_coverage(2),
        "seam_coverage": passing_seam_coverage(2),
        "islands": []
    });
    let report = audit_manifest(&manifest, Path::new("."), "manifest.json");

    assert!(audit_check_passed(&report, "collision_truth_schema"));
    assert!(audit_check_passed(
        &report,
        "collision_truth_top_edge_probe_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_top_edge_air_barrier_count"
    ));
    assert!(audit_check_passed(
        &report,
        "collision_truth_edge_traverse_probe_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_edge_traverse_barrier_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_max_edge_traverse_push"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_walkoff_shoulder_barrier_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_max_walkoff_shoulder_push"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_far_field_hit_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_near_cliff_miss_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_excessive_near_cliff_push_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "collision_truth_max_near_cliff_push"
    ));
}

#[test]
fn audit_manifest_rejects_hollow_terrain_seam_regressions() {
    let manifest = json!({
        "schema": "nau_terrain_export.v1",
        "island_count": 2,
        "terrain_archetype_count": 19,
        "mesh_count": 8,
        "total_vertex_count": 4610,
        "total_triangle_count": 8000,
        "minimums": {
            "terrain_mesh_vertices": 2305,
            "terrain_color_bands": 32,
            "terrain_material_weight_bands": 24,
            "terrain_material_channels": 3,
            "terrain_material_regions": 4,
            "terrain_height_bands": 19,
            "terrain_normal_slope_bands": 10,
            "terrain_texture_detail_bands": 50,
            "terrain_texture_edge_promille": 590,
            "terrain_relief_range_m": 0.8,
            "cliff_color_bands": 9,
            "impostor_mesh_vertices": 140,
            "impostor_color_bands": 18
        },
        "collision_truth": {
            "schema": "nau_terrain_collision_truth.v1",
            "island_count": 2,
            "contour_sample_count": 32,
            "top_edge_probe_count": 64,
            "top_edge_air_barrier_count": 0,
            "edge_traverse_probe_count": 1152,
            "edge_traverse_barrier_count": 0,
            "walkoff_shoulder_probe_count": 576,
            "walkoff_shoulder_barrier_count": 0,
            "far_field_probe_count": 64,
            "far_field_hit_count": 0,
            "near_cliff_probe_count": 64,
            "near_cliff_miss_count": 0,
            "excessive_near_cliff_push_count": 0,
            "max_top_edge_push_m": 0.0,
            "max_edge_traverse_push_m": 0.0,
            "max_walkoff_shoulder_push_m": 0.0,
            "max_far_field_push_m": 0.0,
            "max_near_cliff_push_m": 0.2
        },
        "visual_collision_coverage": passing_visual_collision_coverage(2),
        "seam_coverage": {
            "schema": "nau_terrain_seam_coverage.v1",
            "island_count": 2,
            "max_terrain_cliff_top_gap_m": 0.02,
            "min_terrain_edge_skirt_depth_m": 0.04,
            "max_terrain_edge_skirt_horizontal_gap_m": 0.03
        },
        "islands": [
            {
                "name": "launch mesa",
                "seam": {
                    "max_terrain_cliff_top_gap_m": 0.02,
                    "min_terrain_edge_skirt_depth_m": 0.04,
                    "max_terrain_edge_skirt_horizontal_gap_m": 0.03
                }
            },
            {
                "name": "landing garden",
                "seam": {
                    "max_terrain_cliff_top_gap_m": 0.004,
                    "min_terrain_edge_skirt_depth_m": 0.10,
                    "max_terrain_edge_skirt_horizontal_gap_m": 0.008
                }
            }
        ]
    });
    let report = audit_manifest(&manifest, Path::new("."), "manifest.json");

    assert!(audit_check_passed(&report, "terrain_seam_coverage_schema"));
    assert!(audit_check_passed(
        &report,
        "terrain_seam_coverage_island_count"
    ));
    assert!(!audit_check_passed(&report, "terrain_cliff_top_seam_gap"));
    assert!(!audit_check_passed(&report, "terrain_edge_skirt_depth"));
    assert!(!audit_check_passed(
        &report,
        "terrain_edge_skirt_horizontal_gap"
    ));
    assert!(!audit_check_passed(
        &report,
        "island_terrain_cliff_top_seam_gap"
    ));
    assert!(!audit_check_passed(
        &report,
        "island_terrain_edge_skirt_depth"
    ));
    assert!(!audit_check_passed(
        &report,
        "island_terrain_edge_skirt_horizontal_gap"
    ));
}

#[test]
fn audit_manifest_rejects_visual_collision_coverage_regressions() {
    let manifest = json!({
        "schema": "nau_terrain_export.v1",
        "island_count": 2,
        "terrain_archetype_count": 19,
        "mesh_count": 8,
        "total_vertex_count": 4610,
        "total_triangle_count": 8000,
        "minimums": {
            "terrain_mesh_vertices": 2305,
            "terrain_color_bands": 32,
            "terrain_material_weight_bands": 24,
            "terrain_material_channels": 3,
            "terrain_material_regions": 4,
            "terrain_height_bands": 19,
            "terrain_normal_slope_bands": 10,
            "terrain_texture_detail_bands": 50,
            "terrain_texture_edge_promille": 590,
            "terrain_relief_range_m": 0.8,
            "cliff_color_bands": 9,
            "impostor_mesh_vertices": 140,
            "impostor_color_bands": 18
        },
        "collision_truth": {
            "schema": "nau_terrain_collision_truth.v1",
            "island_count": 2,
            "contour_sample_count": 32,
            "top_edge_probe_count": 64,
            "top_edge_air_barrier_count": 0,
            "edge_traverse_probe_count": 1152,
            "edge_traverse_barrier_count": 0,
            "walkoff_shoulder_probe_count": 576,
            "walkoff_shoulder_barrier_count": 0,
            "far_field_probe_count": 64,
            "far_field_hit_count": 0,
            "near_cliff_probe_count": 64,
            "near_cliff_miss_count": 0,
            "excessive_near_cliff_push_count": 0,
            "max_top_edge_push_m": 0.0,
            "max_edge_traverse_push_m": 0.0,
            "max_walkoff_shoulder_push_m": 0.0,
            "max_far_field_push_m": 0.0,
            "max_near_cliff_push_m": 0.2
        },
        "visual_collision_coverage": {
            "schema": "nau_visual_collision_coverage.v2",
            "passed": false,
            "checked_visual_count": 240,
            "solid_visual_count": 50,
            "surface_supported_solid_proxy_count": 50,
            "footprint_bounded_solid_proxy_count": 49,
            "min_solid_proxy_edge_clearance_m": -0.2,
            "tree_solid_proxy_count": 90,
            "tree_footprint_bounded_proxy_count": 89,
            "rock_solid_proxy_count": 140,
            "rock_footprint_bounded_proxy_count": 139,
            "landmark_solid_proxy_count": 45,
            "landmark_footprint_bounded_proxy_count": 44,
            "obstacle_bounded_solid_proxy_count": 45,
            "terrain_rim_proxy_count": 63,
            "terrain_body_proxy_count": 7,
            "camera_only_allowance_count": 1,
            "non_blocking_visual_count": 50,
            "failure_count": 1,
            "failures": ["edge spill"]
        },
        "islands": []
    });
    let report = audit_manifest(&manifest, Path::new("."), "manifest.json");

    assert!(audit_check_passed(
        &report,
        "visual_collision_coverage_schema"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_coverage_passed"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_coverage_failure_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_solid_visual_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_surface_supported_solid_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_footprint_bounded_solid_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_min_solid_proxy_edge_clearance_m"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_tree_solid_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_tree_footprint_bounded_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_rock_solid_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_rock_footprint_bounded_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_landmark_solid_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_landmark_footprint_bounded_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_terrain_body_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_terrain_rim_proxy_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "visual_collision_camera_only_allowance_count"
    ));
}

#[test]
fn audit_manifest_accepts_diverse_archipelago_layout_metrics() {
    let manifest = layout_manifest(passing_layout_islands());
    let report = audit_manifest(&manifest, Path::new("."), "manifest.json");

    for check in [
        "archipelago_malformed_layout_island_count",
        "archipelago_x_span",
        "archipelago_y_span",
        "archipelago_z_span",
        "archipelago_positive_z_island_count",
        "archipelago_horizontal_sector_coverage",
        "archipelago_area_ratio",
        "archipelago_tiny_area_band_count",
        "archipelago_small_area_band_count",
        "archipelago_medium_area_band_count",
        "archipelago_large_area_band_count",
        "archipelago_vast_area_band_count",
        "archipelago_colossal_area_band_count",
    ] {
        assert!(audit_check_passed(&report, check), "{check} should pass");
    }

    let metrics = report
        .get("layout_metrics")
        .expect("layout metrics should be present");
    assert_eq!(
        metrics.get("x_span_m").and_then(Value::as_f64),
        Some(2_000.0)
    );
    assert_eq!(
        metrics.get("y_span_m").and_then(Value::as_f64),
        Some(1_050.0)
    );
    assert_eq!(
        metrics.get("z_span_m").and_then(Value::as_f64),
        Some(3_600.0)
    );
    assert_eq!(
        metrics
            .get("horizontal_sector_coverage")
            .and_then(Value::as_u64),
        Some(8)
    );
    assert_eq!(
        metrics.get("area_ratio").and_then(Value::as_f64),
        Some(200.0)
    );
    assert_eq!(
        metrics.get("area_bands"),
        Some(&json!({
            "tiny": 5,
            "small": 4,
            "medium": 8,
            "large": 6,
            "vast": 8,
            "colossal": 1,
        }))
    );
    let aspect_buckets = metrics
        .get("aspect_ratio_buckets")
        .and_then(Value::as_object)
        .expect("aspect-ratio buckets should be present");
    assert_eq!(
        aspect_buckets
            .values()
            .filter_map(Value::as_u64)
            .sum::<u64>(),
        32
    );
    assert!(
        aspect_buckets
            .values()
            .all(|count| count.as_u64().unwrap_or(0) > 0)
    );
}

#[test]
fn audit_manifest_rejects_collapsed_corridor_and_size_regression() {
    let islands = (0..32)
        .map(|index| {
            layout_island(
                if index == 0 {
                    "launch mesa".to_string()
                } else {
                    format!("corridor island {index}")
                },
                [0.0, 0.0, -(index as f64) * 120.0],
                [40.0, 40.0],
                12.0,
            )
        })
        .collect();
    let report = audit_manifest(&layout_manifest(islands), Path::new("."), "manifest.json");

    assert!(audit_check_passed(
        &report,
        "archipelago_malformed_layout_island_count"
    ));
    assert!(!audit_check_passed(&report, "archipelago_x_span"));
    assert!(!audit_check_passed(&report, "archipelago_y_span"));
    assert!(audit_check_passed(&report, "archipelago_z_span"));
    assert!(!audit_check_passed(
        &report,
        "archipelago_positive_z_island_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "archipelago_horizontal_sector_coverage"
    ));
    assert!(!audit_check_passed(&report, "archipelago_area_ratio"));
    assert!(!audit_check_passed(
        &report,
        "archipelago_tiny_area_band_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "archipelago_small_area_band_count"
    ));
    assert!(audit_check_passed(
        &report,
        "archipelago_medium_area_band_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "archipelago_large_area_band_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "archipelago_vast_area_band_count"
    ));
    assert!(!audit_check_passed(
        &report,
        "archipelago_colossal_area_band_count"
    ));
}

#[test]
fn audit_manifest_zeroes_layout_metrics_for_missing_or_malformed_vectors() {
    let manifest = layout_manifest(vec![
        layout_island("launch mesa", [0.0, 0.0, 0.0], [10.0, 10.0], 10.0),
        json!({
            "name": "malformed center",
            "center": [1.0, "bad", 3.0],
            "half_extents": [10.0, 10.0],
            "thickness_m": 10.0,
        }),
        json!({
            "name": "missing extents",
            "center": [1.0, 2.0, 3.0],
            "thickness_m": 10.0,
        }),
    ]);
    let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
    let metrics = report
        .get("layout_metrics")
        .expect("layout metrics should be present");

    assert!(!audit_check_passed(
        &report,
        "archipelago_malformed_layout_island_count"
    ));
    assert_eq!(
        metrics
            .get("malformed_island_count")
            .and_then(Value::as_u64),
        Some(2)
    );
    for metric in ["x_span_m", "y_span_m", "z_span_m", "area_ratio"] {
        assert_eq!(metrics.get(metric).and_then(Value::as_f64), Some(0.0));
    }
    assert_eq!(
        metrics.get("area_bands"),
        Some(&json!({
            "tiny": 0,
            "small": 0,
            "medium": 0,
            "large": 0,
            "vast": 0,
            "colossal": 0,
        }))
    );
}

#[test]
fn material_weight_csv_audit_counts_quantized_bands_and_channels() {
    let audit = audit_weight_csv_text(
        "vertex,lush_highland,exposed_edge\n\
             0,0.0000,0.0000\n\
             1,0.3000,0.0000\n\
             2,0.7000,0.0000\n\
             3,0.1000,0.8000\n\
             4,0.0000,0.0000\n\
             5,0.3000,0.0000\n\
             6,0.3000,0.0000\n\
             7,0.3000,0.0000\n\
             8,0.7000,0.0000\n\
             9,0.1000,0.8000\n",
        None,
    )
    .expect("csv should audit");

    assert_eq!(audit.row_count, 10);
    assert_eq!(audit.region_row_count, 10);
    assert_eq!(audit.material_weight_bands, 4);
    assert_eq!(audit.material_channels, 3);
    assert_eq!(audit.material_regions, 4);
    assert_eq!(audit.region_promille, [200, 400, 200, 200]);
}

#[test]
fn material_weight_csv_audit_limits_region_promille_to_surface_rows() {
    let audit = audit_weight_csv_text(
        "vertex,lush_highland,exposed_edge\n\
             0,0.0000,0.0000\n\
             1,0.3000,0.0000\n\
             2,0.7000,0.0000\n\
             3,0.1000,0.8000\n\
             4,0.1000,0.8000\n\
             5,0.1000,0.8000\n",
        Some(4),
    )
    .expect("csv should audit");

    assert_eq!(audit.row_count, 6);
    assert_eq!(audit.region_row_count, 4);
    assert_eq!(audit.material_weight_bands, 4);
    assert_eq!(audit.material_channels, 3);
    assert_eq!(audit.material_regions, 4);
    assert_eq!(audit.region_promille, [250, 250, 250, 250]);
}

fn audit_check_passed(report: &Value, name: &str) -> bool {
    report
        .get("checks")
        .and_then(Value::as_array)
        .and_then(|checks| {
            checks
                .iter()
                .find(|check| check.get("name").and_then(Value::as_str) == Some(name))
        })
        .and_then(|check| check.get("passed").and_then(Value::as_bool))
        .unwrap_or(false)
}
