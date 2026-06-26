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
    pub(crate) weather_cloud_count: usize,
    pub(crate) weather_cloud_bank_count: usize,
    pub(crate) min_ground_cover_mesh_vertices: usize,
    pub(crate) min_ground_cover_blade_count: usize,
    pub(crate) min_ground_cover_blade_height_range_m: f32,
    pub(crate) min_tree_trunk_mesh_vertices: usize,
    pub(crate) min_tree_trunk_taper_ratio: f32,
    pub(crate) min_tree_branch_reach_ratio: f32,
    pub(crate) min_tree_canopy_mesh_vertices: usize,
    pub(crate) min_tree_canopy_lobe_count: usize,
    pub(crate) min_tree_canopy_detail_card_count: usize,
    pub(crate) min_tree_canopy_vertical_to_horizontal_ratio: f32,
    pub(crate) min_weather_cloud_mesh_vertices: usize,
    pub(crate) min_weather_cloud_lobe_count: usize,
    pub(crate) min_weather_cloud_wisp_card_count: usize,
    pub(crate) min_weather_cloud_bank_depth_m: f32,
    pub(crate) min_weather_cloud_bank_lobe_count: usize,
    pub(crate) terrain_biome_palette_count: usize,
    pub(crate) foliage_palette_count: usize,
    pub(crate) stone_palette_count: usize,
    pub(crate) ground_cover: Vec<VisualGroundCoverSummary>,
    pub(crate) trees: Vec<VisualTreeSummary>,
    pub(crate) clouds: Vec<VisualCloudSummary>,
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
    pub(crate) canopy_lobe_count: usize,
    pub(crate) canopy_detail_card_count: usize,
    pub(crate) canopy_vertical_to_horizontal_ratio: f32,
}

#[derive(Debug)]
pub(crate) struct VisualCloudSummary {
    pub(crate) island_name: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) bank: bool,
    pub(crate) mesh: VisualMeshSummary,
    pub(crate) lobe_count: usize,
    pub(crate) wisp_card_count: usize,
    pub(crate) scaled_horizontal_span_m: f32,
    pub(crate) scaled_vertical_depth_m: f32,
    pub(crate) scaled_depth_span_m: f32,
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
        let clouds = self
            .clouds
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
                "    \"weather_cloud_count\": {},\n",
                "    \"weather_cloud_bank_count\": {}\n",
                "  }},\n",
                "  \"minimums\": {{\n",
                "    \"ground_cover_mesh_vertices\": {},\n",
                "    \"ground_cover_blade_count\": {},\n",
                "    \"ground_cover_blade_height_range_m\": {},\n",
                "    \"tree_trunk_mesh_vertices\": {},\n",
                "    \"tree_trunk_taper_ratio\": {},\n",
                "    \"tree_branch_reach_ratio\": {},\n",
                "    \"tree_canopy_mesh_vertices\": {},\n",
                "    \"tree_canopy_lobe_count\": {},\n",
                "    \"tree_canopy_detail_card_count\": {},\n",
                "    \"tree_canopy_vertical_to_horizontal_ratio\": {},\n",
                "    \"weather_cloud_mesh_vertices\": {},\n",
                "    \"weather_cloud_lobe_count\": {},\n",
                "    \"weather_cloud_wisp_card_count\": {},\n",
                "    \"weather_cloud_bank_depth_m\": {},\n",
                "    \"weather_cloud_bank_lobe_count\": {},\n",
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
                "  \"clouds\": [\n",
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
            self.weather_cloud_count,
            self.weather_cloud_bank_count,
            self.min_ground_cover_mesh_vertices,
            self.min_ground_cover_blade_count,
            terrain_export_json_number(self.min_ground_cover_blade_height_range_m),
            self.min_tree_trunk_mesh_vertices,
            terrain_export_json_number(self.min_tree_trunk_taper_ratio),
            terrain_export_json_number(self.min_tree_branch_reach_ratio),
            self.min_tree_canopy_mesh_vertices,
            self.min_tree_canopy_lobe_count,
            self.min_tree_canopy_detail_card_count,
            terrain_export_json_number(self.min_tree_canopy_vertical_to_horizontal_ratio),
            self.min_weather_cloud_mesh_vertices,
            self.min_weather_cloud_lobe_count,
            self.min_weather_cloud_wisp_card_count,
            terrain_export_json_number(self.min_weather_cloud_bank_depth_m),
            self.min_weather_cloud_bank_lobe_count,
            self.terrain_biome_palette_count,
            self.foliage_palette_count,
            self.stone_palette_count,
            ground_cover,
            trees,
            clouds,
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
            self.canopy_lobe_count,
            self.canopy_detail_card_count,
            terrain_export_json_number(self.canopy_vertical_to_horizontal_ratio)
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
            terrain_export_json_number(self.scaled_horizontal_span_m),
            terrain_export_json_number(self.scaled_vertical_depth_m),
            terrain_export_json_number(self.scaled_depth_span_m)
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
