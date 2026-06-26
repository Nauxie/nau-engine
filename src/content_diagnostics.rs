use crate::generated_content::TERRAIN_BIOME_PALETTE_COUNT;
use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct IslandContentDiagnostics {
    pub(crate) island_terrain_surface_count: usize,
    pub(crate) min_island_terrain_mesh_vertices: usize,
    pub(crate) min_island_terrain_color_bands: usize,
    pub(crate) min_island_terrain_material_weight_bands: usize,
    pub(crate) min_island_terrain_material_channels: usize,
    pub(crate) min_island_terrain_material_regions: usize,
    pub(crate) min_island_terrain_texture_detail_bands: usize,
    pub(crate) min_island_terrain_relief_range_cm: usize,
    pub(crate) min_island_cliff_color_bands: usize,
    pub(crate) min_island_impostor_mesh_vertices: usize,
    pub(crate) min_island_impostor_color_bands: usize,
    pub(crate) procedural_island_body_count: usize,
    pub(crate) primitive_island_body_count: usize,
    pub(crate) min_island_body_silhouette_segments: usize,
    pub(crate) max_island_body_silhouette_segments: usize,
    pub(crate) total_island_body_silhouette_segments: usize,
    pub(crate) min_island_body_mesh_vertices: usize,
    pub(crate) max_island_body_mesh_vertices: usize,
    pub(crate) generated_ground_cover_patch_count: usize,
    pub(crate) min_ground_cover_blade_count: usize,
    pub(crate) min_ground_cover_mesh_vertices: usize,
    pub(crate) generated_tree_trunk_count: usize,
    pub(crate) generated_tree_canopy_count: usize,
    pub(crate) min_tree_trunk_mesh_vertices: usize,
    pub(crate) min_tree_canopy_mesh_vertices: usize,
    pub(crate) detail_biome_palette_mask: u32,
    pub(crate) generated_rock_count: usize,
    pub(crate) min_rock_mesh_vertices: usize,
    pub(crate) generated_landmark_count: usize,
    pub(crate) generated_route_cairn_count: usize,
    pub(crate) generated_launch_beacon_count: usize,
    pub(crate) generated_landing_garden_marker_count: usize,
    pub(crate) generated_pond_surface_count: usize,
    pub(crate) min_landmark_mesh_vertices: usize,
    pub(crate) generated_weather_cloud_count: usize,
    pub(crate) generated_weather_cloud_bank_count: usize,
    pub(crate) min_weather_cloud_bank_depth_cm: usize,
    pub(crate) min_weather_cloud_lobe_count: usize,
    pub(crate) max_weather_cloud_lobe_count: usize,
    pub(crate) min_weather_cloud_mesh_vertices: usize,
    pub(crate) min_weather_cloud_filament_ribbon_detail_count: usize,
}

impl IslandContentDiagnostics {
    pub(crate) fn record_island_terrain_surface(
        &mut self,
        mesh_vertices: usize,
        color_bands: usize,
        material_weight_bands: usize,
        material_channels: usize,
        material_regions: usize,
        relief_range_m: f32,
    ) {
        let relief_range_cm = (relief_range_m.max(0.0) * 100.0).round() as usize;
        if self.island_terrain_surface_count == 0 {
            self.min_island_terrain_mesh_vertices = mesh_vertices;
            self.min_island_terrain_color_bands = color_bands;
            self.min_island_terrain_material_weight_bands = material_weight_bands;
            self.min_island_terrain_material_channels = material_channels;
            self.min_island_terrain_material_regions = material_regions;
            self.min_island_terrain_relief_range_cm = relief_range_cm;
        } else {
            self.min_island_terrain_mesh_vertices =
                self.min_island_terrain_mesh_vertices.min(mesh_vertices);
            self.min_island_terrain_color_bands =
                self.min_island_terrain_color_bands.min(color_bands);
            self.min_island_terrain_material_weight_bands = self
                .min_island_terrain_material_weight_bands
                .min(material_weight_bands);
            self.min_island_terrain_material_channels = self
                .min_island_terrain_material_channels
                .min(material_channels);
            self.min_island_terrain_material_regions = self
                .min_island_terrain_material_regions
                .min(material_regions);
            self.min_island_terrain_relief_range_cm =
                self.min_island_terrain_relief_range_cm.min(relief_range_cm);
        }
        self.island_terrain_surface_count += 1;
    }

