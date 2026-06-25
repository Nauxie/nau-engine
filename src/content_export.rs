use crate::eval_runtime::{path_string, remove_existing_dir};
use crate::generated_content::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, GROUND_COVER_PATCHES,
    TERRAIN_BIOME_PALETTE_COUNT, TERRAIN_TEXTURE_SIZE, TREE_CANOPY_CARD_COUNT, TREE_TRUNK_SEGMENTS,
    VERTICES_PER_GROUND_BLADE, biome_detail_color_set, cloud_cluster_mesh, island_cliff_mesh,
    island_ground_cover_mesh, island_impostor_mesh, island_terrain_mesh, island_underside_mesh,
    mesh_terrain_material_channel_count, mesh_terrain_material_region_count,
    mesh_terrain_material_weight_band_count, mesh_vertex_color_band_count, mesh_y_range,
    procedural_terrain_surface_texture_data, terrain_biome_palette, texture_detail_band_count,
    texture_edge_promille, tree_canopy_mesh, tree_trunk_mesh,
};
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use nau_engine::world::{SkyIsland, SkyRoute};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(crate) struct TerrainExportReport {
    pub(crate) manifest_path: PathBuf,
    pub(crate) island_count: usize,
    pub(crate) mesh_count: usize,
    pub(crate) total_vertex_count: usize,
    pub(crate) total_triangle_count: usize,
    pub(crate) min_terrain_mesh_vertices: usize,
    pub(crate) min_terrain_color_bands: usize,
    pub(crate) min_terrain_material_weight_bands: usize,
    pub(crate) min_terrain_material_channels: usize,
    pub(crate) min_terrain_material_regions: usize,
    pub(crate) min_terrain_texture_detail_bands: usize,
    pub(crate) min_terrain_texture_edge_promille: usize,
    pub(crate) min_terrain_relief_range_m: f32,
    pub(crate) min_cliff_color_bands: usize,
    pub(crate) min_impostor_mesh_vertices: usize,
    pub(crate) min_impostor_color_bands: usize,
    pub(crate) islands: Vec<TerrainExportIslandSummary>,
}

#[derive(Debug)]
pub(crate) struct TerrainExportIslandSummary {
    pub(crate) index: usize,
    pub(crate) island: SkyIsland,
    pub(crate) slug: String,
    pub(crate) terrain: TerrainExportMeshSummary,
    pub(crate) cliff: TerrainExportMeshSummary,
    pub(crate) underside: TerrainExportMeshSummary,
    pub(crate) impostor: TerrainExportMeshSummary,
}

#[derive(Debug)]
pub(crate) struct TerrainExportMeshSummary {
    pub(crate) obj_path: PathBuf,
    pub(crate) material_weights_path: Option<PathBuf>,
    pub(crate) vertex_count: usize,
    pub(crate) triangle_count: usize,
    pub(crate) color_bands: usize,
    pub(crate) material_weight_bands: usize,
    pub(crate) material_channels: usize,
    pub(crate) material_regions: usize,
    pub(crate) relief_range_m: f32,
}

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

impl TerrainExportReport {
    fn to_json(&self) -> String {
        let islands = self
            .islands
            .iter()
            .map(|island| island.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            concat!(
                "{{\n",
                "  \"schema\": \"nau_terrain_export.v1\",\n",
                "  \"island_count\": {},\n",
                "  \"mesh_count\": {},\n",
                "  \"total_vertex_count\": {},\n",
                "  \"total_triangle_count\": {},\n",
                "  \"minimums\": {{\n",
                "    \"terrain_mesh_vertices\": {},\n",
                "    \"terrain_color_bands\": {},\n",
                "    \"terrain_material_weight_bands\": {},\n",
                "    \"terrain_material_channels\": {},\n",
                "    \"terrain_material_regions\": {},\n",
                "    \"terrain_texture_detail_bands\": {},\n",
                "    \"terrain_texture_edge_promille\": {},\n",
                "    \"terrain_relief_range_m\": {},\n",
                "    \"cliff_color_bands\": {},\n",
                "    \"impostor_mesh_vertices\": {},\n",
                "    \"impostor_color_bands\": {}\n",
                "  }},\n",
                "  \"islands\": [\n",
                "{}\n",
                "  ]\n",
                "}}\n"
            ),
            self.island_count,
            self.mesh_count,
            self.total_vertex_count,
            self.total_triangle_count,
            self.min_terrain_mesh_vertices,
            self.min_terrain_color_bands,
            self.min_terrain_material_weight_bands,
            self.min_terrain_material_channels,
            self.min_terrain_material_regions,
            self.min_terrain_texture_detail_bands,
            self.min_terrain_texture_edge_promille,
            terrain_export_json_number(self.min_terrain_relief_range_m),
            self.min_cliff_color_bands,
            self.min_impostor_mesh_vertices,
            self.min_impostor_color_bands,
            islands
        )
    }
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

impl TerrainExportIslandSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"index\": {},\n\
             {indent}  \"name\": {},\n\
             {indent}  \"slug\": {},\n\
             {indent}  \"center\": {},\n\
             {indent}  \"half_extents\": {},\n\
             {indent}  \"thickness_m\": {},\n\
             {indent}  \"target\": {},\n\
             {indent}  \"terrain\": {},\n\
             {indent}  \"cliff\": {},\n\
             {indent}  \"underside\": {},\n\
             {indent}  \"impostor\": {}\n\
             {indent}}}",
            self.index,
            terrain_export_json_string(self.island.name),
            terrain_export_json_string(&self.slug),
            terrain_export_json_vec3(self.island.center),
            terrain_export_json_vec2(self.island.half_extents),
            terrain_export_json_number(self.island.thickness),
            self.island.is_target,
            self.terrain.to_json(),
            self.cliff.to_json(),
            self.underside.to_json(),
            self.impostor.to_json(),
        )
    }
}

