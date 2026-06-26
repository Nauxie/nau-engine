use super::{metrics::visual_content_mesh_summary, types::VisualCloudSummary};
use crate::{
    content_export::shared::write_mesh_obj,
    generated_content::{
        CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, cloud_cluster_mesh,
    },
};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;
use std::path::{Path, PathBuf};

pub(super) fn visual_content_cloud_summaries(
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
pub(super) fn write_visual_cloud_summary(
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