    pub(crate) fn record_terrain_material_texture_detail(&mut self, detail_bands: usize) {
        if self.min_island_terrain_texture_detail_bands == 0 {
            self.min_island_terrain_texture_detail_bands = detail_bands;
        } else {
            self.min_island_terrain_texture_detail_bands = self
                .min_island_terrain_texture_detail_bands
                .min(detail_bands);
        }
    }

    pub(crate) fn record_island_cliff_detail(&mut self, color_bands: usize) {
        if self.min_island_cliff_color_bands == 0 {
            self.min_island_cliff_color_bands = color_bands;
        } else {
            self.min_island_cliff_color_bands = self.min_island_cliff_color_bands.min(color_bands);
        }
    }

    pub(crate) fn min_island_terrain_relief_range_m(self) -> f32 {
        self.min_island_terrain_relief_range_cm as f32 / 100.0
    }

    pub(crate) fn record_island_impostor(&mut self, mesh_vertices: usize, color_bands: usize) {
        if self.min_island_impostor_mesh_vertices == 0 {
            self.min_island_impostor_mesh_vertices = mesh_vertices;
            self.min_island_impostor_color_bands = color_bands;
        } else {
            self.min_island_impostor_mesh_vertices =
                self.min_island_impostor_mesh_vertices.min(mesh_vertices);
            self.min_island_impostor_color_bands =
                self.min_island_impostor_color_bands.min(color_bands);
        }
    }

    pub(crate) fn record_procedural_island_body(
        &mut self,
        silhouette_segments: usize,
        mesh_vertices: usize,
    ) {
        if self.procedural_island_body_count == 0 {
            self.min_island_body_silhouette_segments = silhouette_segments;
            self.min_island_body_mesh_vertices = mesh_vertices;
        } else {
            self.min_island_body_silhouette_segments = self
                .min_island_body_silhouette_segments
                .min(silhouette_segments);
            self.min_island_body_mesh_vertices =
                self.min_island_body_mesh_vertices.min(mesh_vertices);
        }
        self.procedural_island_body_count += 1;
        self.max_island_body_silhouette_segments = self
            .max_island_body_silhouette_segments
            .max(silhouette_segments);
        self.total_island_body_silhouette_segments += silhouette_segments;
        self.max_island_body_mesh_vertices = self.max_island_body_mesh_vertices.max(mesh_vertices);
    }

    pub(crate) fn average_island_body_silhouette_segments(self) -> f32 {
        if self.procedural_island_body_count == 0 {
            0.0
        } else {
            self.total_island_body_silhouette_segments as f32
                / self.procedural_island_body_count as f32
        }
    }

    pub(crate) fn record_generated_ground_cover(
        &mut self,
        patch_count: usize,
        blade_count: usize,
        mesh_vertices: usize,
    ) {
        if self.generated_ground_cover_patch_count == 0 {
            self.min_ground_cover_blade_count = blade_count;
            self.min_ground_cover_mesh_vertices = mesh_vertices;
        } else {
            self.min_ground_cover_blade_count = self.min_ground_cover_blade_count.min(blade_count);
            self.min_ground_cover_mesh_vertices =
                self.min_ground_cover_mesh_vertices.min(mesh_vertices);
        }
        self.generated_ground_cover_patch_count += patch_count;
    }

    pub(crate) fn record_generated_tree_trunk(&mut self, mesh_vertices: usize) {
        if self.generated_tree_trunk_count == 0 {
            self.min_tree_trunk_mesh_vertices = mesh_vertices;
        } else {
            self.min_tree_trunk_mesh_vertices =
                self.min_tree_trunk_mesh_vertices.min(mesh_vertices);
        }
        self.generated_tree_trunk_count += 1;
    }

    pub(crate) fn record_generated_tree_canopy(&mut self, mesh_vertices: usize) {
        if self.generated_tree_canopy_count == 0 {
            self.min_tree_canopy_mesh_vertices = mesh_vertices;
        } else {
            self.min_tree_canopy_mesh_vertices =
                self.min_tree_canopy_mesh_vertices.min(mesh_vertices);
        }
        self.generated_tree_canopy_count += 1;
    }

    pub(crate) fn record_detail_biome_palette(&mut self, palette_index: usize) {
        self.detail_biome_palette_mask |= 1_u32 << (palette_index % TERRAIN_BIOME_PALETTE_COUNT);
    }

    pub(crate) fn detail_biome_palette_count(self) -> usize {
        self.detail_biome_palette_mask.count_ones() as usize
    }

