#![recursion_limit = "256"]

use serde_json::{Value, json};
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    process,
};

const EXPECTED_VISUAL_CONTENT_EXPORT_SCHEMA: &str = "nau_visual_content_export.v2";
const EXPECTED_PALETTE_COUNT: u64 = nau_engine::world::SKY_ROUTE_ISLAND_COUNT as u64;
const MIN_PERCEPTUAL_PALETTE_DISTANCE: f64 = 0.004;
const MIN_GROUND_COVER_COUNT: u64 = 20;
const MIN_GROUND_COVER_PATCH_TOTAL: u64 = 2_400;
const MIN_GROUND_COVER_BLADE_TOTAL: u64 = 12_000;
const MIN_GROUND_COVER_PATCH_COUNT: u64 = 24;
const MIN_GROUND_COVER_MESH_VERTICES: u64 = 720;
const MIN_GROUND_COVER_BLADE_COUNT: u64 = 120;
const MIN_GROUND_COVER_BLADE_HEIGHT_RANGE_M: f64 = 0.70;
const MIN_TREE_TRUNK_COUNT: u64 = 160;
const MIN_TREE_CANOPY_COUNT: u64 = 160;
const MIN_TREE_TRUNK_MESH_VERTICES: u64 = 190;
const MIN_TREE_TRUNK_TAPER_RATIO: f64 = 1.35;
const MIN_TREE_BRANCH_REACH_RATIO: f64 = 1.80;
const MIN_TREE_BRANCH_COUNT: u64 = 4;
const MIN_TREE_ROOT_FLARE_COUNT: u64 = 5;
const MIN_TREE_TRUNK_RING_COUNT: u64 = 5;
const MIN_TREE_CANOPY_MESH_VERTICES: u64 = 450;
const MIN_TREE_CANOPY_LOBE_COUNT: u64 = 6;
const MIN_TREE_CANOPY_DETAIL_CARD_COUNT: u64 = 18;
const MIN_TREE_CANOPY_VERTICAL_TO_HORIZONTAL_RATIO: f64 = 0.45;
const MIN_ROCK_COUNT: u64 = 230;
const MIN_ROCK_MESH_VERTICES: u64 = 70;
const MIN_ROCK_VERTICAL_SPAN_M: f64 = 0.40;
const MIN_WEATHER_CLOUD_COUNT: u64 = 40;
const MIN_WEATHER_CLOUD_BANK_COUNT: u64 = 20;
const MIN_WEATHER_CLOUD_VEIL_COUNT: u64 = 30;
const MIN_WEATHER_CLOUD_MESH_VERTICES: u64 = 1530;
const MIN_WEATHER_CLOUD_LOBE_COUNT: u64 = 9;
const MIN_WEATHER_CLOUD_WISP_CARD_COUNT: u64 = 36;
const MIN_WEATHER_CLOUD_FILAMENT_RIBBON_DETAIL_COUNT: u64 = 27;
const MIN_WEATHER_CLOUD_BANK_DEPTH_M: f64 = 5.8;
const MIN_WEATHER_CLOUD_BANK_LOBE_COUNT: u64 = 18;
const MIN_WEATHER_CLOUD_SCALED_DEPTH_SPAN_M: f64 = 12.0;
const MIN_TREE_TRUNK_HEIGHT_RANGE_M: f64 = 1.5;
const MIN_TREE_CANOPY_RADIUS_RANGE_M: f64 = 0.35;
const MIN_LANDMARK_COUNT: u64 = 60;
const MIN_LANDMARK_KIND_COUNT: u64 = 18;
const MIN_FLORA_CLUSTER_COUNT: u64 = 36;
const MIN_FLORA_CLUSTER_KIND_COUNT: u64 = 5;
const MIN_RUIN_COMPLEX_COUNT: u64 = 8;
const MIN_RUIN_COMPLEX_KIND_COUNT: u64 = 4;
const MIN_ROCK_FORMATION_COUNT: u64 = 20;
const MIN_ROCK_FORMATION_KIND_COUNT: u64 = 4;
const MIN_WATER_DETAIL_COUNT: u64 = 10;
const MIN_WATER_DETAIL_KIND_COUNT: u64 = 5;
const MIN_ARTIFACT_DETAIL_COUNT: u64 = 55;
const MIN_ARTIFACT_DETAIL_KIND_COUNT: u64 = 7;
const MIN_ARTIFACT_STAIR_COUNT: u64 = 8;
const MIN_ARTIFACT_BRIDGE_FRAGMENT_COUNT: u64 = 6;
const MIN_ARTIFACT_GLYPH_SLAB_COUNT: u64 = 8;
const MIN_ARTIFACT_RETAINING_WALL_COUNT: u64 = 8;
const MIN_ARTIFACT_BANNER_COUNT: u64 = 6;
const MIN_ARTIFACT_PEBBLE_FIELD_COUNT: u64 = 30;
const MIN_ARTIFACT_REED_PATCH_COUNT: u64 = 4;
const MIN_SMALL_ISLAND_COUNT: u64 = 10;
const MIN_PLATEAU_LANDMARK_COUNT: u64 = 15;
const MIN_PLATEAU_WATERFALL_RIBBON_COUNT: u64 = 2;
const MIN_PLATEAU_WATERFALL_MIST_COUNT: u64 = 2;
const MIN_ROUTE_WATERFALL_RIBBON_COUNT: u64 = 1;
const MIN_ROUTE_WATERFALL_MIST_COUNT: u64 = 1;
const MIN_ROUTE_LAKE_SURFACE_COUNT: u64 = 3;
const MIN_RIVER_CHANNEL_COUNT: u64 = 6;
const MIN_UNDER_ROUTE_VISUAL_COUNT: u64 = 8;
const MIN_UNDER_ROUTE_CAVE_MOUTH_COUNT: u64 = 4;
const MIN_RUIN_CLUSTER_COUNT: u64 = 6;
const MIN_RUIN_ARCH_COUNT: u64 = 4;
const MIN_ROUTE_CAIRN_COUNT: u64 = 16;
const MIN_LAUNCH_BEACON_COUNT: u64 = 1;
const MIN_LANDING_GARDEN_MARKER_COUNT: u64 = 4;
const MIN_POND_SURFACE_COUNT: u64 = 5;
const MIN_OBSTRUCTION_SPIRE_COUNT: u64 = 20;
const MIN_ROUTE_CAIRN_MESH_VERTICES: u64 = 240;
const MIN_ROUTE_CAIRN_VERTICAL_SPAN_M: f64 = 3.0;
const MIN_LAUNCH_BEACON_MESH_VERTICES: u64 = 300;
const MIN_LAUNCH_BEACON_VERTICAL_SPAN_M: f64 = 2.8;
const MIN_LANDING_GARDEN_MARKER_MESH_VERTICES: u64 = 39;
const MIN_LANDING_GARDEN_MARKER_VERTICAL_SPAN_M: f64 = 0.12;
const MIN_POND_SURFACE_MESH_VERTICES: u64 = 65;
const MIN_POND_SURFACE_VERTICAL_SPAN_M: f64 = 0.015;
const MIN_PLATEAU_LANDMARK_VERTEX_TOTAL: u64 = 2_500;
const MIN_MAX_PLATEAU_LANDMARK_MESH_VERTICES: u64 = 600;
const MIN_PLATEAU_WATERFALL_VERTICAL_SPAN_M: f64 = 45.0;
const MIN_ROUTE_WATERFALL_VERTICAL_SPAN_M: f64 = 18.0;
const MIN_ROUTE_LAKE_SURFACE_HORIZONTAL_SPAN_M: f64 = 18.0;
const MIN_RIVER_CHANNEL_HORIZONTAL_SPAN_M: f64 = 4.0;
const MIN_UNDER_ROUTE_VISUAL_VERTICAL_SPAN_M: f64 = 4.0;
const MIN_SURFACE_FEATURE_VERTEX_TOTAL: u64 = 15_000;
const MIN_FLORA_CLUSTER_MESH_VERTICES: u64 = 240;
const MIN_FLORA_CLUSTER_HORIZONTAL_SPAN_M: f64 = 1.25;
const MIN_FLORA_CLUSTER_VERTICAL_SPAN_M: f64 = 0.45;
const MIN_RUIN_COMPLEX_MESH_VERTICES: u64 = 300;
const MIN_RUIN_COMPLEX_HORIZONTAL_SPAN_M: f64 = 2.5;
const MIN_RUIN_COMPLEX_VERTICAL_SPAN_M: f64 = 1.8;
const MIN_ROCK_FORMATION_MESH_VERTICES: u64 = 48;
const MIN_ROCK_FORMATION_HORIZONTAL_SPAN_M: f64 = 1.5;
const MIN_ROCK_FORMATION_VERTICAL_SPAN_M: f64 = 1.1;
const MIN_WATER_DETAIL_MESH_VERTICES: u64 = 40;
const MIN_WATER_DETAIL_HORIZONTAL_SPAN_M: f64 = 1.25;
const MIN_WATER_DETAIL_VERTICAL_SPAN_M: f64 = 0.1;
const MIN_ARTIFACT_DETAIL_VERTEX_TOTAL: u64 = 16_000;
const MIN_ARTIFACT_DETAIL_MESH_VERTICES: u64 = 60;
const MIN_ARTIFACT_STONE_MESH_VERTICES: u64 = 140;
const MIN_ARTIFACT_STONE_NORMAL_SLOPE_BANDS: u64 = 3;
const MIN_ARTIFACT_STAIR_HORIZONTAL_SPAN_M: f64 = 5.0;
const MIN_ARTIFACT_BRIDGE_HORIZONTAL_SPAN_M: f64 = 5.0;
const MIN_ARTIFACT_BANNER_VERTICAL_SPAN_M: f64 = 1.5;
const MIN_ARTIFACT_REED_VERTICAL_SPAN_M: f64 = 0.8;
const MIN_RUIN_ARCH_MESH_VERTICES: u64 = 500;
const MIN_RUIN_ARCH_VERTICAL_SPAN_M: f64 = 4.5;
const MIN_RUIN_ARCH_RADIUS_BANDS: u64 = 8;
const MIN_RUIN_ARCH_NORMAL_SLOPE_BANDS: u64 = 5;
const MIN_OBSTRUCTION_SPIRE_MESH_VERTICES: u64 = 300;
const MIN_OBSTRUCTION_SPIRE_TRIANGLE_COUNT: u64 = 500;
const MIN_OBSTRUCTION_SPIRE_VERTICAL_SPAN_M: f64 = 3.0;
const MIN_OBSTRUCTION_SPIRE_HEIGHT_BANDS: u64 = 6;
const MIN_OBSTRUCTION_SPIRE_RADIUS_BANDS: u64 = 5;
const MIN_OBSTRUCTION_SPIRE_NORMAL_SLOPE_BANDS: u64 = 5;
const MIN_TERRAIN_BIOME_PALETTE_COUNT: u64 = 5;
const MIN_FOLIAGE_PALETTE_COUNT: u64 = 5;
const MIN_STONE_PALETTE_COUNT: u64 = 4;

