use super::palette::visual_content_json_u8_triplet;
use crate::{
    content_export::shared::{terrain_export_json_number, terrain_export_json_string},
    eval_runtime::path_string,
};
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct VisualContentExportReport {
    pub(crate) manifest_path: PathBuf,
    pub(crate) mesh_count: usize,
    pub(crate) total_vertex_count: usize,
    pub(crate) total_triangle_count: usize,
    pub(crate) ground_cover_count: usize,
    pub(crate) ground_cover_patch_total: usize,
    pub(crate) ground_cover_blade_total: usize,
    pub(crate) tree_trunk_count: usize,
    pub(crate) tree_canopy_count: usize,
    pub(crate) rock_count: usize,
    pub(crate) weather_cloud_count: usize,
    pub(crate) weather_cloud_bank_count: usize,
    pub(crate) weather_cloud_veil_count: usize,
    pub(crate) landmark_count: usize,
    pub(crate) landmark_kind_count: usize,
    pub(crate) flora_cluster_count: usize,
    pub(crate) flora_cluster_kind_count: usize,
    pub(crate) ruin_complex_count: usize,
    pub(crate) ruin_complex_kind_count: usize,
    pub(crate) rock_formation_count: usize,
    pub(crate) rock_formation_kind_count: usize,
    pub(crate) water_detail_count: usize,
    pub(crate) water_detail_kind_count: usize,
    pub(crate) artifact_detail_count: usize,
    pub(crate) artifact_detail_kind_count: usize,
    pub(crate) artifact_stair_count: usize,
    pub(crate) artifact_bridge_fragment_count: usize,
    pub(crate) artifact_glyph_slab_count: usize,
    pub(crate) artifact_retaining_wall_count: usize,
    pub(crate) artifact_banner_count: usize,
    pub(crate) artifact_pebble_field_count: usize,
    pub(crate) artifact_reed_patch_count: usize,
    pub(crate) small_island_count: usize,
    pub(crate) plateau_landmark_count: usize,
    pub(crate) plateau_waterfall_ribbon_count: usize,
    pub(crate) plateau_waterfall_mist_count: usize,
    pub(crate) route_waterfall_ribbon_count: usize,
    pub(crate) route_waterfall_mist_count: usize,
    pub(crate) route_lake_surface_count: usize,
    pub(crate) river_channel_count: usize,
    pub(crate) under_route_visual_count: usize,
    pub(crate) under_route_cave_mouth_count: usize,
    pub(crate) ruin_cluster_count: usize,
    pub(crate) ruin_arch_count: usize,
    pub(crate) route_cairn_count: usize,
    pub(crate) launch_beacon_count: usize,
    pub(crate) landing_garden_marker_count: usize,
    pub(crate) pond_surface_count: usize,
    pub(crate) obstruction_spire_count: usize,
    pub(crate) min_ground_cover_patch_count: usize,
    pub(crate) min_ground_cover_mesh_vertices: usize,
    pub(crate) min_ground_cover_blade_count: usize,
    pub(crate) min_ground_cover_blade_height_range_m: f32,
    pub(crate) min_tree_trunk_mesh_vertices: usize,
    pub(crate) min_tree_trunk_taper_ratio: f32,
    pub(crate) min_tree_branch_reach_ratio: f32,
    pub(crate) min_tree_branch_count: usize,
    pub(crate) min_tree_root_flare_count: usize,
    pub(crate) min_tree_trunk_ring_count: usize,
    pub(crate) tree_trunk_height_range_m: f32,
    pub(crate) min_tree_canopy_mesh_vertices: usize,
    pub(crate) min_tree_canopy_lobe_count: usize,
    pub(crate) min_tree_canopy_detail_card_count: usize,
    pub(crate) min_tree_canopy_vertical_to_horizontal_ratio: f32,
    pub(crate) tree_canopy_radius_range_m: f32,
    pub(crate) min_rock_mesh_vertices: usize,
    pub(crate) min_rock_vertical_span_m: f32,
    pub(crate) min_weather_cloud_mesh_vertices: usize,
    pub(crate) min_weather_cloud_lobe_count: usize,
    pub(crate) min_weather_cloud_wisp_card_count: usize,
    pub(crate) min_weather_cloud_filament_ribbon_detail_count: usize,
    pub(crate) min_weather_cloud_bank_depth_m: f32,
    pub(crate) min_weather_cloud_bank_lobe_count: usize,
    pub(crate) min_weather_cloud_scaled_depth_span_m: f32,
    pub(crate) min_route_cairn_mesh_vertices: usize,
    pub(crate) min_route_cairn_vertical_span_m: f32,
    pub(crate) min_launch_beacon_mesh_vertices: usize,
    pub(crate) min_launch_beacon_vertical_span_m: f32,
    pub(crate) min_landing_garden_marker_mesh_vertices: usize,
    pub(crate) min_landing_garden_marker_vertical_span_m: f32,
    pub(crate) min_pond_surface_mesh_vertices: usize,
    pub(crate) min_pond_surface_vertical_span_m: f32,
    pub(crate) plateau_landmark_vertex_total: usize,
    pub(crate) max_plateau_landmark_mesh_vertices: usize,
    pub(crate) min_plateau_waterfall_vertical_span_m: f32,
    pub(crate) min_route_waterfall_vertical_span_m: f32,
    pub(crate) min_route_lake_surface_horizontal_span_m: f32,
    pub(crate) min_river_channel_horizontal_span_m: f32,
    pub(crate) min_under_route_visual_vertical_span_m: f32,
    pub(crate) surface_feature_vertex_total: usize,
    pub(crate) min_flora_cluster_mesh_vertices: usize,
    pub(crate) min_flora_cluster_horizontal_span_m: f32,
    pub(crate) min_flora_cluster_vertical_span_m: f32,
    pub(crate) min_ruin_complex_mesh_vertices: usize,
    pub(crate) min_ruin_complex_horizontal_span_m: f32,
    pub(crate) min_ruin_complex_vertical_span_m: f32,
    pub(crate) min_rock_formation_mesh_vertices: usize,
    pub(crate) min_rock_formation_horizontal_span_m: f32,
    pub(crate) min_rock_formation_vertical_span_m: f32,
    pub(crate) min_water_detail_mesh_vertices: usize,
    pub(crate) min_water_detail_horizontal_span_m: f32,
    pub(crate) min_water_detail_vertical_span_m: f32,
    pub(crate) artifact_detail_vertex_total: usize,
    pub(crate) min_artifact_detail_mesh_vertices: usize,
    pub(crate) min_artifact_stone_mesh_vertices: usize,
    pub(crate) min_artifact_stone_normal_slope_band_count: usize,
    pub(crate) min_artifact_stair_horizontal_span_m: f32,
    pub(crate) min_artifact_bridge_horizontal_span_m: f32,
    pub(crate) min_artifact_banner_vertical_span_m: f32,
    pub(crate) min_artifact_reed_vertical_span_m: f32,
    pub(crate) min_ruin_arch_mesh_vertices: usize,
    pub(crate) min_ruin_arch_vertical_span_m: f32,
    pub(crate) min_ruin_arch_radius_band_count: usize,
    pub(crate) min_ruin_arch_normal_slope_band_count: usize,
    pub(crate) min_obstruction_spire_mesh_vertices: usize,
    pub(crate) min_obstruction_spire_triangle_count: usize,
    pub(crate) min_obstruction_spire_vertical_span_m: f32,
    pub(crate) min_obstruction_spire_height_band_count: usize,
    pub(crate) min_obstruction_spire_radius_band_count: usize,
    pub(crate) min_obstruction_spire_normal_slope_band_count: usize,
    pub(crate) terrain_biome_palette_count: usize,
    pub(crate) foliage_palette_count: usize,
    pub(crate) stone_palette_count: usize,
    pub(crate) ground_cover: Vec<VisualGroundCoverSummary>,
    pub(crate) trees: Vec<VisualTreeSummary>,
    pub(crate) rocks: Vec<VisualRockSummary>,
    pub(crate) clouds: Vec<VisualCloudSummary>,
    pub(crate) landmarks: Vec<VisualLandmarkSummary>,
    pub(crate) palettes: Vec<VisualPaletteSummary>,
}

