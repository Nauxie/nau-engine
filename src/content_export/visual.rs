use super::shared::{
    mesh_index_values, mesh_positions, terrain_export_json_number, terrain_export_json_string,
    terrain_export_slug, write_mesh_obj,
};
use crate::eval_runtime::{path_string, remove_existing_dir};
use crate::generated_content::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, GROUND_COVER_PATCHES,
    TERRAIN_BIOME_PALETTE_COUNT, TREE_CANOPY_CARD_COUNT, TREE_TRUNK_SEGMENTS,
    VERTICES_PER_GROUND_BLADE, biome_detail_color_set, cloud_cluster_mesh,
    island_ground_cover_mesh, terrain_biome_palette, tree_canopy_mesh, tree_trunk_mesh,
};
use bevy::prelude::*;
use nau_engine::world::{SkyIsland, SkyRoute};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

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
    fn to_json(&self) -> String {
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

pub(crate) fn export_visual_content_inspection(
    output_dir: &Path,
) -> std::io::Result<VisualContentExportReport> {
    fs::create_dir_all(output_dir)?;
    let visuals_dir = output_dir.join("visuals");
    remove_existing_dir(&visuals_dir)?;
    fs::create_dir_all(&visuals_dir)?;

    let route = SkyRoute::default();
    let mut ground_cover = Vec::with_capacity(route.islands().len());
    let mut trees = Vec::new();
    let mut clouds = Vec::new();

    for (island_index, island) in route.islands().iter().copied().enumerate() {
        let island_slug = terrain_export_slug(island.name);
        let ground_mesh = island_ground_cover_mesh(island_index, island);
        let ground_obj = PathBuf::from("visuals")
            .join(format!("{island_index:02}_{island_slug}_ground_cover.obj"));
        write_mesh_obj(&output_dir.join(&ground_obj), &ground_mesh, "ground cover")?;
        let blade_stats = ground_cover_blade_stats(&ground_mesh);
        ground_cover.push(VisualGroundCoverSummary {
            island_name: island.name,
            island_slug: island_slug.clone(),
            mesh: visual_content_mesh_summary(ground_obj, &ground_mesh),
            patch_count: GROUND_COVER_PATCHES,
            blade_count: blade_stats.blade_count,
            min_blade_height_m: blade_stats.min_height_m,
            max_blade_height_m: blade_stats.max_height_m,
            blade_height_range_m: blade_stats.height_range_m,
        });

        for tree in visual_content_tree_specs(island_index, island) {
            let tree_slug = terrain_export_slug(&tree.label);
            let trunk_mesh = tree_trunk_mesh(tree.trunk_radius_m, tree.trunk_height_m, tree.seed);
            let canopy_mesh = tree_canopy_mesh(tree.canopy_radius_m, tree.canopy_seed);
            let trunk_obj = PathBuf::from("visuals").join(format!(
                "{island_index:02}_{island_slug}_{tree_slug}_trunk.obj"
            ));
            let canopy_obj = PathBuf::from("visuals").join(format!(
                "{island_index:02}_{island_slug}_{tree_slug}_canopy.obj"
            ));
            write_mesh_obj(&output_dir.join(&trunk_obj), &trunk_mesh, "tree trunk")?;
            write_mesh_obj(&output_dir.join(&canopy_obj), &canopy_mesh, "tree canopy")?;

            let (trunk_taper_ratio, branch_reach_ratio) = tree_trunk_shape_metrics(&trunk_mesh);
            let trunk = visual_content_mesh_summary(trunk_obj, &trunk_mesh);
            let canopy = visual_content_mesh_summary(canopy_obj, &canopy_mesh);
            let canopy_horizontal_span = canopy.horizontal_span_m.max(canopy.depth_span_m);
            let canopy_vertical_to_horizontal_ratio =
                finite_ratio(canopy.vertical_span_m, canopy_horizontal_span);

            trees.push(VisualTreeSummary {
                island_name: island.name,
                label: tree.label,
                trunk,
                canopy,
                trunk_height_m: tree.trunk_height_m,
                canopy_radius_m: tree.canopy_radius_m,
                trunk_taper_ratio,
                branch_reach_ratio,
                canopy_lobe_count: tree_canopy_lobe_count(),
                canopy_detail_card_count: TREE_CANOPY_CARD_COUNT,
                canopy_vertical_to_horizontal_ratio,
            });
        }

        clouds.extend(visual_content_cloud_summaries(
            output_dir,
            island_index,
            island,
            &island_slug,
        )?);
    }

    let palettes = (0..TERRAIN_BIOME_PALETTE_COUNT)
        .map(visual_content_palette_summary)
        .collect::<Vec<_>>();
    let terrain_biome_palette_count = palettes
        .iter()
        .map(|palette| palette.terrain_key)
        .collect::<HashSet<_>>()
        .len();
    let foliage_palette_count = palettes
        .iter()
        .map(|palette| palette.foliage_key)
        .collect::<HashSet<_>>()
        .len();
    let stone_palette_count = palettes
        .iter()
        .map(|palette| palette.stone_key)
        .collect::<HashSet<_>>()
        .len();

    let mesh_count = ground_cover.len() + trees.len() * 2 + clouds.len();
    let total_vertex_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.vertex_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.vertex_count, summary.canopy.vertex_count]),
        )
        .chain(clouds.iter().map(|summary| summary.mesh.vertex_count))
        .sum();
    let total_triangle_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.triangle_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.triangle_count, summary.canopy.triangle_count]),
        )
        .chain(clouds.iter().map(|summary| summary.mesh.triangle_count))
        .sum();

    let manifest_path = output_dir.join("manifest.json");
    let report = VisualContentExportReport {
        manifest_path,
        mesh_count,
        total_vertex_count,
        total_triangle_count,
        ground_cover_count: ground_cover.len(),
        ground_cover_patch_total: ground_cover.iter().map(|summary| summary.patch_count).sum(),
        ground_cover_blade_total: ground_cover.iter().map(|summary| summary.blade_count).sum(),
        tree_trunk_count: trees.len(),
        tree_canopy_count: trees.len(),
        weather_cloud_count: clouds.len(),
        weather_cloud_bank_count: clouds.iter().filter(|summary| summary.bank).count(),
        min_ground_cover_mesh_vertices: ground_cover
            .iter()
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_ground_cover_blade_count: ground_cover
            .iter()
            .map(|summary| summary.blade_count)
            .min()
            .unwrap_or(0),
        min_ground_cover_blade_height_range_m: min_finite_f32(
            ground_cover
                .iter()
                .map(|summary| summary.blade_height_range_m),
        ),
        min_tree_trunk_mesh_vertices: trees
            .iter()
            .map(|summary| summary.trunk.vertex_count)
            .min()
            .unwrap_or(0),
        min_tree_trunk_taper_ratio: min_finite_f32(
            trees.iter().map(|summary| summary.trunk_taper_ratio),
        ),
        min_tree_branch_reach_ratio: min_finite_f32(
            trees.iter().map(|summary| summary.branch_reach_ratio),
        ),
        min_tree_canopy_mesh_vertices: trees
            .iter()
            .map(|summary| summary.canopy.vertex_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_lobe_count: trees
            .iter()
            .map(|summary| summary.canopy_lobe_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_detail_card_count: trees
            .iter()
            .map(|summary| summary.canopy_detail_card_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_vertical_to_horizontal_ratio: min_finite_f32(
            trees
                .iter()
                .map(|summary| summary.canopy_vertical_to_horizontal_ratio),
        ),
        min_weather_cloud_mesh_vertices: clouds
            .iter()
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_lobe_count: clouds
            .iter()
            .map(|summary| summary.lobe_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_wisp_card_count: clouds
            .iter()
            .map(|summary| summary.wisp_card_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_bank_depth_m: min_finite_f32(
            clouds
                .iter()
                .filter(|summary| summary.bank)
                .map(|summary| summary.scaled_vertical_depth_m),
        ),
        min_weather_cloud_bank_lobe_count: clouds
            .iter()
            .filter(|summary| summary.bank)
            .map(|summary| summary.lobe_count)
            .min()
            .unwrap_or(0),
        terrain_biome_palette_count,
        foliage_palette_count,
        stone_palette_count,
        ground_cover,
        trees,
        clouds,
        palettes,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
}

#[derive(Debug)]
struct VisualTreeSpec {
    label: String,
    trunk_radius_m: f32,
    trunk_height_m: f32,
    seed: u32,
    canopy_radius_m: f32,
    canopy_seed: u32,
}

fn visual_content_tree_specs(island_index: usize, island: SkyIsland) -> Vec<VisualTreeSpec> {
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

fn visual_content_cloud_summaries(
    output_dir: &Path,
    island_index: usize,
    island: SkyIsland,
    island_slug: &str,
) -> std::io::Result<Vec<VisualCloudSummary>> {
    let mut clouds = Vec::new();
    let bank_scale = Vec3::new(
        island.half_extents.x * 0.45 + 18.0,
        3.8 + (island_index % 3) as f32 * 0.55,
        island.half_extents.y * 0.26 + 8.0,
    );
    clouds.push(write_visual_cloud_summary(
        output_dir,
        island.name,
        "bank",
        true,
        island_index,
        island_slug,
        0,
        CLOUD_BANK_LOBES,
        bank_scale,
        2_000 + island_index as u32 * 37,
    )?);

    if island_index.is_multiple_of(2) {
        for puff_index in 0..3 {
            let veil_scale = Vec3::new(
                island.half_extents.x * 0.36 + 14.0 + puff_index as f32 * 4.0,
                0.52 + puff_index as f32 * 0.12,
                island.half_extents.y * 0.13 + 6.0 + puff_index as f32 * 1.8,
            );
            clouds.push(write_visual_cloud_summary(
                output_dir,
                island.name,
                "veil",
                false,
                island_index,
                island_slug,
                puff_index + 1,
                CLOUD_VEIL_LOBES,
                veil_scale,
                3_000 + island_index as u32 * 53 + puff_index as u32 * 11,
            )?);
        }
    }

    Ok(clouds)
}

#[allow(clippy::too_many_arguments)]
fn write_visual_cloud_summary(
    output_dir: &Path,
    island_name: &'static str,
    kind: &'static str,
    bank: bool,
    island_index: usize,
    island_slug: &str,
    cloud_index: usize,
    lobe_count: usize,
    scale: Vec3,
    seed: u32,
) -> std::io::Result<VisualCloudSummary> {
    let mesh = cloud_cluster_mesh(seed, lobe_count);
    let obj_path = PathBuf::from("visuals").join(format!(
        "{island_index:02}_{island_slug}_{kind}_{cloud_index}.obj"
    ));
    write_mesh_obj(&output_dir.join(&obj_path), &mesh, kind)?;
    let mesh_summary = visual_content_mesh_summary(obj_path, &mesh);

    Ok(VisualCloudSummary {
        island_name,
        kind,
        bank,
        scaled_horizontal_span_m: mesh_summary.horizontal_span_m * scale.x,
        scaled_vertical_depth_m: mesh_summary.vertical_span_m * scale.y,
        scaled_depth_span_m: mesh_summary.depth_span_m * scale.z,
        mesh: mesh_summary,
        lobe_count,
        wisp_card_count: lobe_count * CLOUD_WISP_CARDS_PER_LOBE,
    })
}

fn visual_content_mesh_summary(obj_path: PathBuf, mesh: &Mesh) -> VisualMeshSummary {
    let (horizontal_span_m, vertical_span_m, depth_span_m) = mesh_bounds(mesh)
        .map_or((0.0, 0.0, 0.0), |(min, max)| {
            (max.x - min.x, max.y - min.y, max.z - min.z)
        });

    VisualMeshSummary {
        obj_path,
        vertex_count: mesh.count_vertices(),
        triangle_count: mesh_index_values(mesh).len() / 3,
        horizontal_span_m,
        vertical_span_m,
        depth_span_m,
    }
}

fn mesh_bounds(mesh: &Mesh) -> Option<(Vec3, Vec3)> {
    let positions = mesh_positions(mesh);
    let first = positions.first()?;
    let mut min = Vec3::from_array(*first);
    let mut max = min;

    for position in positions.iter().skip(1) {
        let position = Vec3::from_array(*position);
        min = min.min(position);
        max = max.max(position);
    }

    Some((min, max))
}

fn ground_cover_blade_stats(mesh: &Mesh) -> GroundCoverBladeStats {
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

fn tree_trunk_shape_metrics(mesh: &Mesh) -> (f32, f32) {
    let positions = mesh_positions(mesh);
    let top_ring_start = TREE_TRUNK_SEGMENTS * 2;
    let branch_vertices_start = TREE_TRUNK_SEGMENTS * 3 + 2;
    if positions.len() <= branch_vertices_start {
        return (0.0, 0.0);
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

    (taper_ratio, branch_reach_ratio)
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

fn tree_canopy_lobe_count() -> usize {
    1 + 5
}

fn visual_content_palette_summary(index: usize) -> VisualPaletteSummary {
    let terrain = terrain_biome_palette(index);
    let detail = biome_detail_color_set(index);

    VisualPaletteSummary {
        index,
        terrain_key: visual_content_vec3_key(terrain.grass),
        foliage_key: visual_content_rgba_key(detail.foliage_primary),
        stone_key: visual_content_rgba_key(detail.stone_primary),
    }
}

fn visual_content_vec3_key(color: Vec3) -> [u8; 3] {
    [
        (color.x.clamp(0.0, 1.0) * 31.0).round() as u8,
        (color.y.clamp(0.0, 1.0) * 31.0).round() as u8,
        (color.z.clamp(0.0, 1.0) * 31.0).round() as u8,
    ]
}

fn visual_content_rgba_key(color: [u8; 4]) -> [u8; 3] {
    [color[0] / 8, color[1] / 8, color[2] / 8]
}

fn visual_content_json_u8_triplet(value: [u8; 3]) -> String {
    format!("[{}, {}, {}]", value[0], value[1], value[2])
}

fn min_finite_f32(values: impl Iterator<Item = f32>) -> f32 {
    values
        .filter(|value| value.is_finite())
        .min_by(f32::total_cmp)
        .unwrap_or(0.0)
}

fn finite_ratio(numerator: f32, denominator: f32) -> f32 {
    if denominator.abs() <= f32::EPSILON {
        return 0.0;
    }
    let ratio = numerator / denominator;
    if ratio.is_finite() { ratio } else { 0.0 }
}