#[derive(Default)]
struct ArtifactCounters {
    expected: u64,
    found: u64,
    missing: u64,
    vertex_mismatches: u64,
    face_mismatches: u64,
}

#[derive(Debug)]
struct ObjAudit {
    vertex_count: u64,
    face_count: u64,
    horizontal_span_m: f64,
    vertical_span_m: f64,
    depth_span_m: f64,
}

#[derive(Default)]
struct SurfaceFeatureStats {
    count: u64,
    kinds: HashSet<String>,
    vertex_total: u64,
    min_mesh_vertices: Option<u64>,
    min_horizontal_span_m: Option<f64>,
    min_vertical_span_m: Option<f64>,
}

impl SurfaceFeatureStats {
    fn kind_count(&self) -> u64 {
        self.kinds.len() as u64
    }

    fn min_mesh_vertices(&self) -> u64 {
        self.min_mesh_vertices.unwrap_or(0)
    }

    fn min_horizontal_span_m(&self) -> f64 {
        self.min_horizontal_span_m.unwrap_or(0.0)
    }

    fn min_vertical_span_m(&self) -> f64 {
        self.min_vertical_span_m.unwrap_or(0.0)
    }
}

fn main() {
    let args = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("Usage: cargo run --bin visual_content_audit -- <manifest.json>");
        process::exit(2);
    }

    match audit_manifest_path(&args[0]) {
        Ok(report) => {
            let passed = report
                .get("passed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("audit report should serialize")
            );
            if !passed {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("visual content audit failed: {error}");
            process::exit(2);
        }
    }
}

fn audit_manifest_path(path: &Path) -> Result<Value, String> {
    let manifest_text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest = serde_json::from_str::<Value>(&manifest_text).map_err(|error| {
        format!(
            "could not parse visual content manifest {}: {error}",
            path.display()
        )
    })?;
    let root_dir = path.parent().unwrap_or_else(|| Path::new("."));

    Ok(audit_manifest(&manifest, root_dir, &path.to_string_lossy()))
}

