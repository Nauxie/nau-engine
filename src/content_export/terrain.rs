use super::shared::{
    mesh_index_values, terrain_export_json_number, terrain_export_json_string,
    terrain_export_json_vec2, terrain_export_json_vec3, terrain_export_slug, write_mesh_obj,
    write_terrain_material_weights_csv,
};
use crate::eval_runtime::{path_string, remove_existing_dir};
use crate::generated_content::{
    TERRAIN_TEXTURE_SIZE, island_cliff_mesh, island_impostor_mesh, island_terrain_mesh,
    island_underside_mesh, mesh_normal_slope_band_count, mesh_terrain_material_channel_count,
    mesh_terrain_material_region_count, mesh_terrain_material_weight_band_count,
    mesh_vertex_color_band_count, mesh_vertical_band_count, mesh_y_range,
    procedural_terrain_surface_texture_data, texture_detail_band_count, texture_edge_promille,
};
use bevy::prelude::*;
use nau_engine::world::{SkyIsland, SkyRoute};
use std::{
    fs,
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
    pub(crate) min_terrain_height_bands: usize,
    pub(crate) min_terrain_normal_slope_bands: usize,
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
    pub(crate) height_bands: usize,
    pub(crate) normal_slope_bands: usize,
    pub(crate) relief_range_m: f32,
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
                "    \"terrain_height_bands\": {},\n",
                "    \"terrain_normal_slope_bands\": {},\n",
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
            self.min_terrain_height_bands,
            self.min_terrain_normal_slope_bands,
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
                "\"material_channels\": {}, \"material_regions\": {}, ",
                "\"height_bands\": {}, \"normal_slope_bands\": {}, \"relief_range_m\": {}}}"
            ),
            terrain_export_json_string(&path_string(&self.obj_path)),
            material_weights_path,
            self.vertex_count,
            self.triangle_count,
            self.color_bands,
            self.material_weight_bands,
            self.material_channels,
            self.material_regions,
            self.height_bands,
            self.normal_slope_bands,
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
    let min_terrain_height_bands = islands
        .iter()
        .map(|island| island.terrain.height_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_normal_slope_bands = islands
        .iter()
        .map(|island| island.terrain.normal_slope_bands)
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
        min_terrain_height_bands,
        min_terrain_normal_slope_bands,
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
        height_bands: mesh_vertical_band_count(mesh),
        normal_slope_bands: mesh_normal_slope_band_count(mesh),
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