impl TerrainExportMeshSummary {
    fn to_json(&self) -> String {
        let material_weights_path = self
            .material_weights_path
            .as_deref()
            .map(|path| terrain_export_json_string(&path_string(path)))
            .unwrap_or_else(|| "null".to_string());

        format!(
            concat!(
                "{{\"obj\": {}, \"material_weights_csv\": {}, ",
                "\"vertex_count\": {}, \"triangle_count\": {}, ",
                "\"color_bands\": {}, \"material_weight_bands\": {}, ",
                "\"material_channels\": {}, \"material_regions\": {}, \"relief_range_m\": {}}}"
            ),
            terrain_export_json_string(&path_string(&self.obj_path)),
            material_weights_path,
            self.vertex_count,
            self.triangle_count,
            self.color_bands,
            self.material_weight_bands,
            self.material_channels,
            self.material_regions,
            terrain_export_json_number(self.relief_range_m)
        )
    }
}

pub(crate) fn export_terrain_inspection(output_dir: &Path) -> std::io::Result<TerrainExportReport> {
    fs::create_dir_all(output_dir)?;
    let islands_dir = output_dir.join("islands");
    remove_existing_dir(&islands_dir)?;
    fs::create_dir_all(&islands_dir)?;

    let route = SkyRoute::default();
    let mut islands = Vec::with_capacity(route.islands().len());

    for (index, island) in route.islands().iter().copied().enumerate() {
        let slug = terrain_export_slug(island.name);
        let prefix = format!("{index:02}_{slug}");
        let terrain_mesh = island_terrain_mesh(index, island);
        let cliff_mesh = island_cliff_mesh(index, island);
        let underside_mesh = island_underside_mesh(index, island);
        let impostor_mesh = island_impostor_mesh(index, island);

        let terrain_obj = PathBuf::from("islands").join(format!("{prefix}_terrain.obj"));
        let terrain_material_weights =
            PathBuf::from("islands").join(format!("{prefix}_terrain_material_weights.csv"));
        let cliff_obj = PathBuf::from("islands").join(format!("{prefix}_cliff.obj"));
        let underside_obj = PathBuf::from("islands").join(format!("{prefix}_underside.obj"));
        let impostor_obj = PathBuf::from("islands").join(format!("{prefix}_impostor.obj"));

        write_mesh_obj(&output_dir.join(&terrain_obj), &terrain_mesh, "terrain")?;
        write_terrain_material_weights_csv(
            &output_dir.join(&terrain_material_weights),
            &terrain_mesh,
        )?;
        write_mesh_obj(&output_dir.join(&cliff_obj), &cliff_mesh, "cliff")?;
        write_mesh_obj(
            &output_dir.join(&underside_obj),
            &underside_mesh,
            "underside",
        )?;
        write_mesh_obj(&output_dir.join(&impostor_obj), &impostor_mesh, "impostor")?;

        islands.push(TerrainExportIslandSummary {
            index,
            island,
            slug,
            terrain: terrain_export_mesh_summary(
                terrain_obj,
                Some(terrain_material_weights),
                &terrain_mesh,
            ),
            cliff: terrain_export_mesh_summary(cliff_obj, None, &cliff_mesh),
            underside: terrain_export_mesh_summary(underside_obj, None, &underside_mesh),
            impostor: terrain_export_mesh_summary(impostor_obj, None, &impostor_mesh),
        });
    }

    let island_count = islands.len();
    let mesh_count = island_count * 4;
    let total_vertex_count = islands
        .iter()
        .map(|island| {
            island.terrain.vertex_count
                + island.cliff.vertex_count
                + island.underside.vertex_count
                + island.impostor.vertex_count
        })
        .sum();
    let total_triangle_count = islands
        .iter()
        .map(|island| {
            island.terrain.triangle_count
                + island.cliff.triangle_count
                + island.underside.triangle_count
                + island.impostor.triangle_count
        })
        .sum();
    let min_terrain_mesh_vertices = islands
        .iter()
        .map(|island| island.terrain.vertex_count)
        .min()
        .unwrap_or(0);
    let min_terrain_color_bands = islands
        .iter()
        .map(|island| island.terrain.color_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_material_weight_bands = islands
        .iter()
        .map(|island| island.terrain.material_weight_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_material_channels = islands
        .iter()
        .map(|island| island.terrain.material_channels)
        .min()
        .unwrap_or(0);
    let min_terrain_material_regions = islands
        .iter()
        .map(|island| island.terrain.material_regions)
        .min()
        .unwrap_or(0);
    let min_terrain_texture_detail_bands = terrain_export_texture_detail_band_floor();
    let min_terrain_texture_edge_promille = terrain_export_texture_edge_promille_floor();
    let min_terrain_relief_range_m = islands
        .iter()
        .map(|island| island.terrain.relief_range_m)
        .min_by(f32::total_cmp)
        .unwrap_or(0.0);
    let min_cliff_color_bands = islands
        .iter()
        .flat_map(|island| [island.cliff.color_bands, island.underside.color_bands])
        .min()
        .unwrap_or(0);
    let min_impostor_mesh_vertices = islands
        .iter()
        .map(|island| island.impostor.vertex_count)
        .min()
        .unwrap_or(0);
    let min_impostor_color_bands = islands
        .iter()
        .map(|island| island.impostor.color_bands)
        .min()
        .unwrap_or(0);

    let manifest_path = output_dir.join("manifest.json");
    let report = TerrainExportReport {
        manifest_path,
        island_count,
        mesh_count,
        total_vertex_count,
        total_triangle_count,
        min_terrain_mesh_vertices,
        min_terrain_color_bands,
        min_terrain_material_weight_bands,
        min_terrain_material_channels,
        min_terrain_material_regions,
        min_terrain_texture_detail_bands,
        min_terrain_texture_edge_promille,
        min_terrain_relief_range_m,
        min_cliff_color_bands,
        min_impostor_mesh_vertices,
        min_impostor_color_bands,
        islands,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
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

fn terrain_export_mesh_summary(
    obj_path: PathBuf,
    material_weights_path: Option<PathBuf>,
    mesh: &Mesh,
) -> TerrainExportMeshSummary {
    TerrainExportMeshSummary {
        obj_path,
        material_weights_path,
        vertex_count: mesh.count_vertices(),
        triangle_count: mesh_index_values(mesh).len() / 3,
        color_bands: mesh_vertex_color_band_count(mesh),
        material_weight_bands: mesh_terrain_material_weight_band_count(mesh),
        material_channels: mesh_terrain_material_channel_count(mesh),
        material_regions: mesh_terrain_material_region_count(mesh),
        relief_range_m: mesh_y_range(mesh),
    }
}

fn terrain_export_texture_detail_band_floor() -> usize {
    terrain_export_texture_metric_floor(texture_detail_band_count)
}

fn terrain_export_texture_edge_promille_floor() -> usize {
    terrain_export_texture_metric_floor(|data| texture_edge_promille(data, TERRAIN_TEXTURE_SIZE))
}

fn terrain_export_texture_metric_floor(mut metric: impl FnMut(&[u8]) -> usize) -> usize {
    [
        (
            [54, 128, 70, 255],
            [28, 92, 48, 255],
            [128, 174, 78, 255],
            17,
        ),
        (
            [96, 138, 70, 255],
            [56, 104, 54, 255],
            [166, 172, 90, 255],
            19,
        ),
        (
            [126, 104, 76, 255],
            [80, 70, 60, 255],
            [162, 138, 96, 255],
            23,
        ),
        (
            [52, 110, 118, 255],
            [30, 80, 94, 255],
            [142, 176, 164, 255],
            29,
        ),
        (
            [132, 132, 92, 255],
            [86, 96, 70, 255],
            [178, 166, 112, 255],
            31,
        ),
        (
            [70, 150, 94, 255],
            [34, 100, 62, 255],
            [156, 198, 112, 255],
            37,
        ),
    ]
    .into_iter()
    .map(|(primary, secondary, accent, seed)| {
        let data = procedural_terrain_surface_texture_data(
            primary,
            secondary,
            accent,
            seed,
            TERRAIN_TEXTURE_SIZE,
        );
        metric(&data)
    })
    .min()
    .unwrap_or(0)
}

fn write_mesh_obj(path: &Path, mesh: &Mesh, object_name: &str) -> std::io::Result<()> {
    let positions = mesh_positions(mesh);
    let normals = mesh_normals(mesh).filter(|normals| normals.len() == positions.len());
    let uvs = mesh_uv0(mesh).filter(|uvs| uvs.len() == positions.len());
    let colors = mesh_colors(mesh).filter(|colors| colors.len() == positions.len());
    let indices = mesh_index_values(mesh);
    let mut file = File::create(path)?;

    writeln!(file, "# NAU terrain export")?;
    writeln!(file, "o {}", terrain_export_slug(object_name))?;
    for (index, position) in positions.iter().enumerate() {
        if let Some(colors) = colors {
            let color = colors[index];
            writeln!(
                file,
                "v {:.4} {:.4} {:.4} {:.4} {:.4} {:.4}",
                position[0], position[1], position[2], color[0], color[1], color[2]
            )?;
        } else {
            writeln!(
                file,
                "v {:.4} {:.4} {:.4}",
                position[0], position[1], position[2]
            )?;
        }
    }
    if let Some(uvs) = uvs {
        for uv in uvs {
            writeln!(file, "vt {:.4} {:.4}", uv[0], uv[1])?;
        }
    }
    if let Some(normals) = normals {
        for normal in normals {
            writeln!(
                file,
                "vn {:.4} {:.4} {:.4}",
                normal[0], normal[1], normal[2]
            )?;
        }
    }

    let has_uvs = uvs.is_some();
    let has_normals = normals.is_some();
    for triangle in indices.chunks_exact(3) {
        writeln!(
            file,
            "f {} {} {}",
            obj_face_index(triangle[0], has_uvs, has_normals),
            obj_face_index(triangle[1], has_uvs, has_normals),
            obj_face_index(triangle[2], has_uvs, has_normals)
        )?;
    }

    Ok(())
}

fn write_terrain_material_weights_csv(path: &Path, mesh: &Mesh) -> std::io::Result<()> {
    let Some(weights) = mesh_terrain_material_weights(mesh) else {
        return Ok(());
    };
    let mut file = File::create(path)?;
    writeln!(file, "vertex,lush_highland,exposed_edge")?;
    for (index, weight) in weights.iter().enumerate() {
        writeln!(file, "{index},{:.4},{:.4}", weight[0], weight[1])?;
    }
    Ok(())
}

fn obj_face_index(index: u32, has_uvs: bool, has_normals: bool) -> String {
    let obj_index = index + 1;
    match (has_uvs, has_normals) {
        (true, true) => format!("{obj_index}/{obj_index}/{obj_index}"),
        (true, false) => format!("{obj_index}/{obj_index}"),
        (false, true) => format!("{obj_index}//{obj_index}"),
        (false, false) => obj_index.to_string(),
    }
}

fn mesh_positions(mesh: &Mesh) -> &[[f32; 3]] {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => &[],
    }
}

fn mesh_normals(mesh: &Mesh) -> Option<&[[f32; 3]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => Some(values),
        _ => None,
    }
}

pub(crate) fn mesh_uv0(mesh: &Mesh) -> Option<&[[f32; 2]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values),
        _ => None,
    }
}