fn audit_manifest(manifest: &Value, root_dir: &Path, manifest_path: &str) -> Value {
    let mut checks = Vec::new();
    let mut artifacts = Vec::new();
    let mut artifact_counters = ArtifactCounters::default();

    let schema = manifest.get("schema").and_then(Value::as_str).unwrap_or("");
    checks.push(check_eq_str(
        "schema",
        schema,
        EXPECTED_VISUAL_CONTENT_EXPORT_SCHEMA,
        "schema",
    ));

    let mesh_count = value_u64(manifest, "mesh_count");
    let total_vertex_count = value_u64(manifest, "total_vertex_count");
    let total_triangle_count = value_u64(manifest, "total_triangle_count");
    let counts = manifest.get("counts").unwrap_or(&Value::Null);
    let minimums = manifest.get("minimums").unwrap_or(&Value::Null);
    let landmarks = manifest.get("landmarks").and_then(Value::as_array);
    let landmark_count = landmarks.map_or(0, |entries| entries.len() as u64);
    let landmark_kind_count = landmarks.map_or(0, |entries| {
        entries
            .iter()
            .filter_map(|entry| entry.get("kind").and_then(Value::as_str))
            .collect::<HashSet<_>>()
            .len() as u64
    });
    let palette_stats = visual_palette_stats(manifest.get("palettes").and_then(Value::as_array));
    let flora_stats = surface_feature_stats(landmarks, "flora_cluster", root_dir);
    let ruin_stats = surface_feature_stats(landmarks, "ruin_complex", root_dir);
    let formation_stats = surface_feature_stats(landmarks, "rock_formation", root_dir);
    let water_detail_stats = surface_feature_stats(landmarks, "water_detail", root_dir);
    let surface_feature_vertex_total = flora_stats.vertex_total
        + ruin_stats.vertex_total
        + formation_stats.vertex_total
        + water_detail_stats.vertex_total;

    checks.push(check_at_least_u64(
        "ground_cover_count",
        value_u64(counts, "ground_cover_count"),
        MIN_GROUND_COVER_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_patch_total",
        value_u64(counts, "ground_cover_patch_total"),
        MIN_GROUND_COVER_PATCH_TOTAL,
        "patches",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_blade_total",
        value_u64(counts, "ground_cover_blade_total"),
        MIN_GROUND_COVER_BLADE_TOTAL,
        "blades",
    ));
    checks.push(check_at_least_u64(
        "tree_trunk_count",
        value_u64(counts, "tree_trunk_count"),
        MIN_TREE_TRUNK_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_count",
        value_u64(counts, "tree_canopy_count"),
        MIN_TREE_CANOPY_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "rock_count",
        value_u64(counts, "rock_count"),
        MIN_ROCK_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_count",
        value_u64(counts, "weather_cloud_count"),
        MIN_WEATHER_CLOUD_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_bank_count",
        value_u64(counts, "weather_cloud_bank_count"),
        MIN_WEATHER_CLOUD_BANK_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_veil_count",
        value_u64(counts, "weather_cloud_veil_count"),
        MIN_WEATHER_CLOUD_VEIL_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "landmark_count",
        value_u64(counts, "landmark_count"),
        MIN_LANDMARK_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "landmark_kind_count",
        value_u64(counts, "landmark_kind_count"),
        MIN_LANDMARK_KIND_COUNT,
        "kinds",
    ));
    checks.push(check_eq_u64(
        "landmark_count_manifest_parity",
        value_u64(counts, "landmark_count"),
        landmark_count,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "landmark_kind_count_manifest_parity",
        value_u64(counts, "landmark_kind_count"),
        landmark_kind_count,
        "kinds",
    ));
    checks.push(check_eq_u64(
        "palette_count",
        palette_stats.entry_count,
        EXPECTED_PALETTE_COUNT,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "palette_count_manifest_parity",
        value_u64(counts, "palette_count"),
        palette_stats.entry_count,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "distinct_palette_count",
        palette_stats.distinct_palette_count,
        EXPECTED_PALETTE_COUNT,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "distinct_palette_count_manifest_parity",
        value_u64(counts, "distinct_palette_count"),
        palette_stats.distinct_palette_count,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "lossless_art_direction_signatures",
        palette_stats.lossless_signature_count,
        palette_stats.entry_count,
        "signatures",
    ));
    checks.push(check_at_least_u64(
        "flora_cluster_count",
        flora_stats.count,
        MIN_FLORA_CLUSTER_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "flora_cluster_kind_count",
        flora_stats.kind_count(),
        MIN_FLORA_CLUSTER_KIND_COUNT,
        "kinds",
    ));
    checks.push(check_at_least_u64(
        "ruin_complex_count",
        ruin_stats.count,
        MIN_RUIN_COMPLEX_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "ruin_complex_kind_count",
        ruin_stats.kind_count(),
        MIN_RUIN_COMPLEX_KIND_COUNT,
        "kinds",
    ));
    checks.push(check_at_least_u64(
        "rock_formation_count",
        formation_stats.count,
        MIN_ROCK_FORMATION_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "rock_formation_kind_count",
        formation_stats.kind_count(),
        MIN_ROCK_FORMATION_KIND_COUNT,
        "kinds",
    ));
    checks.push(check_at_least_u64(
        "water_detail_count",
        water_detail_stats.count,
        MIN_WATER_DETAIL_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "water_detail_kind_count",
        water_detail_stats.kind_count(),
        MIN_WATER_DETAIL_KIND_COUNT,
        "kinds",
    ));
    checks.push(check_eq_u64(
        "flora_cluster_count_manifest_parity",
        value_u64(counts, "flora_cluster_count"),
        flora_stats.count,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "flora_cluster_kind_count_manifest_parity",
        value_u64(counts, "flora_cluster_kind_count"),
        flora_stats.kind_count(),
        "kinds",
    ));
    checks.push(check_eq_u64(
        "ruin_complex_count_manifest_parity",
        value_u64(counts, "ruin_complex_count"),
        ruin_stats.count,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "ruin_complex_kind_count_manifest_parity",
        value_u64(counts, "ruin_complex_kind_count"),
        ruin_stats.kind_count(),
        "kinds",
    ));
    checks.push(check_eq_u64(
        "rock_formation_count_manifest_parity",
        value_u64(counts, "rock_formation_count"),
        formation_stats.count,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "rock_formation_kind_count_manifest_parity",
        value_u64(counts, "rock_formation_kind_count"),
        formation_stats.kind_count(),
        "kinds",
    ));
    checks.push(check_eq_u64(
        "water_detail_count_manifest_parity",
        value_u64(counts, "water_detail_count"),
        water_detail_stats.count,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "water_detail_kind_count_manifest_parity",
        value_u64(counts, "water_detail_kind_count"),
        water_detail_stats.kind_count(),
        "kinds",
    ));
    checks.push(check_at_least_u64(
        "artifact_detail_count",
        value_u64(counts, "artifact_detail_count"),
        MIN_ARTIFACT_DETAIL_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "artifact_detail_kind_count",
        value_u64(counts, "artifact_detail_kind_count"),
        MIN_ARTIFACT_DETAIL_KIND_COUNT,
        "kinds",
    ));
    checks.push(check_at_least_u64(
        "artifact_stair_count",
        value_u64(counts, "artifact_stair_count"),
        MIN_ARTIFACT_STAIR_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "artifact_bridge_fragment_count",
        value_u64(counts, "artifact_bridge_fragment_count"),
        MIN_ARTIFACT_BRIDGE_FRAGMENT_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "artifact_glyph_slab_count",
        value_u64(counts, "artifact_glyph_slab_count"),
        MIN_ARTIFACT_GLYPH_SLAB_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "artifact_retaining_wall_count",
        value_u64(counts, "artifact_retaining_wall_count"),
        MIN_ARTIFACT_RETAINING_WALL_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "artifact_banner_count",
        value_u64(counts, "artifact_banner_count"),
        MIN_ARTIFACT_BANNER_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "artifact_pebble_field_count",
        value_u64(counts, "artifact_pebble_field_count"),
        MIN_ARTIFACT_PEBBLE_FIELD_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "artifact_reed_patch_count",
        value_u64(counts, "artifact_reed_patch_count"),
        MIN_ARTIFACT_REED_PATCH_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "small_island_count",
        value_u64(counts, "small_island_count"),
        MIN_SMALL_ISLAND_COUNT,
        "islands",
    ));
    checks.push(check_at_least_u64(
        "plateau_landmark_count",
        value_u64(counts, "plateau_landmark_count"),
        MIN_PLATEAU_LANDMARK_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "plateau_waterfall_ribbon_count",
        value_u64(counts, "plateau_waterfall_ribbon_count"),
        MIN_PLATEAU_WATERFALL_RIBBON_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "plateau_waterfall_mist_count",
        value_u64(counts, "plateau_waterfall_mist_count"),
        MIN_PLATEAU_WATERFALL_MIST_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "route_waterfall_ribbon_count",
        value_u64(counts, "route_waterfall_ribbon_count"),
        MIN_ROUTE_WATERFALL_RIBBON_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "route_waterfall_mist_count",
        value_u64(counts, "route_waterfall_mist_count"),
        MIN_ROUTE_WATERFALL_MIST_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "route_lake_surface_count",
        value_u64(counts, "route_lake_surface_count"),
        MIN_ROUTE_LAKE_SURFACE_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "river_channel_count",
        value_u64(counts, "river_channel_count"),
        MIN_RIVER_CHANNEL_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "under_route_visual_count",
        value_u64(counts, "under_route_visual_count"),
        MIN_UNDER_ROUTE_VISUAL_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "under_route_cave_mouth_count",
        value_u64(counts, "under_route_cave_mouth_count"),
        MIN_UNDER_ROUTE_CAVE_MOUTH_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "ruin_cluster_count",
        value_u64(counts, "ruin_cluster_count"),
        MIN_RUIN_CLUSTER_COUNT,
        "clusters",
    ));
    checks.push(check_at_least_u64(
        "ruin_arch_count",
        value_u64(counts, "ruin_arch_count"),
        MIN_RUIN_ARCH_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "route_cairn_count",
        value_u64(counts, "route_cairn_count"),
        MIN_ROUTE_CAIRN_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "launch_beacon_count",
        value_u64(counts, "launch_beacon_count"),
        MIN_LAUNCH_BEACON_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "landing_garden_marker_count",
        value_u64(counts, "landing_garden_marker_count"),
        MIN_LANDING_GARDEN_MARKER_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "pond_surface_count",
        value_u64(counts, "pond_surface_count"),
        MIN_POND_SURFACE_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "obstruction_spire_count",
        value_u64(counts, "obstruction_spire_count"),
        MIN_OBSTRUCTION_SPIRE_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_patch_count",
        value_u64(minimums, "ground_cover_patch_count"),
        MIN_GROUND_COVER_PATCH_COUNT,
        "patches",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_mesh_vertices",
        value_u64(minimums, "ground_cover_mesh_vertices"),
        MIN_GROUND_COVER_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_blade_count",
        value_u64(minimums, "ground_cover_blade_count"),
        MIN_GROUND_COVER_BLADE_COUNT,
        "blades",
    ));
    checks.push(check_at_least_f64(
        "ground_cover_blade_height_range",
        value_f64(minimums, "ground_cover_blade_height_range_m"),
        MIN_GROUND_COVER_BLADE_HEIGHT_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "tree_trunk_mesh_vertices",
        value_u64(minimums, "tree_trunk_mesh_vertices"),
        MIN_TREE_TRUNK_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "tree_trunk_taper_ratio",
        value_f64(minimums, "tree_trunk_taper_ratio"),
        MIN_TREE_TRUNK_TAPER_RATIO,
        "ratio",
    ));
    checks.push(check_at_least_f64(
        "tree_branch_reach_ratio",
        value_f64(minimums, "tree_branch_reach_ratio"),
        MIN_TREE_BRANCH_REACH_RATIO,
        "ratio",
    ));
    checks.push(check_at_least_u64(
        "tree_branch_count",
        value_u64(minimums, "tree_branch_count"),
        MIN_TREE_BRANCH_COUNT,
        "branches",
    ));
    checks.push(check_at_least_u64(
        "tree_root_flare_count",
        value_u64(minimums, "tree_root_flare_count"),
        MIN_TREE_ROOT_FLARE_COUNT,
        "roots",
    ));
    checks.push(check_at_least_u64(
        "tree_trunk_ring_count",
        value_u64(minimums, "tree_trunk_ring_count"),
        MIN_TREE_TRUNK_RING_COUNT,
        "rings",
    ));
    checks.push(check_at_least_f64(
        "tree_trunk_height_range",
        value_f64(minimums, "tree_trunk_height_range_m"),
        MIN_TREE_TRUNK_HEIGHT_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_mesh_vertices",
        value_u64(minimums, "tree_canopy_mesh_vertices"),
        MIN_TREE_CANOPY_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_lobe_count",
        value_u64(minimums, "tree_canopy_lobe_count"),
        MIN_TREE_CANOPY_LOBE_COUNT,
        "lobes",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_detail_card_count",
        value_u64(minimums, "tree_canopy_detail_card_count"),
        MIN_TREE_CANOPY_DETAIL_CARD_COUNT,
        "cards",
    ));
    checks.push(check_at_least_f64(
        "tree_canopy_vertical_to_horizontal_ratio",
        value_f64(minimums, "tree_canopy_vertical_to_horizontal_ratio"),
        MIN_TREE_CANOPY_VERTICAL_TO_HORIZONTAL_RATIO,
        "ratio",
    ));
    checks.push(check_at_least_f64(
        "tree_canopy_radius_range",
        value_f64(minimums, "tree_canopy_radius_range_m"),
        MIN_TREE_CANOPY_RADIUS_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "rock_mesh_vertices",
        value_u64(minimums, "rock_mesh_vertices"),
        MIN_ROCK_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "rock_vertical_span",
        value_f64(minimums, "rock_vertical_span_m"),
        MIN_ROCK_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_mesh_vertices",
        value_u64(minimums, "weather_cloud_mesh_vertices"),
        MIN_WEATHER_CLOUD_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_lobe_count",
        value_u64(minimums, "weather_cloud_lobe_count"),
        MIN_WEATHER_CLOUD_LOBE_COUNT,
        "lobes",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_wisp_card_count",
        value_u64(minimums, "weather_cloud_wisp_card_count"),
        MIN_WEATHER_CLOUD_WISP_CARD_COUNT,
        "cards",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_filament_ribbon_detail_count",
        value_u64(minimums, "weather_cloud_filament_ribbon_detail_count"),
        MIN_WEATHER_CLOUD_FILAMENT_RIBBON_DETAIL_COUNT,
        "ribbons",
    ));
    checks.push(check_at_least_f64(
        "weather_cloud_bank_depth",
        value_f64(minimums, "weather_cloud_bank_depth_m"),
        MIN_WEATHER_CLOUD_BANK_DEPTH_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_bank_lobe_count",
        value_u64(minimums, "weather_cloud_bank_lobe_count"),
        MIN_WEATHER_CLOUD_BANK_LOBE_COUNT,
        "lobes",
    ));
    checks.push(check_at_least_f64(
        "weather_cloud_scaled_depth_span",
        value_f64(minimums, "weather_cloud_scaled_depth_span_m"),
        MIN_WEATHER_CLOUD_SCALED_DEPTH_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "route_cairn_mesh_vertices",
        value_u64(minimums, "route_cairn_mesh_vertices"),
        MIN_ROUTE_CAIRN_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "route_cairn_vertical_span",
        value_f64(minimums, "route_cairn_vertical_span_m"),
        MIN_ROUTE_CAIRN_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "launch_beacon_mesh_vertices",
        value_u64(minimums, "launch_beacon_mesh_vertices"),
        MIN_LAUNCH_BEACON_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "launch_beacon_vertical_span",
        value_f64(minimums, "launch_beacon_vertical_span_m"),
        MIN_LAUNCH_BEACON_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "landing_garden_marker_mesh_vertices",
        value_u64(minimums, "landing_garden_marker_mesh_vertices"),
        MIN_LANDING_GARDEN_MARKER_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "landing_garden_marker_vertical_span",
        value_f64(minimums, "landing_garden_marker_vertical_span_m"),
        MIN_LANDING_GARDEN_MARKER_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "pond_surface_mesh_vertices",
        value_u64(minimums, "pond_surface_mesh_vertices"),
        MIN_POND_SURFACE_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "pond_surface_vertical_span",
        value_f64(minimums, "pond_surface_vertical_span_m"),
        MIN_POND_SURFACE_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "plateau_landmark_vertex_total",
        value_u64(minimums, "plateau_landmark_vertex_total"),
        MIN_PLATEAU_LANDMARK_VERTEX_TOTAL,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "max_plateau_landmark_mesh_vertices",
        value_u64(minimums, "max_plateau_landmark_mesh_vertices"),
        MIN_MAX_PLATEAU_LANDMARK_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "plateau_waterfall_vertical_span",
        value_f64(minimums, "plateau_waterfall_vertical_span_m"),
        MIN_PLATEAU_WATERFALL_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "route_waterfall_vertical_span",
        value_f64(minimums, "route_waterfall_vertical_span_m"),
        MIN_ROUTE_WATERFALL_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "route_lake_surface_horizontal_span",
        value_f64(minimums, "route_lake_surface_horizontal_span_m"),
        MIN_ROUTE_LAKE_SURFACE_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "river_channel_horizontal_span",
        value_f64(minimums, "river_channel_horizontal_span_m"),
        MIN_RIVER_CHANNEL_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "under_route_visual_vertical_span",
        value_f64(minimums, "under_route_visual_vertical_span_m"),
        MIN_UNDER_ROUTE_VISUAL_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "surface_feature_vertex_total",
        surface_feature_vertex_total,
        MIN_SURFACE_FEATURE_VERTEX_TOTAL,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "flora_cluster_mesh_vertices",
        flora_stats.min_mesh_vertices(),
        MIN_FLORA_CLUSTER_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "flora_cluster_horizontal_span",
        flora_stats.min_horizontal_span_m(),
        MIN_FLORA_CLUSTER_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "flora_cluster_vertical_span",
        flora_stats.min_vertical_span_m(),
        MIN_FLORA_CLUSTER_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "ruin_complex_mesh_vertices",
        ruin_stats.min_mesh_vertices(),
        MIN_RUIN_COMPLEX_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "ruin_complex_horizontal_span",
        ruin_stats.min_horizontal_span_m(),
        MIN_RUIN_COMPLEX_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "ruin_complex_vertical_span",
        ruin_stats.min_vertical_span_m(),
        MIN_RUIN_COMPLEX_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "rock_formation_mesh_vertices",
        formation_stats.min_mesh_vertices(),
        MIN_ROCK_FORMATION_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "rock_formation_horizontal_span",
        formation_stats.min_horizontal_span_m(),
        MIN_ROCK_FORMATION_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "rock_formation_vertical_span",
        formation_stats.min_vertical_span_m(),
        MIN_ROCK_FORMATION_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "water_detail_mesh_vertices",
        water_detail_stats.min_mesh_vertices(),
        MIN_WATER_DETAIL_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "water_detail_horizontal_span",
        water_detail_stats.min_horizontal_span_m(),
        MIN_WATER_DETAIL_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "water_detail_vertical_span",
        water_detail_stats.min_vertical_span_m(),
        MIN_WATER_DETAIL_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_eq_u64(
        "surface_feature_vertex_total_manifest_parity",
        value_u64(minimums, "surface_feature_vertex_total"),
        surface_feature_vertex_total,
        "vertices",
    ));
    append_surface_feature_minimum_parity_checks(
        &mut checks,
        minimums,
        "flora_cluster",
        &flora_stats,
    );
    append_surface_feature_minimum_parity_checks(
        &mut checks,
        minimums,
        "ruin_complex",
        &ruin_stats,
    );
    append_surface_feature_minimum_parity_checks(
        &mut checks,
        minimums,
        "rock_formation",
        &formation_stats,
    );
    append_surface_feature_minimum_parity_checks(
        &mut checks,
        minimums,
        "water_detail",
        &water_detail_stats,
    );
    checks.push(check_at_least_u64(
        "artifact_detail_vertex_total",
        value_u64(minimums, "artifact_detail_vertex_total"),
        MIN_ARTIFACT_DETAIL_VERTEX_TOTAL,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "artifact_detail_mesh_vertices",
        value_u64(minimums, "artifact_detail_mesh_vertices"),
        MIN_ARTIFACT_DETAIL_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "artifact_stone_mesh_vertices",
        value_u64(minimums, "artifact_stone_mesh_vertices"),
        MIN_ARTIFACT_STONE_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "artifact_stone_normal_slope_bands",
        value_u64(minimums, "artifact_stone_normal_slope_band_count"),
        MIN_ARTIFACT_STONE_NORMAL_SLOPE_BANDS,
        "bands",
    ));
    checks.push(check_at_least_f64(
        "artifact_stair_horizontal_span",
        value_f64(minimums, "artifact_stair_horizontal_span_m"),
        MIN_ARTIFACT_STAIR_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "artifact_bridge_horizontal_span",
        value_f64(minimums, "artifact_bridge_horizontal_span_m"),
        MIN_ARTIFACT_BRIDGE_HORIZONTAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "artifact_banner_vertical_span",
        value_f64(minimums, "artifact_banner_vertical_span_m"),
        MIN_ARTIFACT_BANNER_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "artifact_reed_vertical_span",
        value_f64(minimums, "artifact_reed_vertical_span_m"),
        MIN_ARTIFACT_REED_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "ruin_arch_mesh_vertices",
        value_u64(minimums, "ruin_arch_mesh_vertices"),
        MIN_RUIN_ARCH_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "ruin_arch_vertical_span",
        value_f64(minimums, "ruin_arch_vertical_span_m"),
        MIN_RUIN_ARCH_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "ruin_arch_radius_bands",
        value_u64(minimums, "ruin_arch_radius_band_count"),
        MIN_RUIN_ARCH_RADIUS_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "ruin_arch_normal_slope_bands",
        value_u64(minimums, "ruin_arch_normal_slope_band_count"),
        MIN_RUIN_ARCH_NORMAL_SLOPE_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "obstruction_spire_mesh_vertices",
        value_u64(minimums, "obstruction_spire_mesh_vertices"),
        MIN_OBSTRUCTION_SPIRE_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "obstruction_spire_triangle_count",
        value_u64(minimums, "obstruction_spire_triangle_count"),
        MIN_OBSTRUCTION_SPIRE_TRIANGLE_COUNT,
        "triangles",
    ));
    checks.push(check_at_least_f64(
        "obstruction_spire_vertical_span",
        value_f64(minimums, "obstruction_spire_vertical_span_m"),
        MIN_OBSTRUCTION_SPIRE_VERTICAL_SPAN_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "obstruction_spire_height_bands",
        value_u64(minimums, "obstruction_spire_height_band_count"),
        MIN_OBSTRUCTION_SPIRE_HEIGHT_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "obstruction_spire_radius_bands",
        value_u64(minimums, "obstruction_spire_radius_band_count"),
        MIN_OBSTRUCTION_SPIRE_RADIUS_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "obstruction_spire_normal_slope_bands",
        value_u64(minimums, "obstruction_spire_normal_slope_band_count"),
        MIN_OBSTRUCTION_SPIRE_NORMAL_SLOPE_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "coarse_terrain_biome_palette_count",
        palette_stats.terrain_color_count,
        MIN_TERRAIN_BIOME_PALETTE_COUNT,
        "palettes",
    ));
    checks.push(check_at_least_u64(
        "coarse_foliage_palette_count",
        palette_stats.foliage_color_count,
        MIN_FOLIAGE_PALETTE_COUNT,
        "palettes",
    ));
    checks.push(check_at_least_u64(
        "coarse_stone_palette_count",
        palette_stats.stone_color_count,
        MIN_STONE_PALETTE_COUNT,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "terrain_biome_palette_count_manifest_parity",
        value_u64(minimums, "terrain_biome_palette_count"),
        palette_stats.exact_terrain_color_count,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "foliage_palette_count_manifest_parity",
        value_u64(minimums, "foliage_palette_count"),
        palette_stats.exact_foliage_color_count,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "stone_palette_count_manifest_parity",
        value_u64(minimums, "stone_palette_count"),
        palette_stats.exact_stone_color_count,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "coarse_terrain_biome_palette_count_manifest_parity",
        value_u64(minimums, "coarse_terrain_biome_palette_count"),
        palette_stats.terrain_color_count,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "coarse_foliage_palette_count_manifest_parity",
        value_u64(minimums, "coarse_foliage_palette_count"),
        palette_stats.foliage_color_count,
        "palettes",
    ));
    checks.push(check_eq_u64(
        "coarse_stone_palette_count_manifest_parity",
        value_u64(minimums, "coarse_stone_palette_count"),
        palette_stats.stone_color_count,
        "palettes",
    ));

    audit_mesh_array(
        manifest.get("ground_cover").and_then(Value::as_array),
        "mesh",
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );
    audit_tree_array(
        manifest.get("trees").and_then(Value::as_array),
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );
    audit_mesh_array(
        manifest.get("rocks").and_then(Value::as_array),
        "mesh",
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );
    audit_mesh_array(
        manifest.get("clouds").and_then(Value::as_array),
        "mesh",
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );
    audit_mesh_array(
        manifest.get("landmarks").and_then(Value::as_array),
        "mesh",
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );

    checks.push(check_eq_u64(
        "mesh_count",
        mesh_count,
        artifact_counters.expected,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "total_vertex_count",
        total_vertex_count,
        artifact_counters.expected.saturating_mul(300),
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "total_triangle_count",
        total_triangle_count,
        artifact_counters.expected.saturating_mul(200),
        "triangles",
    ));
    checks.push(check_eq_u64(
        "mesh_artifact_count",
        artifact_counters.found,
        artifact_counters.expected,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_vertex_mismatch_count",
        artifact_counters.vertex_mismatches,
        0,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_face_mismatch_count",
        artifact_counters.face_mismatches,
        0,
        "meshes",
    ));

    let passed = checks.iter().all(|check| {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });

    json!({
        "schema": "nau_visual_content_audit.v1",
        "manifest": manifest_path,
        "passed": passed,
        "checks": checks,
        "artifacts": {
            "expected_mesh_count": artifact_counters.expected,
            "found_mesh_count": artifact_counters.found,
            "missing_mesh_count": artifact_counters.missing,
            "vertex_mismatch_count": artifact_counters.vertex_mismatches,
            "face_mismatch_count": artifact_counters.face_mismatches,
            "failures": artifacts,
        }
    })
}

fn audit_tree_array(
    entries: Option<&Vec<Value>>,
    root_dir: &Path,
    counters: &mut ArtifactCounters,
    artifacts: &mut Vec<Value>,
) {
    let Some(entries) = entries else {
        return;
    };

    for tree in entries {
        audit_mesh_value(
            tree.get("trunk").unwrap_or(&Value::Null),
            root_dir,
            counters,
            artifacts,
        );
        audit_mesh_value(
            tree.get("canopy").unwrap_or(&Value::Null),
            root_dir,
            counters,
            artifacts,
        );
    }
}

fn audit_mesh_array(
    entries: Option<&Vec<Value>>,
    mesh_key: &str,
    root_dir: &Path,
    counters: &mut ArtifactCounters,
    artifacts: &mut Vec<Value>,
) {
    let Some(entries) = entries else {
        return;
    };

    for entry in entries {
        audit_mesh_value(
            entry.get(mesh_key).unwrap_or(&Value::Null),
            root_dir,
            counters,
            artifacts,
        );
    }
}

fn audit_mesh_value(
    mesh: &Value,
    root_dir: &Path,
    counters: &mut ArtifactCounters,
    artifacts: &mut Vec<Value>,
) {
    counters.expected += 1;
    let Some(obj_path) = relative_path(mesh, "obj") else {
        counters.missing += 1;
        artifacts.push(json!({"error": "missing obj path"}));
        return;
    };
    let full_path = root_dir.join(&obj_path);
    let manifest_vertices = value_u64(mesh, "vertex_count");
    let manifest_faces = value_u64(mesh, "triangle_count");

    match audit_obj_path(&full_path) {
        Ok(obj) => {
            counters.found += 1;
            if obj.vertex_count != manifest_vertices {
                counters.vertex_mismatches += 1;
                artifacts.push(json!({
                    "path": obj_path.to_string_lossy(),
                    "error": "vertex mismatch",
                    "manifest": manifest_vertices,
                    "obj": obj.vertex_count,
                }));
            }
            if obj.face_count != manifest_faces {
                counters.face_mismatches += 1;
                artifacts.push(json!({
                    "path": obj_path.to_string_lossy(),
                    "error": "face mismatch",
                    "manifest": manifest_faces,
                    "obj": obj.face_count,
                }));
            }
        }
        Err(error) => {
            counters.missing += 1;
            artifacts.push(json!({
                "path": obj_path.to_string_lossy(),
                "error": error,
            }));
        }
    }
}

fn surface_feature_stats(
    entries: Option<&Vec<Value>>,
    family: &str,
    root_dir: &Path,
) -> SurfaceFeatureStats {
    let mut stats = SurfaceFeatureStats::default();
    let Some(entries) = entries else {
        return stats;
    };

    for entry in entries {
        if entry.get("surface_feature_family").and_then(Value::as_str) != Some(family) {
            continue;
        }

        stats.count += 1;
        if let Some(kind) = entry.get("kind").and_then(Value::as_str) {
            stats.kinds.insert(kind.to_string());
        }

        let mesh = entry.get("mesh").unwrap_or(&Value::Null);
        let Some(obj_path) = relative_path(mesh, "obj") else {
            continue;
        };
        let Ok(obj) = audit_obj_path(&root_dir.join(obj_path)) else {
            continue;
        };

        let horizontal_span_m = obj.horizontal_span_m.max(obj.depth_span_m);
        stats.vertex_total += obj.vertex_count;
        stats.min_mesh_vertices = Some(
            stats
                .min_mesh_vertices
                .map_or(obj.vertex_count, |minimum| minimum.min(obj.vertex_count)),
        );
        stats.min_horizontal_span_m = Some(
            stats
                .min_horizontal_span_m
                .map_or(horizontal_span_m, |minimum| minimum.min(horizontal_span_m)),
        );
        stats.min_vertical_span_m = Some(
            stats
                .min_vertical_span_m
                .map_or(obj.vertical_span_m, |minimum| {
                    minimum.min(obj.vertical_span_m)
                }),
        );
    }

    stats
}

fn append_surface_feature_minimum_parity_checks(
    checks: &mut Vec<Value>,
    minimums: &Value,
    family: &str,
    stats: &SurfaceFeatureStats,
) {
    checks.push(check_eq_u64(
        &format!("{family}_mesh_vertices_manifest_parity"),
        value_u64(minimums, &format!("{family}_mesh_vertices")),
        stats.min_mesh_vertices(),
        "vertices",
    ));
    checks.push(check_approx_eq_f64(
        &format!("{family}_horizontal_span_manifest_parity"),
        value_f64(minimums, &format!("{family}_horizontal_span_m")),
        stats.min_horizontal_span_m(),
        0.001,
        "m",
    ));
    checks.push(check_approx_eq_f64(
        &format!("{family}_vertical_span_manifest_parity"),
        value_f64(minimums, &format!("{family}_vertical_span_m")),
        stats.min_vertical_span_m(),
        0.001,
        "m",
    ));
}

#[derive(Default)]
struct VisualPaletteStats {
    entry_count: u64,
    distinct_palette_count: u64,
    lossless_signature_count: u64,
    exact_terrain_color_count: u64,
    exact_foliage_color_count: u64,
    exact_stone_color_count: u64,
    terrain_color_count: u64,
    foliage_color_count: u64,
    stone_color_count: u64,
}

fn visual_palette_stats(entries: Option<&Vec<Value>>) -> VisualPaletteStats {
    let Some(entries) = entries else {
        return VisualPaletteStats::default();
    };
    let palette_keys = entries
        .iter()
        .filter_map(visual_palette_key)
        .collect::<Vec<_>>();

    VisualPaletteStats {
        entry_count: entries.len() as u64,
        distinct_palette_count: distinct_palette_count(&palette_keys),
        lossless_signature_count: entries
            .iter()
            .filter(|entry| {
                entry
                    .get("art_direction_signature")
                    .and_then(Value::as_str)
                    .is_some_and(|signature| signature.parse::<u64>().is_ok())
            })
            .count() as u64,
        exact_terrain_color_count: exact_color_count(
            entries
                .iter()
                .filter_map(|entry| visual_color_key(entry, "terrain_key")),
        ),
        exact_foliage_color_count: exact_color_count(
            entries
                .iter()
                .filter_map(|entry| visual_color_key(entry, "foliage_key")),
        ),
        exact_stone_color_count: exact_color_count(
            entries
                .iter()
                .filter_map(|entry| visual_color_key(entry, "stone_key")),
        ),
        terrain_color_count: coarse_color_count(
            entries
                .iter()
                .filter_map(|entry| visual_color_key(entry, "terrain_key")),
        ),
        foliage_color_count: coarse_color_count(
            entries
                .iter()
                .filter_map(|entry| visual_color_key(entry, "foliage_key")),
        ),
        stone_color_count: coarse_color_count(
            entries
                .iter()
                .filter_map(|entry| visual_color_key(entry, "stone_key")),
        ),
    }
}

fn visual_palette_key(entry: &Value) -> Option<[[u8; 3]; 3]> {
    Some([
        visual_color_key(entry, "terrain_key")?,
        visual_color_key(entry, "foliage_key")?,
        visual_color_key(entry, "stone_key")?,
    ])
}

fn visual_color_key(entry: &Value, field: &str) -> Option<[u8; 3]> {
    let channels = entry.get(field)?.as_array()?;
    if channels.len() != 3 {
        return None;
    }
    Some([
        u8::try_from(channels[0].as_u64()?).ok()?,
        u8::try_from(channels[1].as_u64()?).ok()?,
        u8::try_from(channels[2].as_u64()?).ok()?,
    ])
}

fn exact_color_count(colors: impl Iterator<Item = [u8; 3]>) -> u64 {
    colors.collect::<HashSet<_>>().len() as u64
}

fn coarse_color_count(colors: impl Iterator<Item = [u8; 3]>) -> u64 {
    colors
        .map(|color| color.map(|channel| channel / 8))
        .collect::<HashSet<_>>()
        .len() as u64
}

fn distinct_palette_count(palettes: &[[[u8; 3]; 3]]) -> u64 {
    let mut distinct = Vec::<[[u8; 3]; 3]>::new();
    for palette in palettes {
        if distinct.iter().all(|other| {
            visual_palette_distance(*palette, *other) >= MIN_PERCEPTUAL_PALETTE_DISTANCE
        }) {
            distinct.push(*palette);
        }
    }
    distinct.len() as u64
}

fn visual_palette_distance(left: [[u8; 3]; 3], right: [[u8; 3]; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| {
            let left = visual_oklab(left);
            let right = visual_oklab(right);
            left.into_iter()
                .zip(right)
                .map(|(left, right)| (left - right).powi(2))
                .sum::<f64>()
                .sqrt()
        })
        .fold(0.0, f64::max)
}

fn visual_oklab(color: [u8; 3]) -> [f64; 3] {
    let [red, green, blue] = color.map(srgb_channel_to_linear);
    let light = (0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue).cbrt();
    let medium = (0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue).cbrt();
    let short = (0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue).cbrt();

    [
        0.210_454_26 * light + 0.793_617_8 * medium - 0.004_072_047 * short,
        1.977_998_5 * light - 2.428_592_2 * medium + 0.450_593_7 * short,
        0.025_904_037 * light + 0.782_771_77 * medium - 0.808_675_77 * short,
    ]
}

fn srgb_channel_to_linear(channel: u8) -> f64 {
    let channel = f64::from(channel) / 255.0;
    if channel <= 0.040_45 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn audit_obj_path(path: &Path) -> Result<ObjAudit, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut vertex_count = 0;
    let mut face_count = 0;
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];

    for line in text.lines() {
        if let Some(vertex) = line.strip_prefix("v ") {
            let coordinates = vertex
                .split_whitespace()
                .take(3)
                .map(str::parse::<f64>)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("invalid OBJ vertex in {}: {error}", path.display()))?;
            if coordinates.len() != 3 {
                return Err(format!("incomplete OBJ vertex in {}", path.display()));
            }
            for axis in 0..3 {
                min[axis] = min[axis].min(coordinates[axis]);
                max[axis] = max[axis].max(coordinates[axis]);
            }
            vertex_count += 1;
        } else if line.starts_with("f ") {
            face_count += 1;
        }
    }

    let span = if vertex_count > 0 {
        [max[0] - min[0], max[1] - min[1], max[2] - min[2]]
    } else {
        [0.0; 3]
    };
    Ok(ObjAudit {
        vertex_count,
        face_count,
        horizontal_span_m: span[0],
        vertical_span_m: span[1],
        depth_span_m: span[2],
    })
}

