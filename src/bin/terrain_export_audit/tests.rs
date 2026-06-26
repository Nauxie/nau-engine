use crate::{
    artifact::{audit_obj_text, audit_weight_csv_text},
    manifest::audit_manifest,
};
use serde_json::{Value, json};
use std::path::Path;

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
            "terrain_texture_detail_bands": 44,
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
    assert!(!audit_check_passed(
        &report,
        "terrain_texture_edge_promille"
    ));
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
    assert!(!audit_check_passed(&report, "island_body_vertical_range"));
    assert!(!audit_check_passed(
        &report,
        "island_body_silhouette_radius_bands"
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
    )
    .expect("csv should audit");

    assert_eq!(audit.row_count, 10);
    assert_eq!(audit.material_weight_bands, 4);
    assert_eq!(audit.material_channels, 3);
    assert_eq!(audit.material_regions, 4);
    assert_eq!(audit.region_promille, [200, 400, 200, 200]);
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