fn mesh_colors(mesh: &Mesh) -> Option<&[[f32; 4]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => Some(values),
        _ => None,
    }
}

fn mesh_terrain_material_weights(mesh: &Mesh) -> Option<&[[f32; 2]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_1) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values),
        _ => None,
    }
}

fn mesh_index_values(mesh: &Mesh) -> Vec<u32> {
    match mesh.indices() {
        Some(Indices::U16(values)) => values.iter().map(|index| u32::from(*index)).collect(),
        Some(Indices::U32(values)) => values.clone(),
        None => (0..mesh.count_vertices() as u32).collect(),
    }
}

fn terrain_export_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;

    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }

    if last_was_separator {
        slug.pop();
    }
    if slug.is_empty() {
        "unnamed".to_string()
    } else {
        slug
    }
}

pub(crate) fn terrain_export_json_vec3(value: Vec3) -> String {
    format!(
        "[{}, {}, {}]",
        terrain_export_json_number(value.x),
        terrain_export_json_number(value.y),
        terrain_export_json_number(value.z)
    )
}

fn terrain_export_json_vec2(value: Vec2) -> String {
    format!(
        "[{}, {}]",
        terrain_export_json_number(value.x),
        terrain_export_json_number(value.y)
    )
}

pub(crate) fn terrain_export_json_number(value: f32) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "0.0000".to_string()
    }
}

pub(crate) fn terrain_export_json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => output.push_str(&format!("\\u{:04x}", value as u32)),
            value => output.push(value),
        }
    }
    output.push('"');
    output
}