#[derive(Debug)]
pub(crate) struct VisualMeshSummary {
    pub(crate) obj_path: PathBuf,
    pub(crate) vertex_count: usize,
    pub(crate) triangle_count: usize,
    pub(crate) horizontal_span_m: f32,
    pub(crate) vertical_span_m: f32,
    pub(crate) depth_span_m: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GroundCoverBladeStats {
    pub(crate) blade_count: usize,
    pub(crate) min_height_m: f32,
    pub(crate) max_height_m: f32,
    pub(crate) height_range_m: f32,
}

#[derive(Debug)]
pub(crate) struct VisualGroundCoverSummary {
    pub(crate) island_name: &'static str,
    pub(crate) island_slug: String,
    pub(crate) mesh: VisualMeshSummary,
    pub(crate) patch_count: usize,
    pub(crate) blade_count: usize,
    pub(crate) min_blade_height_m: f32,
    pub(crate) max_blade_height_m: f32,
    pub(crate) blade_height_range_m: f32,
}

#[derive(Debug)]
pub(crate) struct VisualTreeSummary {
    pub(crate) island_name: &'static str,
    pub(crate) label: String,
    pub(crate) trunk: VisualMeshSummary,
    pub(crate) canopy: VisualMeshSummary,
    pub(crate) trunk_height_m: f32,
    pub(crate) canopy_radius_m: f32,
    pub(crate) trunk_taper_ratio: f32,
    pub(crate) branch_reach_ratio: f32,
    pub(crate) branch_count: usize,
    pub(crate) root_flare_count: usize,
    pub(crate) trunk_ring_count: usize,
    pub(crate) canopy_lobe_count: usize,
    pub(crate) canopy_detail_card_count: usize,
    pub(crate) canopy_vertical_to_horizontal_ratio: f32,
}

#[derive(Debug)]
pub(crate) struct VisualRockSummary {
    pub(crate) island_name: &'static str,
    pub(crate) label: String,
    pub(crate) mesh: VisualMeshSummary,
    pub(crate) scale_m: f32,
}

#[derive(Debug)]
pub(crate) struct VisualCloudSummary {
    pub(crate) island_name: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) bank: bool,
    pub(crate) mesh: VisualMeshSummary,
    pub(crate) lobe_count: usize,
    pub(crate) wisp_card_count: usize,
    pub(crate) filament_ribbon_detail_count: usize,
    pub(crate) scaled_horizontal_span_m: f32,
    pub(crate) scaled_vertical_depth_m: f32,
    pub(crate) scaled_depth_span_m: f32,
}