fn relative_path(parent: &Value, key: &str) -> Option<PathBuf> {
    parent.get(key).and_then(Value::as_str).map(PathBuf::from)
}

fn value_u64(parent: &Value, key: &str) -> u64 {
    parent.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn value_f64(parent: &Value, key: &str) -> f64 {
    parent.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn check_eq_str(name: &str, value: &str, expected: &str, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == expected,
        "value": value,
        "comparator": "==",
        "threshold": expected,
        "unit": unit,
    })
}

fn check_eq_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_approx_eq_f64(
    name: &str,
    value: f64,
    threshold: f64,
    tolerance: f64,
    unit: &str,
) -> Value {
    json!({
        "name": name,
        "passed": (value - threshold).abs() <= tolerance,
        "value": value,
        "comparator": "~=",
        "threshold": threshold,
        "tolerance": tolerance,
        "unit": unit,
    })
}

fn check_at_least_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_least_f64(name: &str, value: f64, threshold: f64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detail_layout_thresholds_match_export_contract() {
        assert_eq!(MIN_GROUND_COVER_PATCH_TOTAL, 2_400);
        assert_eq!(MIN_GROUND_COVER_BLADE_TOTAL, 12_000);
        assert_eq!(MIN_GROUND_COVER_PATCH_COUNT, 24);
        assert_eq!(MIN_GROUND_COVER_BLADE_COUNT, 120);
        assert_eq!(MIN_GROUND_COVER_MESH_VERTICES, 720);
        assert_eq!(MIN_GROUND_COVER_BLADE_HEIGHT_RANGE_M, 0.70);
        assert_eq!(MIN_TREE_TRUNK_COUNT, 160);
        assert_eq!(MIN_TREE_CANOPY_COUNT, 160);
        assert_eq!(MIN_ROCK_COUNT, 230);
        assert_eq!(MIN_RUIN_CLUSTER_COUNT, 6);
        assert_eq!(MIN_RIVER_CHANNEL_COUNT, 6);
        assert_eq!(MIN_RIVER_CHANNEL_HORIZONTAL_SPAN_M, 4.0);
        assert_eq!(MIN_POND_SURFACE_COUNT, 5);
        assert_eq!(MIN_FLORA_CLUSTER_COUNT, 36);
        assert_eq!(MIN_FLORA_CLUSTER_KIND_COUNT, 5);
        assert_eq!(MIN_RUIN_COMPLEX_COUNT, 8);
        assert_eq!(MIN_RUIN_COMPLEX_KIND_COUNT, 4);
        assert_eq!(MIN_ROCK_FORMATION_COUNT, 20);
        assert_eq!(MIN_ROCK_FORMATION_KIND_COUNT, 4);
        assert_eq!(MIN_WATER_DETAIL_COUNT, 10);
        assert_eq!(MIN_WATER_DETAIL_KIND_COUNT, 5);
        assert_eq!(MIN_SURFACE_FEATURE_VERTEX_TOTAL, 15_000);
    }

    #[test]
    fn audit_requires_v2_schema_and_string_encoded_u64_signatures() {
        let manifest = json!({
            "schema": "nau_visual_content_export.v1",
            "counts": {
                "palette_count": 1,
                "distinct_palette_count": 1
            },
            "minimums": {},
            "landmarks": [],
            "palettes": [{
                "art_direction_signature": u64::MAX,
                "terrain_key": [80, 140, 72],
                "foliage_key": [68, 124, 64],
                "stone_key": [122, 112, 98]
            }]
        });

        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
        let checks = report.get("checks").and_then(Value::as_array).unwrap();
        for name in ["schema", "lossless_art_direction_signatures"] {
            assert!(
                check_named(checks, name).is_some_and(|check| {
                    !check.get("passed").and_then(Value::as_bool).unwrap()
                }),
                "{name} should reject legacy manifest encoding"
            );
        }
    }

    #[test]
    fn audit_rejects_tampered_landmark_and_palette_counts() {
        let manifest = json!({
            "schema": EXPECTED_VISUAL_CONTENT_EXPORT_SCHEMA,
            "counts": {
                "landmark_count": 2,
                "landmark_kind_count": 2,
                "palette_count": 2,
                "distinct_palette_count": 2
            },
            "minimums": {
                "terrain_biome_palette_count": 2,
                "foliage_palette_count": 2,
                "stone_palette_count": 2
            },
            "landmarks": [{"kind": "route_cairn"}],
            "palettes": [{
                "art_direction_signature": u64::MAX.to_string(),
                "terrain_key": [80, 140, 72],
                "foliage_key": [68, 124, 64],
                "stone_key": [122, 112, 98]
            }]
        });

        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
        let checks = report.get("checks").and_then(Value::as_array).unwrap();
        for name in [
            "landmark_count_manifest_parity",
            "landmark_kind_count_manifest_parity",
            "palette_count_manifest_parity",
            "distinct_palette_count_manifest_parity",
            "terrain_biome_palette_count_manifest_parity",
            "foliage_palette_count_manifest_parity",
            "stone_palette_count_manifest_parity",
        ] {
            assert!(
                check_named(checks, name).is_some_and(|check| {
                    !check.get("passed").and_then(Value::as_bool).unwrap()
                }),
                "{name} should reject tampered aggregate counts"
            );
        }
    }

    #[test]
    fn audit_rejects_missing_palette_entries() {
        let manifest = json!({
            "schema": EXPECTED_VISUAL_CONTENT_EXPORT_SCHEMA,
            "counts": {
                "palette_count": EXPECTED_PALETTE_COUNT,
                "distinct_palette_count": EXPECTED_PALETTE_COUNT
            },
            "minimums": {},
            "landmarks": [],
            "palettes": []
        });

        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
        let checks = report.get("checks").and_then(Value::as_array).unwrap();
        for name in [
            "palette_count",
            "palette_count_manifest_parity",
            "distinct_palette_count",
            "distinct_palette_count_manifest_parity",
        ] {
            assert!(
                check_named(checks, name).is_some_and(|check| {
                    !check.get("passed").and_then(Value::as_bool).unwrap()
                }),
                "{name} should reject a manifest with missing palettes"
            );
        }
    }

    #[test]
    fn audit_rejects_low_shape_manifest() {
        let manifest = json!({
            "schema": "nau_visual_content_export.v2",
            "mesh_count": 0,
            "total_vertex_count": 0,
            "total_triangle_count": 0,
            "counts": {
                "ground_cover_count": 1,
                "ground_cover_patch_total": 10,
                "ground_cover_blade_total": 20,
                "tree_trunk_count": 1,
                "tree_canopy_count": 1,
                "rock_count": 1,
                "weather_cloud_count": 1,
                "weather_cloud_bank_count": 0,
                "weather_cloud_veil_count": 0,
                "landmark_count": 1,
                "landmark_kind_count": 1,
                "flora_cluster_count": 1,
                "flora_cluster_kind_count": 1,
                "ruin_complex_count": 1,
                "ruin_complex_kind_count": 1,
                "rock_formation_count": 1,
                "rock_formation_kind_count": 1,
                "water_detail_count": 1,
                "water_detail_kind_count": 1,
                "artifact_detail_count": 0,
                "artifact_detail_kind_count": 0,
                "artifact_stair_count": 0,
                "artifact_bridge_fragment_count": 0,
                "artifact_glyph_slab_count": 0,
                "artifact_retaining_wall_count": 0,
                "artifact_banner_count": 0,
                "artifact_pebble_field_count": 0,
                "artifact_reed_patch_count": 0,
                "small_island_count": 1,
                "plateau_landmark_count": 1,
                "plateau_waterfall_ribbon_count": 0,
                "plateau_waterfall_mist_count": 0,
                "route_waterfall_ribbon_count": 0,
                "route_waterfall_mist_count": 0,
                "route_lake_surface_count": 0,
                "river_channel_count": 0,
                "under_route_visual_count": 0,
                "under_route_cave_mouth_count": 0,
                "ruin_cluster_count": 0,
                "ruin_arch_count": 0,
                "route_cairn_count": 0,
                "launch_beacon_count": 0,
                "landing_garden_marker_count": 0,
                "pond_surface_count": 0,
                "obstruction_spire_count": 0
            },
            "minimums": {
                "ground_cover_patch_count": 1,
                "ground_cover_mesh_vertices": 10,
                "ground_cover_blade_count": 5,
                "ground_cover_blade_height_range_m": 0.1,
                "tree_trunk_mesh_vertices": 8,
                "tree_trunk_taper_ratio": 1.0,
                "tree_branch_reach_ratio": 1.0,
                "tree_branch_count": 1,
                "tree_root_flare_count": 0,
                "tree_trunk_ring_count": 2,
                "tree_trunk_height_range_m": 0.1,
                "tree_canopy_mesh_vertices": 45,
                "tree_canopy_lobe_count": 1,
                "tree_canopy_detail_card_count": 0,
                "tree_canopy_vertical_to_horizontal_ratio": 0.1,
                "tree_canopy_radius_range_m": 0.1,
                "rock_mesh_vertices": 8,
                "rock_vertical_span_m": 0.1,
                "weather_cloud_mesh_vertices": 45,
                "weather_cloud_lobe_count": 1,
                "weather_cloud_wisp_card_count": 0,
                "weather_cloud_filament_ribbon_detail_count": 0,
                "weather_cloud_bank_depth_m": 0.2,
                "weather_cloud_bank_lobe_count": 0,
                "weather_cloud_scaled_depth_span_m": 0.5,
                "route_cairn_mesh_vertices": 10,
                "route_cairn_vertical_span_m": 0.2,
                "launch_beacon_mesh_vertices": 10,
                "launch_beacon_vertical_span_m": 0.3,
                "landing_garden_marker_mesh_vertices": 6,
                "landing_garden_marker_vertical_span_m": 0.01,
                "pond_surface_mesh_vertices": 6,
                "pond_surface_vertical_span_m": 0.0,
                "plateau_landmark_vertex_total": 10,
                "max_plateau_landmark_mesh_vertices": 10,
                "plateau_waterfall_vertical_span_m": 3.0,
                "route_waterfall_vertical_span_m": 2.0,
                "route_lake_surface_horizontal_span_m": 4.0,
                "river_channel_horizontal_span_m": 0.5,
                "under_route_visual_vertical_span_m": 0.5,
                "surface_feature_vertex_total": 12,
                "flora_cluster_mesh_vertices": 3,
                "flora_cluster_horizontal_span_m": 0.1,
                "flora_cluster_vertical_span_m": 0.0,
                "ruin_complex_mesh_vertices": 3,
                "ruin_complex_horizontal_span_m": 0.1,
                "ruin_complex_vertical_span_m": 0.0,
                "rock_formation_mesh_vertices": 3,
                "rock_formation_horizontal_span_m": 0.1,
                "rock_formation_vertical_span_m": 0.0,
                "water_detail_mesh_vertices": 3,
                "water_detail_horizontal_span_m": 0.1,
                "water_detail_vertical_span_m": 0.0,
                "artifact_detail_vertex_total": 10,
                "artifact_detail_mesh_vertices": 4,
                "artifact_stone_mesh_vertices": 8,
                "artifact_stone_normal_slope_band_count": 1,
                "artifact_stair_horizontal_span_m": 0.5,
                "artifact_bridge_horizontal_span_m": 0.5,
                "artifact_banner_vertical_span_m": 0.2,
                "artifact_reed_vertical_span_m": 0.1,
                "ruin_arch_mesh_vertices": 10,
                "ruin_arch_vertical_span_m": 0.4,
                "ruin_arch_radius_band_count": 1,
                "ruin_arch_normal_slope_band_count": 1,
                "obstruction_spire_mesh_vertices": 8,
                "obstruction_spire_triangle_count": 12,
                "obstruction_spire_vertical_span_m": 0.4,
                "obstruction_spire_height_band_count": 2,
                "obstruction_spire_radius_band_count": 1,
                "obstruction_spire_normal_slope_band_count": 1,
                "terrain_biome_palette_count": 1,
                "foliage_palette_count": 1,
                "stone_palette_count": 1
            },
            "ground_cover": [],
            "trees": [],
            "rocks": [],
            "clouds": [],
            "landmarks": []
        });

        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
        assert!(!report.get("passed").and_then(Value::as_bool).unwrap());
        let checks = report.get("checks").and_then(Value::as_array).unwrap();
        assert!(
            check_named(checks, "tree_branch_reach_ratio")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "tree_root_flare_count")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "tree_trunk_ring_count")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "tree_trunk_height_range")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "tree_canopy_radius_range")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "weather_cloud_veil_count")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "weather_cloud_wisp_card_count")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "weather_cloud_filament_ribbon_detail_count")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "weather_cloud_scaled_depth_span")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        for name in [
            "ground_cover_patch_total",
            "ground_cover_blade_total",
            "tree_trunk_count",
            "tree_canopy_count",
            "rock_count",
            "ground_cover_patch_count",
            "ground_cover_mesh_vertices",
            "ground_cover_blade_count",
            "rock_mesh_vertices",
            "rock_vertical_span",
            "landmark_count",
            "landmark_kind_count",
            "flora_cluster_count",
            "flora_cluster_kind_count",
            "ruin_complex_count",
            "ruin_complex_kind_count",
            "rock_formation_count",
            "rock_formation_kind_count",
            "water_detail_count",
            "water_detail_kind_count",
            "artifact_detail_count",
            "artifact_detail_kind_count",
            "artifact_stair_count",
            "artifact_bridge_fragment_count",
            "artifact_glyph_slab_count",
            "artifact_retaining_wall_count",
            "artifact_banner_count",
            "artifact_pebble_field_count",
            "artifact_reed_patch_count",
            "small_island_count",
            "plateau_landmark_count",
            "plateau_waterfall_ribbon_count",
            "plateau_waterfall_mist_count",
            "route_waterfall_ribbon_count",
            "route_waterfall_mist_count",
            "route_lake_surface_count",
            "river_channel_count",
            "under_route_visual_count",
            "under_route_cave_mouth_count",
            "ruin_cluster_count",
            "ruin_arch_count",
            "route_cairn_count",
            "launch_beacon_count",
            "landing_garden_marker_count",
            "pond_surface_count",
            "obstruction_spire_count",
            "route_cairn_mesh_vertices",
            "route_cairn_vertical_span",
            "launch_beacon_mesh_vertices",
            "launch_beacon_vertical_span",
            "landing_garden_marker_mesh_vertices",
            "landing_garden_marker_vertical_span",
            "pond_surface_mesh_vertices",
            "pond_surface_vertical_span",
            "plateau_landmark_vertex_total",
            "max_plateau_landmark_mesh_vertices",
            "plateau_waterfall_vertical_span",
            "route_waterfall_vertical_span",
            "route_lake_surface_horizontal_span",
            "river_channel_horizontal_span",
            "under_route_visual_vertical_span",
            "surface_feature_vertex_total",
            "flora_cluster_mesh_vertices",
            "flora_cluster_horizontal_span",
            "flora_cluster_vertical_span",
            "ruin_complex_mesh_vertices",
            "ruin_complex_horizontal_span",
            "ruin_complex_vertical_span",
            "rock_formation_mesh_vertices",
            "rock_formation_horizontal_span",
            "rock_formation_vertical_span",
            "water_detail_mesh_vertices",
            "water_detail_horizontal_span",
            "water_detail_vertical_span",
            "artifact_detail_vertex_total",
            "artifact_detail_mesh_vertices",
            "artifact_stone_mesh_vertices",
            "artifact_stone_normal_slope_bands",
            "artifact_stair_horizontal_span",
            "artifact_bridge_horizontal_span",
            "artifact_banner_vertical_span",
            "artifact_reed_vertical_span",
            "ruin_arch_mesh_vertices",
            "ruin_arch_vertical_span",
            "ruin_arch_radius_bands",
            "ruin_arch_normal_slope_bands",
            "obstruction_spire_mesh_vertices",
            "obstruction_spire_triangle_count",
            "obstruction_spire_vertical_span",
            "obstruction_spire_height_bands",
            "obstruction_spire_radius_bands",
            "obstruction_spire_normal_slope_bands",
        ] {
            assert!(
                check_named(checks, name).is_some_and(|check| {
                    !check.get("passed").and_then(Value::as_bool).unwrap()
                }),
                "{name} should fail for primitive landmark regressions"
            );
        }
    }

    #[test]
    fn audit_rejects_missing_surface_feature_fields() {
        let manifest = json!({
            "schema": "nau_visual_content_export.v2",
            "counts": {},
            "minimums": {},
            "ground_cover": [],
            "trees": [],
            "rocks": [],
            "clouds": [],
            "landmarks": []
        });

        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
        let checks = report.get("checks").and_then(Value::as_array).unwrap();
        for name in [
            "flora_cluster_count",
            "flora_cluster_kind_count",
            "ruin_complex_count",
            "ruin_complex_kind_count",
            "rock_formation_count",
            "rock_formation_kind_count",
            "water_detail_count",
            "water_detail_kind_count",
            "surface_feature_vertex_total",
            "flora_cluster_mesh_vertices",
            "flora_cluster_horizontal_span",
            "flora_cluster_vertical_span",
            "ruin_complex_mesh_vertices",
            "ruin_complex_horizontal_span",
            "ruin_complex_vertical_span",
            "rock_formation_mesh_vertices",
            "rock_formation_horizontal_span",
            "rock_formation_vertical_span",
            "water_detail_mesh_vertices",
            "water_detail_horizontal_span",
            "water_detail_vertical_span",
        ] {
            assert!(
                check_named(checks, name).is_some_and(|check| {
                    !check.get("passed").and_then(Value::as_bool).unwrap()
                }),
                "{name} should fail when surface feature input is missing"
            );
        }
    }

    #[test]
    fn audit_checks_surface_feature_landmark_obj_parity() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nau-visual-content-audit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).expect("temp directory should be creatable");
        let obj_path = temp_dir.join("flora_cluster.obj");
        let mut file = fs::File::create(&obj_path).expect("obj should be creatable");
        writeln!(file, "v 0 0 0").unwrap();
        writeln!(file, "v 1 0 0").unwrap();
        writeln!(file, "v 0 1 0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();

        let manifest = json!({
            "schema": "nau_visual_content_export.v2",
            "mesh_count": 1,
            "counts": {},
            "minimums": {},
            "ground_cover": [],
            "trees": [],
            "rocks": [],
            "clouds": [],
            "landmarks": [{
                "kind": "flora_cluster_test",
                "surface_feature_family": "flora_cluster",
                "mesh": {
                    "obj": "flora_cluster.obj",
                    "vertex_count": 4,
                    "triangle_count": 2
                }
            }]
        });

        let report = audit_manifest(&manifest, &temp_dir, "manifest.json");
        let artifacts = report.get("artifacts").unwrap();
        assert_eq!(
            value_u64(artifacts, "vertex_mismatch_count"),
            1,
            "surface feature OBJ vertices should match the manifest"
        );
        assert_eq!(
            value_u64(artifacts, "face_mismatch_count"),
            1,
            "surface feature OBJ faces should match the manifest"
        );

        fs::remove_dir_all(temp_dir).expect("temp directory should be removable");
    }

    #[test]
    fn audit_rejects_surface_feature_claims_without_landmark_entries() {
        let manifest = json!({
            "schema": "nau_visual_content_export.v2",
            "mesh_count": 0,
            "counts": {
                "flora_cluster_count": 1,
                "flora_cluster_kind_count": 1,
                "ruin_complex_count": 1,
                "ruin_complex_kind_count": 1,
                "rock_formation_count": 1,
                "rock_formation_kind_count": 1,
                "water_detail_count": 1,
                "water_detail_kind_count": 1
            },
            "minimums": {
                "surface_feature_vertex_total": 100,
                "flora_cluster_mesh_vertices": 25,
                "flora_cluster_horizontal_span_m": 2.0,
                "flora_cluster_vertical_span_m": 1.0,
                "ruin_complex_mesh_vertices": 25,
                "ruin_complex_horizontal_span_m": 2.0,
                "ruin_complex_vertical_span_m": 1.0,
                "rock_formation_mesh_vertices": 25,
                "rock_formation_horizontal_span_m": 2.0,
                "rock_formation_vertical_span_m": 1.0,
                "water_detail_mesh_vertices": 25,
                "water_detail_horizontal_span_m": 2.0,
                "water_detail_vertical_span_m": 1.0
            },
            "ground_cover": [],
            "trees": [],
            "rocks": [],
            "clouds": [],
            "landmarks": []
        });

        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
        let checks = report.get("checks").and_then(Value::as_array).unwrap();
        for name in [
            "flora_cluster_count_manifest_parity",
            "ruin_complex_count_manifest_parity",
            "rock_formation_count_manifest_parity",
            "water_detail_count_manifest_parity",
            "surface_feature_vertex_total_manifest_parity",
            "flora_cluster_mesh_vertices_manifest_parity",
            "ruin_complex_horizontal_span_manifest_parity",
            "rock_formation_vertical_span_manifest_parity",
            "water_detail_mesh_vertices_manifest_parity",
        ] {
            assert!(
                check_named(checks, name).is_some_and(|check| {
                    !check.get("passed").and_then(Value::as_bool).unwrap()
                }),
                "{name} should reject claims without exported feature landmarks"
            );
        }
    }

    #[test]
    fn obj_audit_counts_vertices_and_faces() {
        let path = std::env::temp_dir().join(format!(
            "nau-visual-content-audit-{}-{}.obj",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        let mut file = fs::File::create(&path).expect("obj should be creatable");
        writeln!(file, "v 0 0 0").unwrap();
        writeln!(file, "v 1 0 0").unwrap();
        writeln!(file, "v 0 1 0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();

        let audit = audit_obj_path(&path).expect("obj should parse");
        assert_eq!(audit.vertex_count, 3);
        assert_eq!(audit.face_count, 1);

        fs::remove_file(path).expect("obj should be removable");
    }

    fn check_named<'a>(checks: &'a [Value], name: &str) -> Option<&'a Value> {
        checks.iter().find(|check| {
            check
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|check_name| check_name == name)
        })
    }
}