    pub(crate) fn record_generated_rock(&mut self, mesh_vertices: usize) {
        if self.generated_rock_count == 0 {
            self.min_rock_mesh_vertices = mesh_vertices;
        } else {
            self.min_rock_mesh_vertices = self.min_rock_mesh_vertices.min(mesh_vertices);
        }
        self.generated_rock_count += 1;
    }

    pub(crate) fn record_generated_landmark(
        &mut self,
        kind: GeneratedLandmarkKind,
        mesh_vertices: usize,
    ) {
        if self.generated_landmark_count == 0 {
            self.min_landmark_mesh_vertices = mesh_vertices;
        } else {
            self.min_landmark_mesh_vertices = self.min_landmark_mesh_vertices.min(mesh_vertices);
        }
        self.generated_landmark_count += 1;
        match kind {
            GeneratedLandmarkKind::RouteCairn => self.generated_route_cairn_count += 1,
            GeneratedLandmarkKind::LaunchBeacon => self.generated_launch_beacon_count += 1,
            GeneratedLandmarkKind::LandingGardenMarker => {
                self.generated_landing_garden_marker_count += 1;
            }
            GeneratedLandmarkKind::PondSurface => self.generated_pond_surface_count += 1,
        }
    }

    pub(crate) fn record_generated_weather_cloud(
        &mut self,
        lobe_count: usize,
        mesh_vertices: usize,
        filament_ribbon_detail_count: usize,
        depth_m: f32,
        is_bank: bool,
    ) {
        if self.generated_weather_cloud_count == 0 {
            self.min_weather_cloud_lobe_count = lobe_count;
            self.min_weather_cloud_mesh_vertices = mesh_vertices;
            self.min_weather_cloud_filament_ribbon_detail_count = filament_ribbon_detail_count;
        } else {
            self.min_weather_cloud_lobe_count = self.min_weather_cloud_lobe_count.min(lobe_count);
            self.min_weather_cloud_mesh_vertices =
                self.min_weather_cloud_mesh_vertices.min(mesh_vertices);
            self.min_weather_cloud_filament_ribbon_detail_count = self
                .min_weather_cloud_filament_ribbon_detail_count
                .min(filament_ribbon_detail_count);
        }
        self.generated_weather_cloud_count += 1;
        self.max_weather_cloud_lobe_count = self.max_weather_cloud_lobe_count.max(lobe_count);
        if is_bank {
            let depth_cm = (depth_m.max(0.0) * 100.0).round() as usize;
            if self.generated_weather_cloud_bank_count == 0 {
                self.min_weather_cloud_bank_depth_cm = depth_cm;
            } else {
                self.min_weather_cloud_bank_depth_cm =
                    self.min_weather_cloud_bank_depth_cm.min(depth_cm);
            }
            self.generated_weather_cloud_bank_count += 1;
        }
    }