#[derive(Debug)]
pub(crate) struct VisualLandmarkSummary {
    pub(crate) island_name: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) label: String,
    pub(crate) mesh: VisualMeshSummary,
    pub(crate) height_band_count: usize,
    pub(crate) radius_band_count: usize,
    pub(crate) normal_slope_band_count: usize,
    pub(crate) surface_feature_family: Option<VisualSurfaceFeatureFamily>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VisualSurfaceFeatureFamily {
    FloraCluster,
    RuinComplex,
    RockFormation,
    WaterDetail,
}

impl VisualSurfaceFeatureFamily {
    fn label(self) -> &'static str {
        match self {
            Self::FloraCluster => "flora_cluster",
            Self::RuinComplex => "ruin_complex",
            Self::RockFormation => "rock_formation",
            Self::WaterDetail => "water_detail",
        }
    }
}

#[derive(Debug)]
pub(crate) struct VisualPaletteSummary {
    pub(crate) index: usize,
    pub(crate) terrain_key: [u8; 3],
    pub(crate) foliage_key: [u8; 3],
    pub(crate) stone_key: [u8; 3],
}

impl VisualContentExportReport {
    pub(super) fn to_json(&self) -> String {
        let ground_cover = self
            .ground_cover
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let trees = self
            .trees
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let rocks = self
            .rocks
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let clouds = self
            .clouds
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let landmarks = self
            .landmarks
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let palettes = self
            .palettes
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            concat!(
                "{{\n",
                "  \"schema\": \"nau_visual_content_export.v1\",\n",
                "  \"mesh_count\": {},\n",
                "  \"total_vertex_count\": {},\n",
                "  \"total_triangle_count\": {},\n",
                "  \"counts\": {{\n",
                "    \"ground_cover_count\": {},\n",
                "    \"ground_cover_patch_total\": {},\n",
                "    \"ground_cover_blade_total\": {},\n",
                "    \"tree_trunk_count\": {},\n",
                "    \"tree_canopy_count\": {},\n",
                "    \"rock_count\": {},\n",
                "    \"weather_cloud_count\": {},\n",
                "    \"weather_cloud_bank_count\": {},\n",
                "    \"weather_cloud_veil_count\": {},\n",
                "    \"landmark_count\": {},\n",
                "    \"landmark_kind_count\": {},\n",
                "    \"flora_cluster_count\": {},\n",
                "    \"flora_cluster_kind_count\": {},\n",
                "    \"ruin_complex_count\": {},\n",
                "    \"ruin_complex_kind_count\": {},\n",
                "    \"rock_formation_count\": {},\n",
                "    \"rock_formation_kind_count\": {},\n",
                "    \"water_detail_count\": {},\n",
                "    \"water_detail_kind_count\": {},\n",
                "    \"artifact_detail_count\": {},\n",
                "    \"artifact_detail_kind_count\": {},\n",
                "    \"artifact_stair_count\": {},\n",
                "    \"artifact_bridge_fragment_count\": {},\n",
                "    \"artifact_glyph_slab_count\": {},\n",
                "    \"artifact_retaining_wall_count\": {},\n",
                "    \"artifact_banner_count\": {},\n",
                "    \"artifact_pebble_field_count\": {},\n",
                "    \"artifact_reed_patch_count\": {},\n",
                "    \"small_island_count\": {},\n",
                "    \"plateau_landmark_count\": {},\n",
                "    \"plateau_waterfall_ribbon_count\": {},\n",
                "    \"plateau_waterfall_mist_count\": {},\n",
                "    \"route_waterfall_ribbon_count\": {},\n",
                "    \"route_waterfall_mist_count\": {},\n",
                "    \"route_lake_surface_count\": {},\n",
                "    \"river_channel_count\": {},\n",
                "    \"under_route_visual_count\": {},\n",
                "    \"under_route_cave_mouth_count\": {},\n",
                "    \"ruin_cluster_count\": {},\n",
                "    \"ruin_arch_count\": {},\n",
                "    \"route_cairn_count\": {},\n",
                "    \"launch_beacon_count\": {},\n",
                "    \"landing_garden_marker_count\": {},\n",
                "    \"pond_surface_count\": {},\n",
                "    \"obstruction_spire_count\": {}\n",
                "  }},\n",
                "  \"minimums\": {{\n",
                "    \"ground_cover_patch_count\": {},\n",
                "    \"ground_cover_mesh_vertices\": {},\n",
                "    \"ground_cover_blade_count\": {},\n",
                "    \"ground_cover_blade_height_range_m\": {},\n",
                "    \"tree_trunk_mesh_vertices\": {},\n",
                "    \"tree_trunk_taper_ratio\": {},\n",
                "    \"tree_branch_reach_ratio\": {},\n",
                "    \"tree_branch_count\": {},\n",
                "    \"tree_root_flare_count\": {},\n",
                "    \"tree_trunk_ring_count\": {},\n",
                "    \"tree_trunk_height_range_m\": {},\n",
                "    \"tree_canopy_mesh_vertices\": {},\n",
                "    \"tree_canopy_lobe_count\": {},\n",
                "    \"tree_canopy_detail_card_count\": {},\n",
                "    \"tree_canopy_vertical_to_horizontal_ratio\": {},\n",
                "    \"tree_canopy_radius_range_m\": {},\n",
                "    \"rock_mesh_vertices\": {},\n",
                "    \"rock_vertical_span_m\": {},\n",
                "    \"weather_cloud_mesh_vertices\": {},\n",
                "    \"weather_cloud_lobe_count\": {},\n",
                "    \"weather_cloud_wisp_card_count\": {},\n",
                "    \"weather_cloud_filament_ribbon_detail_count\": {},\n",
                "    \"weather_cloud_bank_depth_m\": {},\n",
                "    \"weather_cloud_bank_lobe_count\": {},\n",
                "    \"weather_cloud_scaled_depth_span_m\": {},\n",
                "    \"route_cairn_mesh_vertices\": {},\n",
                "    \"route_cairn_vertical_span_m\": {},\n",
                "    \"launch_beacon_mesh_vertices\": {},\n",
                "    \"launch_beacon_vertical_span_m\": {},\n",
                "    \"landing_garden_marker_mesh_vertices\": {},\n",
                "    \"landing_garden_marker_vertical_span_m\": {},\n",
                "    \"pond_surface_mesh_vertices\": {},\n",
                "    \"pond_surface_vertical_span_m\": {},\n",
                "    \"plateau_landmark_vertex_total\": {},\n",
                "    \"max_plateau_landmark_mesh_vertices\": {},\n",
                "    \"plateau_waterfall_vertical_span_m\": {},\n",
                "    \"route_waterfall_vertical_span_m\": {},\n",
                "    \"route_lake_surface_horizontal_span_m\": {},\n",
                "    \"river_channel_horizontal_span_m\": {},\n",
                "    \"under_route_visual_vertical_span_m\": {},\n",
                "    \"surface_feature_vertex_total\": {},\n",
                "    \"flora_cluster_mesh_vertices\": {},\n",
                "    \"flora_cluster_horizontal_span_m\": {},\n",
                "    \"flora_cluster_vertical_span_m\": {},\n",
                "    \"ruin_complex_mesh_vertices\": {},\n",
                "    \"ruin_complex_horizontal_span_m\": {},\n",
                "    \"ruin_complex_vertical_span_m\": {},\n",
                "    \"rock_formation_mesh_vertices\": {},\n",
                "    \"rock_formation_horizontal_span_m\": {},\n",
                "    \"rock_formation_vertical_span_m\": {},\n",
                "    \"water_detail_mesh_vertices\": {},\n",
                "    \"water_detail_horizontal_span_m\": {},\n",
                "    \"water_detail_vertical_span_m\": {},\n",
                "    \"artifact_detail_vertex_total\": {},\n",
                "    \"artifact_detail_mesh_vertices\": {},\n",
                "    \"artifact_stone_mesh_vertices\": {},\n",
                "    \"artifact_stone_normal_slope_band_count\": {},\n",
                "    \"artifact_stair_horizontal_span_m\": {},\n",
                "    \"artifact_bridge_horizontal_span_m\": {},\n",
                "    \"artifact_banner_vertical_span_m\": {},\n",
                "    \"artifact_reed_vertical_span_m\": {},\n",
                "    \"ruin_arch_mesh_vertices\": {},\n",
                "    \"ruin_arch_vertical_span_m\": {},\n",
                "    \"ruin_arch_radius_band_count\": {},\n",
                "    \"ruin_arch_normal_slope_band_count\": {},\n",
                "    \"obstruction_spire_mesh_vertices\": {},\n",
                "    \"obstruction_spire_triangle_count\": {},\n",
                "    \"obstruction_spire_vertical_span_m\": {},\n",
                "    \"obstruction_spire_height_band_count\": {},\n",
                "    \"obstruction_spire_radius_band_count\": {},\n",
                "    \"obstruction_spire_normal_slope_band_count\": {},\n",
                "    \"terrain_biome_palette_count\": {},\n",
                "    \"foliage_palette_count\": {},\n",
                "    \"stone_palette_count\": {}\n",
                "  }},\n",
                "  \"ground_cover\": [\n",
                "{}\n",
                "  ],\n",
                "  \"trees\": [\n",
                "{}\n",
                "  ],\n",
                "  \"rocks\": [\n",
                "{}\n",
                "  ],\n",
                "  \"clouds\": [\n",
                "{}\n",
                "  ],\n",
                "  \"landmarks\": [\n",
                "{}\n",
                "  ],\n",
                "  \"palettes\": [\n",
                "{}\n",
                "  ]\n",
                "}}\n"
            ),
            self.mesh_count,
            self.total_vertex_count,
            self.total_triangle_count,
            self.ground_cover_count,
            self.ground_cover_patch_total,
            self.ground_cover_blade_total,
            self.tree_trunk_count,
            self.tree_canopy_count,
            self.rock_count,
            self.weather_cloud_count,
            self.weather_cloud_bank_count,
            self.weather_cloud_veil_count,
            self.landmark_count,
            self.landmark_kind_count,
            self.flora_cluster_count,
            self.flora_cluster_kind_count,
            self.ruin_complex_count,
            self.ruin_complex_kind_count,
            self.rock_formation_count,
            self.rock_formation_kind_count,
            self.water_detail_count,
            self.water_detail_kind_count,
            self.artifact_detail_count,
            self.artifact_detail_kind_count,
            self.artifact_stair_count,
            self.artifact_bridge_fragment_count,
            self.artifact_glyph_slab_count,
            self.artifact_retaining_wall_count,
            self.artifact_banner_count,
            self.artifact_pebble_field_count,
            self.artifact_reed_patch_count,
            self.small_island_count,
            self.plateau_landmark_count,
            self.plateau_waterfall_ribbon_count,
            self.plateau_waterfall_mist_count,
            self.route_waterfall_ribbon_count,
            self.route_waterfall_mist_count,
            self.route_lake_surface_count,
            self.river_channel_count,
            self.under_route_visual_count,
            self.under_route_cave_mouth_count,
            self.ruin_cluster_count,
            self.ruin_arch_count,
            self.route_cairn_count,
            self.launch_beacon_count,
            self.landing_garden_marker_count,
            self.pond_surface_count,
            self.obstruction_spire_count,
            self.min_ground_cover_patch_count,
            self.min_ground_cover_mesh_vertices,
            self.min_ground_cover_blade_count,
            terrain_export_json_number(self.min_ground_cover_blade_height_range_m),
            self.min_tree_trunk_mesh_vertices,
            terrain_export_json_number(self.min_tree_trunk_taper_ratio),
            terrain_export_json_number(self.min_tree_branch_reach_ratio),
            self.min_tree_branch_count,
            self.min_tree_root_flare_count,
            self.min_tree_trunk_ring_count,
            terrain_export_json_number(self.tree_trunk_height_range_m),
            self.min_tree_canopy_mesh_vertices,
            self.min_tree_canopy_lobe_count,
            self.min_tree_canopy_detail_card_count,
            terrain_export_json_number(self.min_tree_canopy_vertical_to_horizontal_ratio),
            terrain_export_json_number(self.tree_canopy_radius_range_m),
            self.min_rock_mesh_vertices,
            terrain_export_json_number(self.min_rock_vertical_span_m),
            self.min_weather_cloud_mesh_vertices,
            self.min_weather_cloud_lobe_count,
            self.min_weather_cloud_wisp_card_count,
            self.min_weather_cloud_filament_ribbon_detail_count,
            terrain_export_json_number(self.min_weather_cloud_bank_depth_m),
            self.min_weather_cloud_bank_lobe_count,
            terrain_export_json_number(self.min_weather_cloud_scaled_depth_span_m),
            self.min_route_cairn_mesh_vertices,
            terrain_export_json_number(self.min_route_cairn_vertical_span_m),
            self.min_launch_beacon_mesh_vertices,
            terrain_export_json_number(self.min_launch_beacon_vertical_span_m),
            self.min_landing_garden_marker_mesh_vertices,
            terrain_export_json_number(self.min_landing_garden_marker_vertical_span_m),
            self.min_pond_surface_mesh_vertices,
            terrain_export_json_number(self.min_pond_surface_vertical_span_m),
            self.plateau_landmark_vertex_total,
            self.max_plateau_landmark_mesh_vertices,
            terrain_export_json_number(self.min_plateau_waterfall_vertical_span_m),
            terrain_export_json_number(self.min_route_waterfall_vertical_span_m),
            terrain_export_json_number(self.min_route_lake_surface_horizontal_span_m),
            terrain_export_json_number(self.min_river_channel_horizontal_span_m),
            terrain_export_json_number(self.min_under_route_visual_vertical_span_m),
            self.surface_feature_vertex_total,
            self.min_flora_cluster_mesh_vertices,
            terrain_export_json_number(self.min_flora_cluster_horizontal_span_m),
            terrain_export_json_number(self.min_flora_cluster_vertical_span_m),
            self.min_ruin_complex_mesh_vertices,
            terrain_export_json_number(self.min_ruin_complex_horizontal_span_m),
            terrain_export_json_number(self.min_ruin_complex_vertical_span_m),
            self.min_rock_formation_mesh_vertices,
            terrain_export_json_number(self.min_rock_formation_horizontal_span_m),
            terrain_export_json_number(self.min_rock_formation_vertical_span_m),
            self.min_water_detail_mesh_vertices,
            terrain_export_json_number(self.min_water_detail_horizontal_span_m),
            terrain_export_json_number(self.min_water_detail_vertical_span_m),
            self.artifact_detail_vertex_total,
            self.min_artifact_detail_mesh_vertices,
            self.min_artifact_stone_mesh_vertices,
            self.min_artifact_stone_normal_slope_band_count,
            terrain_export_json_number(self.min_artifact_stair_horizontal_span_m),
            terrain_export_json_number(self.min_artifact_bridge_horizontal_span_m),
            terrain_export_json_number(self.min_artifact_banner_vertical_span_m),
            terrain_export_json_number(self.min_artifact_reed_vertical_span_m),
            self.min_ruin_arch_mesh_vertices,
            terrain_export_json_number(self.min_ruin_arch_vertical_span_m),
            self.min_ruin_arch_radius_band_count,
            self.min_ruin_arch_normal_slope_band_count,
            self.min_obstruction_spire_mesh_vertices,
            self.min_obstruction_spire_triangle_count,
            terrain_export_json_number(self.min_obstruction_spire_vertical_span_m),
            self.min_obstruction_spire_height_band_count,
            self.min_obstruction_spire_radius_band_count,
            self.min_obstruction_spire_normal_slope_band_count,
            self.terrain_biome_palette_count,
            self.foliage_palette_count,
            self.stone_palette_count,
            ground_cover,
            trees,
            rocks,
            clouds,
            landmarks,
            palettes
        )
    }
}

impl VisualMeshSummary {
    fn to_json(&self) -> String {
        format!(
            concat!(
                "{{\"obj\": {}, \"vertex_count\": {}, \"triangle_count\": {}, ",
                "\"horizontal_span_m\": {}, \"vertical_span_m\": {}, \"depth_span_m\": {}}}"
            ),
            terrain_export_json_string(&path_string(&self.obj_path)),
            self.vertex_count,
            self.triangle_count,
            terrain_export_json_number(self.horizontal_span_m),
            terrain_export_json_number(self.vertical_span_m),
            terrain_export_json_number(self.depth_span_m)
        )
    }
}

impl VisualGroundCoverSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"island_slug\": {},\n\
             {indent}  \"mesh\": {},\n\
             {indent}  \"patch_count\": {},\n\
             {indent}  \"blade_count\": {},\n\
             {indent}  \"min_blade_height_m\": {},\n\
             {indent}  \"max_blade_height_m\": {},\n\
             {indent}  \"blade_height_range_m\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(&self.island_slug),
            self.mesh.to_json(),
            self.patch_count,
            self.blade_count,
            terrain_export_json_number(self.min_blade_height_m),
            terrain_export_json_number(self.max_blade_height_m),
            terrain_export_json_number(self.blade_height_range_m)
        )
    }
}