    pub(crate) fn min_weather_cloud_bank_depth_m(self) -> f32 {
        self.min_weather_cloud_bank_depth_cm as f32 / 100.0
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum GeneratedLandmarkKind {
    RouteCairn,
    LaunchBeacon,
    LandingGardenMarker,
    PondSurface,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_content::ISLAND_BODY_SEGMENTS;

    #[test]
    fn content_diagnostics_tracks_procedural_body_complexity() {
        let mut diagnostics = IslandContentDiagnostics::default();

        diagnostics.record_procedural_island_body(ISLAND_BODY_SEGMENTS, 833);
        diagnostics.record_procedural_island_body(ISLAND_BODY_SEGMENTS, 821);
        diagnostics.record_island_terrain_surface(2305, 9, 16, 3, 4, 1.12);
        diagnostics.record_island_terrain_surface(2305, 7, 12, 3, 4, 0.92);
        diagnostics.record_terrain_material_texture_detail(72);
        diagnostics.record_terrain_material_texture_detail(64);
        diagnostics.record_island_cliff_detail(11);
        diagnostics.record_island_cliff_detail(10);
        diagnostics.record_island_impostor(146, 22);
        diagnostics.record_island_impostor(144, 19);

        assert_eq!(diagnostics.procedural_island_body_count, 2);
        assert_eq!(diagnostics.island_terrain_surface_count, 2);
        assert_eq!(diagnostics.min_island_terrain_mesh_vertices, 2305);
        assert_eq!(diagnostics.min_island_terrain_color_bands, 7);
        assert_eq!(diagnostics.min_island_terrain_material_weight_bands, 12);
        assert_eq!(diagnostics.min_island_terrain_material_channels, 3);
        assert_eq!(diagnostics.min_island_terrain_material_regions, 4);
        assert_eq!(diagnostics.min_island_terrain_texture_detail_bands, 64);
        assert_eq!(diagnostics.min_island_terrain_relief_range_m(), 0.92);
        assert_eq!(diagnostics.min_island_cliff_color_bands, 10);
        assert_eq!(diagnostics.min_island_impostor_mesh_vertices, 144);
        assert_eq!(diagnostics.min_island_impostor_color_bands, 19);
        assert_eq!(diagnostics.primitive_island_body_count, 0);
        assert_eq!(
            diagnostics.min_island_body_silhouette_segments,
            ISLAND_BODY_SEGMENTS
        );
        assert_eq!(
            diagnostics.average_island_body_silhouette_segments(),
            ISLAND_BODY_SEGMENTS as f32
        );
        assert_eq!(diagnostics.min_island_body_mesh_vertices, 821);
        assert_eq!(diagnostics.max_island_body_mesh_vertices, 833);
    }

    #[test]
    fn content_diagnostics_tracks_generated_tree_and_cloud_complexity() {
        let mut diagnostics = IslandContentDiagnostics::default();

        diagnostics.record_detail_biome_palette(0);
        diagnostics.record_detail_biome_palette(2);
        diagnostics.record_detail_biome_palette(2);
        diagnostics.record_generated_ground_cover(44, 220, 1100);
        diagnostics.record_generated_ground_cover(44, 220, 1100);
        diagnostics.record_generated_tree_trunk(26);
        diagnostics.record_generated_tree_trunk(30);
        diagnostics.record_generated_tree_canopy(226);
        diagnostics.record_generated_tree_canopy(240);
        diagnostics.record_generated_rock(74);
        diagnostics.record_generated_rock(80);
        diagnostics.record_generated_landmark(GeneratedLandmarkKind::RouteCairn, 250);
        diagnostics.record_generated_landmark(GeneratedLandmarkKind::LaunchBeacon, 306);
        diagnostics.record_generated_landmark(GeneratedLandmarkKind::LandingGardenMarker, 39);
        diagnostics.record_generated_landmark(GeneratedLandmarkKind::PondSurface, 65);
        diagnostics.record_generated_weather_cloud(7, 315, 14, 4.2, true);
        diagnostics.record_generated_weather_cloud(4, 180, 8, 0.8, false);

        assert_eq!(diagnostics.generated_ground_cover_patch_count, 88);
        assert_eq!(diagnostics.min_ground_cover_blade_count, 220);
        assert_eq!(diagnostics.min_ground_cover_mesh_vertices, 1100);
        assert_eq!(diagnostics.generated_tree_trunk_count, 2);
        assert_eq!(diagnostics.generated_tree_canopy_count, 2);
        assert_eq!(diagnostics.min_tree_trunk_mesh_vertices, 26);
        assert_eq!(diagnostics.min_tree_canopy_mesh_vertices, 226);
        assert_eq!(diagnostics.detail_biome_palette_count(), 2);
        assert_eq!(diagnostics.generated_rock_count, 2);
        assert_eq!(diagnostics.min_rock_mesh_vertices, 74);
        assert_eq!(diagnostics.generated_landmark_count, 4);
        assert_eq!(diagnostics.generated_route_cairn_count, 1);
        assert_eq!(diagnostics.generated_launch_beacon_count, 1);
        assert_eq!(diagnostics.generated_landing_garden_marker_count, 1);
        assert_eq!(diagnostics.generated_pond_surface_count, 1);
        assert_eq!(diagnostics.min_landmark_mesh_vertices, 39);
        assert_eq!(diagnostics.generated_weather_cloud_count, 2);
        assert_eq!(diagnostics.generated_weather_cloud_bank_count, 1);
        assert_eq!(diagnostics.min_weather_cloud_bank_depth_m(), 4.2);
        assert_eq!(diagnostics.min_weather_cloud_lobe_count, 4);
        assert_eq!(diagnostics.max_weather_cloud_lobe_count, 7);
        assert_eq!(diagnostics.min_weather_cloud_mesh_vertices, 180);
        assert_eq!(
            diagnostics.min_weather_cloud_filament_ribbon_detail_count,
            8
        );
    }
}