impl VisualTreeSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"label\": {},\n\
             {indent}  \"trunk\": {},\n\
             {indent}  \"canopy\": {},\n\
             {indent}  \"trunk_height_m\": {},\n\
             {indent}  \"canopy_radius_m\": {},\n\
             {indent}  \"trunk_taper_ratio\": {},\n\
             {indent}  \"branch_reach_ratio\": {},\n\
             {indent}  \"branch_count\": {},\n\
             {indent}  \"root_flare_count\": {},\n\
             {indent}  \"trunk_ring_count\": {},\n\
             {indent}  \"canopy_lobe_count\": {},\n\
             {indent}  \"canopy_detail_card_count\": {},\n\
             {indent}  \"canopy_vertical_to_horizontal_ratio\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(&self.label),
            self.trunk.to_json(),
            self.canopy.to_json(),
            terrain_export_json_number(self.trunk_height_m),
            terrain_export_json_number(self.canopy_radius_m),
            terrain_export_json_number(self.trunk_taper_ratio),
            terrain_export_json_number(self.branch_reach_ratio),
            self.branch_count,
            self.root_flare_count,
            self.trunk_ring_count,
            self.canopy_lobe_count,
            self.canopy_detail_card_count,
            terrain_export_json_number(self.canopy_vertical_to_horizontal_ratio)
        )
    }
}

impl VisualRockSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"label\": {},\n\
             {indent}  \"mesh\": {},\n\
             {indent}  \"scale_m\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(&self.label),
            self.mesh.to_json(),
            terrain_export_json_number(self.scale_m)
        )
    }
}

impl VisualCloudSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"kind\": {},\n\
             {indent}  \"bank\": {},\n\
             {indent}  \"mesh\": {},\n\
             {indent}  \"lobe_count\": {},\n\
             {indent}  \"wisp_card_count\": {},\n\
             {indent}  \"filament_ribbon_detail_count\": {},\n\
             {indent}  \"scaled_horizontal_span_m\": {},\n\
             {indent}  \"scaled_vertical_depth_m\": {},\n\
             {indent}  \"scaled_depth_span_m\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(self.kind),
            self.bank,
            self.mesh.to_json(),
            self.lobe_count,
            self.wisp_card_count,
            self.filament_ribbon_detail_count,
            terrain_export_json_number(self.scaled_horizontal_span_m),
            terrain_export_json_number(self.scaled_vertical_depth_m),
            terrain_export_json_number(self.scaled_depth_span_m)
        )
    }
}

impl VisualLandmarkSummary {
    fn to_json(&self, indent: &str) -> String {
        let surface_feature_family = self.surface_feature_family.map_or_else(
            || "null".to_string(),
            |family| terrain_export_json_string(family.label()),
        );
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"kind\": {},\n\
             {indent}  \"label\": {},\n\
             {indent}  \"surface_feature_family\": {},\n\
             {indent}  \"mesh\": {},\n\
             {indent}  \"height_band_count\": {},\n\
             {indent}  \"radius_band_count\": {},\n\
             {indent}  \"normal_slope_band_count\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(self.kind),
            terrain_export_json_string(&self.label),
            surface_feature_family,
            self.mesh.to_json(),
            self.height_band_count,
            self.radius_band_count,
            self.normal_slope_band_count
        )
    }
}

impl VisualPaletteSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\"index\": {}, \"terrain_key\": {}, \"foliage_key\": {}, \"stone_key\": {}}}",
            self.index,
            visual_content_json_u8_triplet(self.terrain_key),
            visual_content_json_u8_triplet(self.foliage_key),
            visual_content_json_u8_triplet(self.stone_key)
        )
    }
}
