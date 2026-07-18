use super::{
    super::random_unit,
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::{
    IslandLandmarkRole, IslandPlateauRegion, IslandRouteRole, IslandTerrainArchetype,
    IslandWaterFeature, IslandWaterStory, ROUTE_EDGE_WATERFALL_CHANNEL_OUTLET_OFFSET, SkyIsland,
    authored_island_art_direction, route_edge_waterfall_placement,
};

pub(crate) const ROUTE_CAIRN_STONE_COUNT: usize = 5;
pub(crate) const RUIN_ARCH_STONE_COUNT: usize = 11;
pub(crate) const LAUNCH_BEACON_CRYSTAL_COUNT: usize = 4;
pub(crate) const LANDING_GARDEN_MARKER_SEGMENTS: usize = 12;
pub(crate) const GARDEN_RING_SEGMENTS: usize = 36;
pub(crate) const GARDEN_RING_BANDS: usize = 4;
pub(crate) const LAKE_BASIN_RIM_SEGMENTS: usize = 48;
pub(crate) const LAKE_BASIN_RIM_BANDS: usize = 5;
pub(crate) const POND_SURFACE_SEGMENTS: usize = 72;
pub(crate) const LAKE_SURFACE_SEGMENTS: usize = 96;
pub(crate) const RIVER_CHANNEL_SEGMENTS: usize = 32;
pub(crate) const RIVER_CHANNEL_COLUMNS: usize = 7;
pub(crate) const WATERFALL_RIBBON_COLUMNS: usize = 13;
pub(crate) const WATERFALL_RIBBON_ROWS: usize = 28;
pub(crate) const WATERFALL_MIST_LOBES: usize = 7;
pub(crate) const ARTIFACT_STAIR_STEP_COUNT: usize = 9;
pub(crate) const ARTIFACT_RETAINING_WALL_SEGMENTS: usize = 8;
pub(crate) const ARTIFACT_GLYPH_STROKE_COUNT: usize = 7;
pub(crate) const ARTIFACT_BRIDGE_FRAGMENT_COUNT: usize = 9;
pub(crate) const ARTIFACT_BANNER_STRIP_COUNT: usize = 6;
pub(crate) const ARTIFACT_PEBBLE_COUNT: usize = 18;
pub(crate) const ARTIFACT_REED_COUNT: usize = 16;

const LANDMARK_LOBE_LATITUDE_SEGMENTS: usize = 4;
const LANDMARK_LOBE_LONGITUDE_SEGMENTS: usize = 9;
const CRYSTAL_RING_SEGMENTS: usize = 6;
const HORIZONTAL_WATER_TERRAIN_CLEARANCE_M: f32 = 0.14;
const WATER_FOOTPRINT_GEOMETRY_PADDING_M: f32 = 0.05;
const POND_SURFACE_RINGS: [f32; 2] = [0.48, 1.0];
const LAKE_SURFACE_RINGS: [f32; 3] = [0.34, 0.68, 1.0];

#[derive(Clone, Copy, Debug)]
struct WaterSurfaceVertex {
    position: Vec3,
    uv: [f32; 2],
    foam_mask: f32,
    flow_class: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IslandWaterVisualKind {
    PondSurface,
    PlateauLakeSurface,
    PlateauWaterfallRibbon,
    PlateauWaterfallMist,
    RouteWaterfallRibbon,
    RouteWaterfallMist,
    RouteLakeSurface,
    RiverChannel,
}

impl IslandWaterVisualKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PondSurface => "pond_surface",
            Self::PlateauLakeSurface => "plateau_lake_surface",
            Self::PlateauWaterfallRibbon => "plateau_waterfall_ribbon",
            Self::PlateauWaterfallMist => "plateau_waterfall_mist",
            Self::RouteWaterfallRibbon => "route_waterfall_ribbon",
            Self::RouteWaterfallMist => "route_waterfall_mist",
            Self::RouteLakeSurface => "route_lake_surface",
            Self::RiverChannel => "river_channel",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::PondSurface => "island pond",
            Self::PlateauLakeSurface => "plateau lake",
            Self::PlateauWaterfallRibbon => "plateau waterfall ribbon",
            Self::PlateauWaterfallMist => "plateau waterfall mist",
            Self::RouteWaterfallRibbon => "route waterfall ribbon",
            Self::RouteWaterfallMist => "route waterfall mist",
            Self::RouteLakeSurface => "route lake",
            Self::RiverChannel => "river channel",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IslandWaterVisualMesh {
    PondSurface {
        radius_x: f32,
        radius_z: f32,
    },
    LakeSurface {
        radius_x: f32,
        radius_z: f32,
    },
    RiverChannel {
        length: f32,
        width: f32,
        elevation_drop: f32,
    },
    WaterfallRibbon {
        width: f32,
        height: f32,
        depth: f32,
    },
    WaterfallMist {
        radius: f32,
        height: f32,
    },
}

#[derive(Clone, Copy, Debug)]
enum IslandWaterFootprintShape {
    Ellipse { radius_x: f32, radius_z: f32 },
    Channel { half_length: f32, half_width: f32 },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandWaterFootprint {
    center: Vec2,
    rotation_y: f32,
    shape: IslandWaterFootprintShape,
}

impl IslandWaterFootprint {
    pub(crate) fn contains_world_xz(self, world_xz: Vec2, margin_m: f32) -> bool {
        let delta = world_xz - self.center;
        let local = Quat::from_rotation_y(-self.rotation_y) * Vec3::new(delta.x, 0.0, delta.y);
        let margin_m = margin_m.max(0.0);

        match self.shape {
            IslandWaterFootprintShape::Ellipse { radius_x, radius_z } => {
                let normalized_x = local.x / (radius_x + margin_m).max(0.001);
                let normalized_z = local.z / (radius_z + margin_m).max(0.001);
                normalized_x * normalized_x + normalized_z * normalized_z <= 1.0
            }
            IslandWaterFootprintShape::Channel {
                half_length,
                half_width,
            } => local.x.abs() <= half_width + margin_m && local.z.abs() <= half_length + margin_m,
        }
    }
}

impl IslandWaterVisualMesh {
    pub(crate) fn build(self, seed: u32) -> Mesh {
        match self {
            Self::PondSurface { radius_x, radius_z } => pond_surface_mesh(radius_x, radius_z, seed),
            Self::LakeSurface { radius_x, radius_z } => lake_surface_mesh(radius_x, radius_z, seed),
            Self::RiverChannel {
                length,
                width,
                elevation_drop,
            } => river_channel_surface_mesh(length, width, elevation_drop, seed),
            Self::WaterfallRibbon {
                width,
                height,
                depth,
            } => waterfall_ribbon_mesh(width, height, depth, seed),
            Self::WaterfallMist { radius, height } => waterfall_mist_mesh(radius, height, seed),
        }
    }

    fn horizontal_footprint_shape(self, seed: u32) -> Option<IslandWaterFootprintShape> {
        let mesh = self;
        match self {
            Self::PondSurface { radius_x, radius_z } | Self::LakeSurface { radius_x, radius_z } => {
                let mut radial_scale = 1.0_f32;
                mesh.visit_horizontal_surface_positions(seed, |position| {
                    let normalized = Vec2::new(position.x / radius_x, position.z / radius_z);
                    radial_scale = radial_scale.max(normalized.length());
                });
                Some(IslandWaterFootprintShape::Ellipse {
                    radius_x: radius_x * radial_scale + WATER_FOOTPRINT_GEOMETRY_PADDING_M,
                    radius_z: radius_z * radial_scale + WATER_FOOTPRINT_GEOMETRY_PADDING_M,
                })
            }
            Self::RiverChannel { .. } => {
                let mut half_length = 0.0_f32;
                let mut half_width = 0.0_f32;
                mesh.visit_horizontal_surface_positions(seed, |position| {
                    half_length = half_length.max(position.z.abs());
                    half_width = half_width.max(position.x.abs());
                });
                Some(IslandWaterFootprintShape::Channel {
                    half_length: half_length + WATER_FOOTPRINT_GEOMETRY_PADDING_M,
                    half_width: half_width + WATER_FOOTPRINT_GEOMETRY_PADDING_M,
                })
            }
            Self::WaterfallRibbon { .. } | Self::WaterfallMist { .. } => None,
        }
    }

    fn semantic_sample_indices(self) -> Option<[usize; 5]> {
        match self {
            Self::PondSurface { .. } => Some([
                0,
                1,
                1 + POND_SURFACE_SEGMENTS / 4,
                1 + POND_SURFACE_SEGMENTS / 2,
                1 + POND_SURFACE_SEGMENTS * 3 / 4,
            ]),
            Self::LakeSurface { .. } => Some([
                0,
                1,
                1 + LAKE_SURFACE_SEGMENTS / 4,
                1 + LAKE_SURFACE_SEGMENTS / 2,
                1 + LAKE_SURFACE_SEGMENTS * 3 / 4,
            ]),
            Self::RiverChannel { .. } => {
                let center_column = RIVER_CHANNEL_COLUMNS / 2;
                Some([
                    RIVER_CHANNEL_SEGMENTS * 2 / 18 * RIVER_CHANNEL_COLUMNS + center_column,
                    RIVER_CHANNEL_SEGMENTS * 5 / 18 * RIVER_CHANNEL_COLUMNS + center_column,
                    RIVER_CHANNEL_SEGMENTS / 2 * RIVER_CHANNEL_COLUMNS + center_column,
                    RIVER_CHANNEL_SEGMENTS * 13 / 18 * RIVER_CHANNEL_COLUMNS + center_column,
                    RIVER_CHANNEL_SEGMENTS * 16 / 18 * RIVER_CHANNEL_COLUMNS + center_column,
                ])
            }
            Self::WaterfallRibbon { .. } | Self::WaterfallMist { .. } => None,
        }
    }

    fn visit_horizontal_surface_positions(self, seed: u32, visitor: impl FnMut(Vec3)) {
        match self {
            Self::PondSurface { radius_x, radius_z } => visit_irregular_water_surface_positions(
                radius_x,
                radius_z,
                seed,
                POND_SURFACE_SEGMENTS,
                &POND_SURFACE_RINGS,
                0.15,
                0.040,
                visitor,
            ),
            Self::LakeSurface { radius_x, radius_z } => visit_irregular_water_surface_positions(
                radius_x,
                radius_z,
                seed,
                LAKE_SURFACE_SEGMENTS,
                &LAKE_SURFACE_RINGS,
                0.20,
                0.085,
                visitor,
            ),
            Self::RiverChannel {
                length,
                width,
                elevation_drop,
            } => {
                visit_river_channel_surface_positions(length, width, elevation_drop, seed, visitor)
            }
            Self::WaterfallRibbon { .. } | Self::WaterfallMist { .. } => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandWaterVisualSpec {
    pub(crate) kind: IslandWaterVisualKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) wind_phase: f32,
    pub(crate) wind_motion_scale: f32,
    pub(crate) mesh: IslandWaterVisualMesh,
    seed: u32,
}

impl IslandWaterVisualSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        self.mesh.build(self.seed)
    }

    pub(crate) fn horizontal_footprint(self) -> Option<IslandWaterFootprint> {
        Some(IslandWaterFootprint {
            center: self.translation.xz(),
            rotation_y: self.rotation_y,
            shape: self.mesh.horizontal_footprint_shape(self.seed)?,
        })
    }

    pub(crate) fn semantic_surface_sample_positions(self) -> Vec<Vec3> {
        let Some(indices) = self.mesh.semantic_sample_indices() else {
            return vec![self.translation];
        };
        let rotation = Quat::from_rotation_y(self.rotation_y);
        let mut samples = Vec::with_capacity(indices.len());
        let mut vertex_index = 0;
        self.mesh
            .visit_horizontal_surface_positions(self.seed, |position| {
                if indices.contains(&vertex_index) {
                    samples.push(self.translation + rotation * position);
                }
                vertex_index += 1;
            });
        samples
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandLakeBasinVisualSpec {
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) radius_x: f32,
    pub(crate) radius_z: f32,
    pub(crate) rim_width: f32,
    pub(crate) rim_height: f32,
    seed: u32,
}

impl IslandLakeBasinVisualSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        lake_basin_rim_mesh(
            self.radius_x,
            self.radius_z,
            self.rim_width,
            self.rim_height,
            self.seed,
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct PlateauWaterfallFeatureSpec {
    region: IslandPlateauRegion,
    ribbon_label: &'static str,
    mist_label: &'static str,
    width_scale: f32,
    index: u32,
}

#[derive(Clone, Copy, Debug)]
struct RiverChannelFeatureSpec {
    label: &'static str,
    source_offset: Vec2,
    outlet_offset: Vec2,
    width_scale: f32,
    index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FirstExpeditionSilhouetteKind {
    NorthRuinSpire,
    SouthRuinSpire,
    WaterfallCliff,
    CaveArch,
    RingGarden,
    BrokenStair,
    HighCrown,
}

impl FirstExpeditionSilhouetteKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::NorthRuinSpire => "first_expedition_north_ruin_spire",
            Self::SouthRuinSpire => "first_expedition_south_ruin_spire",
            Self::WaterfallCliff => "first_expedition_waterfall_cliff",
            Self::CaveArch => "first_expedition_cave_arch",
            Self::RingGarden => "first_expedition_ring_garden",
            Self::BrokenStair => "first_expedition_broken_stair",
            Self::HighCrown => "first_expedition_high_crown",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::NorthRuinSpire => "first expedition north ruin spire",
            Self::SouthRuinSpire => "first expedition south ruin spire",
            Self::WaterfallCliff => "first expedition waterfall cliff",
            Self::CaveArch => "first expedition cave arch",
            Self::RingGarden => "first expedition ring garden",
            Self::BrokenStair => "first expedition broken stair",
            Self::HighCrown => "first expedition high crown",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FirstExpeditionSilhouetteSpec {
    pub(crate) kind: FirstExpeditionSilhouetteKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    mesh: FirstExpeditionSilhouetteMesh,
    seed: u32,
}

impl FirstExpeditionSilhouetteSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        match self.mesh {
            FirstExpeditionSilhouetteMesh::Cairn { radius, height } => {
                route_cairn_mesh(radius, height, self.seed)
            }
            FirstExpeditionSilhouetteMesh::RuinArch {
                width,
                height,
                depth,
            } => ruin_arch_mesh(width, height, depth, self.seed),
            FirstExpeditionSilhouetteMesh::GardenRing {
                radius,
                band_width,
                height,
            } => garden_ring_mesh(radius, band_width, height, self.seed),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum FirstExpeditionSilhouetteMesh {
    Cairn {
        radius: f32,
        height: f32,
    },
    RuinArch {
        width: f32,
        height: f32,
        depth: f32,
    },
    GardenRing {
        radius: f32,
        band_width: f32,
        height: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum IslandArtifactVisualKind {
    AncientStairRun,
    RetainingWall,
    GlyphSlab,
    BridgeFragment,
    BannerStrips,
    PebbleField,
    ReedPatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IslandArtifactCollisionPolicy {
    SolidAabb,
    NonBlockingRouteAffordance,
    NonBlockingDecoration,
}

impl IslandArtifactVisualKind {
    #[cfg(test)]
    const ALL: [Self; 7] = [
        Self::AncientStairRun,
        Self::RetainingWall,
        Self::GlyphSlab,
        Self::BridgeFragment,
        Self::BannerStrips,
        Self::PebbleField,
        Self::ReedPatch,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::AncientStairRun => "artifact_ancient_stair",
            Self::RetainingWall => "artifact_retaining_wall",
            Self::GlyphSlab => "artifact_glyph_slab",
            Self::BridgeFragment => "artifact_bridge_fragment",
            Self::BannerStrips => "artifact_banner_strips",
            Self::PebbleField => "artifact_pebble_field",
            Self::ReedPatch => "artifact_reed_patch",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::AncientStairRun => "ancient stair run",
            Self::RetainingWall => "retaining wall fragment",
            Self::GlyphSlab => "glyph stone slab",
            Self::BridgeFragment => "broken bridge fragment",
            Self::BannerStrips => "weathered banner strips",
            Self::PebbleField => "pebble field",
            Self::ReedPatch => "reed patch",
        }
    }

    fn collision_policy(self) -> IslandArtifactCollisionPolicy {
        match self {
            Self::RetainingWall | Self::GlyphSlab => IslandArtifactCollisionPolicy::SolidAabb,
            Self::AncientStairRun | Self::BridgeFragment => {
                IslandArtifactCollisionPolicy::NonBlockingRouteAffordance
            }
            Self::BannerStrips | Self::PebbleField | Self::ReedPatch => {
                IslandArtifactCollisionPolicy::NonBlockingDecoration
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IslandArtifactMaterial {
    Stone,
    Foliage,
    Trunk,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandArtifactVisualSpec {
    pub(crate) kind: IslandArtifactVisualKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) material: IslandArtifactMaterial,
    mesh: IslandArtifactVisualMesh,
    seed: u32,
}

impl IslandArtifactVisualSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        match self.mesh {
            IslandArtifactVisualMesh::AncientStairRun {
                length,
                width,
                rise,
            } => artifact_stair_run_mesh(length, width, rise, self.seed),
            IslandArtifactVisualMesh::RetainingWall {
                length,
                height,
                depth,
            } => artifact_retaining_wall_mesh(length, height, depth, self.seed),
            IslandArtifactVisualMesh::GlyphSlab {
                width,
                height,
                depth,
            } => artifact_glyph_slab_mesh(width, height, depth, self.seed),
            IslandArtifactVisualMesh::BridgeFragment {
                length,
                width,
                thickness,
            } => artifact_bridge_fragment_mesh(length, width, thickness, self.seed),
            IslandArtifactVisualMesh::BannerStrips { width, height } => {
                artifact_banner_strips_mesh(width, height, self.seed)
            }
            IslandArtifactVisualMesh::PebbleField { radius } => {
                artifact_pebble_field_mesh(radius, self.seed)
            }
            IslandArtifactVisualMesh::ReedPatch { radius, height } => {
                artifact_reed_patch_mesh(radius, height, self.seed)
            }
        }
    }

    pub(crate) fn solid_world_aabb(self) -> Option<(Vec3, Vec3)> {
        let local_aabb = match self.mesh {
            IslandArtifactVisualMesh::AncientStairRun { .. } => None,
            IslandArtifactVisualMesh::RetainingWall {
                length,
                height,
                depth,
            } => Some((
                Vec3::new(0.0, height * 0.5, 0.0),
                Vec3::new(length * 0.48, height * 0.5, depth * 0.48),
            )),
            IslandArtifactVisualMesh::GlyphSlab {
                width,
                height,
                depth,
            } => Some((
                Vec3::new(0.0, height * 0.5, 0.0),
                Vec3::new(width * 0.48, height * 0.5, depth * 0.48),
            )),
            IslandArtifactVisualMesh::BridgeFragment { .. }
            | IslandArtifactVisualMesh::BannerStrips { .. }
            | IslandArtifactVisualMesh::PebbleField { .. }
            | IslandArtifactVisualMesh::ReedPatch { .. } => None,
        };
        debug_assert_eq!(
            local_aabb.is_some(),
            self.kind.collision_policy() == IslandArtifactCollisionPolicy::SolidAabb,
            "{} collision policy must match its mesh family",
            self.kind.visual_name()
        );

        local_aabb.map(|(local_center, local_half_extents)| {
            let rotation = Quat::from_rotation_y(self.rotation_y);
            let center = self.translation + rotation * local_center;
            let yaw_sin = self.rotation_y.sin().abs();
            let yaw_cos = self.rotation_y.cos().abs();
            let half_extents = Vec3::new(
                local_half_extents.x * yaw_cos + local_half_extents.z * yaw_sin,
                local_half_extents.y,
                local_half_extents.x * yaw_sin + local_half_extents.z * yaw_cos,
            );
            (center, half_extents)
        })
    }
}

#[derive(Clone, Copy, Debug)]
enum IslandArtifactVisualMesh {
    AncientStairRun {
        length: f32,
        width: f32,
        rise: f32,
    },
    RetainingWall {
        length: f32,
        height: f32,
        depth: f32,
    },
    GlyphSlab {
        width: f32,
        height: f32,
        depth: f32,
    },
    BridgeFragment {
        length: f32,
        width: f32,
        thickness: f32,
    },
    BannerStrips {
        width: f32,
        height: f32,
    },
    PebbleField {
        radius: f32,
    },
    ReedPatch {
        radius: f32,
        height: f32,
    },
}

pub(crate) fn first_expedition_silhouette_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<FirstExpeditionSilhouetteSpec> {
    let seed = 51_000 + island_index as u32 * 269;
    match island.name {
        "mist arch" => {
            let height = 16.0;
            vec![FirstExpeditionSilhouetteSpec {
                kind: FirstExpeditionSilhouetteKind::NorthRuinSpire,
                label: "north ruin spire",
                translation: island_water_surface_position(island, Vec2::new(-0.24, 0.18))
                    + Vec3::Y * (height * 0.5),
                rotation_y: -0.34,
                mesh: FirstExpeditionSilhouetteMesh::Cairn {
                    radius: 1.05,
                    height,
                },
                seed,
            }]
        }
        "cloud gate" => {
            let height = 15.5;
            vec![FirstExpeditionSilhouetteSpec {
                kind: FirstExpeditionSilhouetteKind::SouthRuinSpire,
                label: "south ruin spire",
                translation: island_water_surface_position(island, Vec2::new(0.22, -0.16))
                    + Vec3::Y * (height * 0.5),
                rotation_y: 0.42,
                mesh: FirstExpeditionSilhouetteMesh::Cairn {
                    radius: 1.0,
                    height,
                },
                seed,
            }]
        }
        "cloudfall meadow" => {
            let height = 14.0;
            vec![FirstExpeditionSilhouetteSpec {
                kind: FirstExpeditionSilhouetteKind::WaterfallCliff,
                label: "waterfall cliff silhouette",
                translation: island_water_surface_position(island, Vec2::new(0.54, 0.10))
                    + Vec3::Y * (height * 0.44),
                rotation_y: -0.18,
                mesh: FirstExpeditionSilhouetteMesh::RuinArch {
                    width: 18.0,
                    height,
                    depth: 5.2,
                },
                seed,
            }]
        }
        "underbridge cay" => island
            .under_route_segment()
            .map(|segment| {
                let height = 8.2;
                vec![FirstExpeditionSilhouetteSpec {
                    kind: FirstExpeditionSilhouetteKind::CaveArch,
                    label: "cave mouth silhouette",
                    translation: segment.entry + Vec3::Y * (height * 0.34),
                    rotation_y: -0.62,
                    mesh: FirstExpeditionSilhouetteMesh::RuinArch {
                        width: 10.5,
                        height,
                        depth: 2.7,
                    },
                    seed,
                }]
            })
            .unwrap_or_default(),
        "sunspire garden" => vec![FirstExpeditionSilhouetteSpec {
            kind: FirstExpeditionSilhouetteKind::RingGarden,
            label: "ring garden silhouette",
            translation: island_water_surface_position(island, Vec2::ZERO) + Vec3::Y * 0.12,
            rotation_y: 0.18,
            mesh: FirstExpeditionSilhouetteMesh::GardenRing {
                radius: 11.0,
                band_width: 2.4,
                height: 0.62,
            },
            seed,
        }],
        "broken stair" => {
            let height = 12.0;
            vec![FirstExpeditionSilhouetteSpec {
                kind: FirstExpeditionSilhouetteKind::BrokenStair,
                label: "broken stair silhouette",
                translation: island_water_surface_position(island, Vec2::new(0.18, 0.22))
                    + Vec3::Y * (height * 0.44),
                rotation_y: 0.72,
                mesh: FirstExpeditionSilhouetteMesh::RuinArch {
                    width: 15.0,
                    height,
                    depth: 4.6,
                },
                seed,
            }]
        }
        "upper crown" => {
            let height = 21.0;
            vec![FirstExpeditionSilhouetteSpec {
                kind: FirstExpeditionSilhouetteKind::HighCrown,
                label: "high crown silhouette",
                translation: island_water_surface_position(island, Vec2::ZERO)
                    + Vec3::Y * (height * 0.5),
                rotation_y: -0.25,
                mesh: FirstExpeditionSilhouetteMesh::Cairn {
                    radius: 1.45,
                    height,
                },
                seed,
            }]
        }
        _ => Vec::new(),
    }
}

pub(crate) fn island_artifact_visual_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandArtifactVisualSpec> {
    if island.is_great_plateau_anchor() {
        return great_plateau_artifact_visual_specs(island_index, island);
    }

    let mut specs = Vec::new();
    let scale = island.half_extents.min_element();
    let base_seed = 61_000 + island_index as u32 * 283;

    if artifact_has_stair_run(island) {
        specs.push(IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::AncientStairRun,
            label: "ancient stair run",
            translation: island_water_surface_position(island, Vec2::new(-0.18, -0.26))
                + Vec3::Y * 0.04,
            rotation_y: -0.34 + island_index as f32 * 0.027,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::AncientStairRun {
                length: (scale * 0.30).clamp(5.8, 18.0),
                width: (scale * 0.105).clamp(1.9, 5.6),
                rise: (island.thickness * 0.085).clamp(1.4, 5.2),
            },
            seed: base_seed + 1,
        });
    }

    if artifact_has_ruin_stonework(island) {
        specs.push(IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::RetainingWall,
            label: "retaining wall fragment",
            translation: island_water_surface_position(island, Vec2::new(0.28, -0.18))
                + Vec3::Y * 0.03,
            rotation_y: 0.72 + island_index as f32 * 0.019,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::RetainingWall {
                length: (scale * 0.24).clamp(5.0, 16.0),
                height: (island.thickness * 0.080).clamp(1.2, 4.8),
                depth: (scale * 0.050).clamp(0.8, 2.6),
            },
            seed: base_seed + 11,
        });
        specs.push(IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::GlyphSlab,
            label: "glyph stone slab",
            translation: island_water_surface_position(island, Vec2::new(-0.30, 0.18))
                + Vec3::Y * 0.05,
            rotation_y: -0.94 + island_index as f32 * 0.031,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::GlyphSlab {
                width: (scale * 0.080).clamp(1.4, 4.4),
                height: (island.thickness * 0.13).clamp(2.0, 7.0),
                depth: (scale * 0.035).clamp(0.45, 1.6),
            },
            seed: base_seed + 23,
        });
    }

    if artifact_has_bridge_fragment(island) {
        specs.push(IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::BridgeFragment,
            label: "broken bridge fragment",
            translation: island_water_surface_position(island, Vec2::new(0.08, 0.42))
                + Vec3::Y * 0.06,
            rotation_y: 1.08 + island_index as f32 * 0.021,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::BridgeFragment {
                length: (scale * 0.34).clamp(6.0, 20.0),
                width: (scale * 0.095).clamp(1.8, 5.2),
                thickness: (island.thickness * 0.030).clamp(0.38, 1.2),
            },
            seed: base_seed + 37,
        });
    }

    if artifact_has_wind_cloth(island) {
        specs.push(IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::BannerStrips,
            label: "weathered banner strips",
            translation: island_water_surface_position(island, Vec2::new(0.34, 0.12))
                + Vec3::Y * (island.thickness * 0.075).clamp(1.4, 4.5),
            rotation_y: 0.18 + island_index as f32 * 0.044,
            material: IslandArtifactMaterial::Trunk,
            mesh: IslandArtifactVisualMesh::BannerStrips {
                width: (scale * 0.12).clamp(2.0, 6.4),
                height: (island.thickness * 0.10).clamp(2.4, 5.8),
            },
            seed: base_seed + 53,
        });
    }

    if artifact_has_reeds(island) {
        specs.push(IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::ReedPatch,
            label: "reed patch",
            translation: island_water_surface_position(island, Vec2::new(-0.12, 0.22))
                + Vec3::Y * 0.06,
            rotation_y: island_index as f32 * 0.058,
            material: IslandArtifactMaterial::Foliage,
            mesh: IslandArtifactVisualMesh::ReedPatch {
                radius: (scale * 0.060).clamp(1.0, 3.8),
                height: (island.thickness * 0.055).clamp(0.9, 3.0),
            },
            seed: base_seed + 71,
        });
    }

    if artifact_has_pebble_field(island) {
        specs.push(IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::PebbleField,
            label: "pebble field",
            translation: island_water_surface_position(island, Vec2::new(0.18, -0.38))
                + Vec3::Y * 0.025,
            rotation_y: island_index as f32 * 0.073,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::PebbleField {
                radius: (scale * 0.090).clamp(1.3, 5.0),
            },
            seed: base_seed + 89,
        });
    }

    specs
}

fn great_plateau_artifact_visual_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandArtifactVisualSpec> {
    let base_seed = 61_000 + island_index as u32 * 283;
    vec![
        IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::AncientStairRun,
            label: "ancient stair run",
            translation: island_water_surface_position(island, Vec2::new(-0.40, 0.28))
                + Vec3::Y * 0.06,
            rotation_y: -0.88,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::AncientStairRun {
                length: 28.0,
                width: 7.2,
                rise: 6.4,
            },
            seed: base_seed + 1,
        },
        IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::RetainingWall,
            label: "retaining wall fragment",
            translation: island_water_surface_position(island, Vec2::new(-0.58, 0.12))
                + Vec3::Y * 0.05,
            rotation_y: 0.18,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::RetainingWall {
                length: 34.0,
                height: 7.2,
                depth: 3.4,
            },
            seed: base_seed + 11,
        },
        IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::GlyphSlab,
            label: "glyph stone slab",
            translation: island_water_surface_position(island, Vec2::new(-0.34, 0.36))
                + Vec3::Y * 0.08,
            rotation_y: -0.64,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::GlyphSlab {
                width: 5.4,
                height: 8.2,
                depth: 1.8,
            },
            seed: base_seed + 23,
        },
        IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::BridgeFragment,
            label: "broken bridge fragment",
            translation: island_water_surface_position(island, Vec2::new(0.48, 0.12))
                + Vec3::Y * 0.08,
            rotation_y: -0.52,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::BridgeFragment {
                length: 27.0,
                width: 7.0,
                thickness: 1.5,
            },
            seed: base_seed + 37,
        },
        IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::ReedPatch,
            label: "reed patch",
            translation: island_water_surface_position(island, Vec2::new(0.18, -0.34))
                + Vec3::Y * 0.07,
            rotation_y: 0.22,
            material: IslandArtifactMaterial::Foliage,
            mesh: IslandArtifactVisualMesh::ReedPatch {
                radius: 7.2,
                height: 4.2,
            },
            seed: base_seed + 71,
        },
        IslandArtifactVisualSpec {
            kind: IslandArtifactVisualKind::PebbleField,
            label: "pebble field",
            translation: island_water_surface_position(island, Vec2::new(0.44, -0.44))
                + Vec3::Y * 0.035,
            rotation_y: -0.28,
            material: IslandArtifactMaterial::Stone,
            mesh: IslandArtifactVisualMesh::PebbleField { radius: 9.0 },
            seed: base_seed + 89,
        },
    ]
}

fn artifact_has_stair_run(island: SkyIsland) -> bool {
    matches!(
        island.terrain_archetype,
        IslandTerrainArchetype::TerracedSpur
            | IslandTerrainArchetype::BrokenStair
            | IslandTerrainArchetype::SkyPlateau
            | IslandTerrainArchetype::LaunchMesa
    )
}

fn artifact_has_ruin_stonework(island: SkyIsland) -> bool {
    matches!(
        island.world_tags.landmark_role,
        IslandLandmarkRole::RuinArch
            | IslandLandmarkRole::CaveMouth
            | IslandLandmarkRole::HighCrown
    ) || island.is_great_plateau_anchor()
}

fn artifact_has_bridge_fragment(island: SkyIsland) -> bool {
    matches!(
        island.terrain_archetype,
        IslandTerrainArchetype::MistArch
            | IslandTerrainArchetype::CloudGate
            | IslandTerrainArchetype::MistStep
            | IslandTerrainArchetype::BrokenStair
    )
}

fn artifact_has_wind_cloth(island: SkyIsland) -> bool {
    matches!(
        island.world_tags.landmark_role,
        IslandLandmarkRole::WindGate | IslandLandmarkRole::HighCrown
    ) || island.world_tags.route_role == IslandRouteRole::UpdraftHub
}

fn artifact_has_reeds(island: SkyIsland) -> bool {
    if let Some(art) = authored_island_art_direction(island.name) {
        return matches!(
            art.water_story,
            IslandWaterStory::SpringPond
                | IslandWaterStory::ReflectingBasin
                | IslandWaterStory::ReedyLake
                | IslandWaterStory::CascadeRun
                | IslandWaterStory::WaterfallGarden
                | IslandWaterStory::MistPool
                | IslandWaterStory::CaveSeep
        );
    }

    matches!(
        island.world_tags.water_feature,
        IslandWaterFeature::LakeBasin | IslandWaterFeature::WaterfallSource
    ) || island.is_great_plateau_anchor()
}

fn artifact_has_pebble_field(island: SkyIsland) -> bool {
    !matches!(
        island.terrain_archetype,
        IslandTerrainArchetype::Needle | IslandTerrainArchetype::MistStep
    )
}

pub(crate) fn island_water_visual_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandWaterVisualSpec> {
    let mut specs = Vec::new();
    let water_story =
        authored_island_art_direction(island.name).map(|art_direction| art_direction.water_story);
    let pond_story = matches!(
        water_story,
        Some(
            IslandWaterStory::SpringPond | IslandWaterStory::MistPool | IslandWaterStory::CaveSeep
        )
    ) || water_story.is_none()
        && island.world_tags.water_feature == IslandWaterFeature::Pond;
    if pond_story && !island.is_great_plateau_anchor() {
        let (label, pond_offset, radius_x_scale, radius_z_scale, wind_motion_scale) =
            match water_story {
                Some(IslandWaterStory::MistPool) => {
                    ("mist pool", Vec2::new(-0.40, -0.28), 0.10, 0.07, 0.72)
                }
                Some(IslandWaterStory::CaveSeep) => {
                    ("cave seep pool", Vec2::new(-0.30, 0.14), 0.18, 0.10, 0.48)
                }
                _ if island.is_target => ("spring pond", Vec2::new(-0.34, 0.18), 0.12, 0.08, 1.0),
                _ => ("spring pond", Vec2::new(0.18, 0.28), 0.12, 0.08, 1.0),
            };
        let rotation_y = 0.0;
        let mesh = IslandWaterVisualMesh::PondSurface {
            radius_x: island.half_extents.x * radius_x_scale,
            radius_z: island.half_extents.y * radius_z_scale,
        };
        let seed = 11_000 + island_index as u32 * 149;
        let translation = terrain_clear_horizontal_water_translation(
            island,
            island_water_surface_position(island, pond_offset),
            rotation_y,
            mesh,
            seed,
        );
        specs.push(IslandWaterVisualSpec {
            kind: IslandWaterVisualKind::PondSurface,
            label,
            translation,
            rotation_y,
            wind_phase: 3.4,
            wind_motion_scale,
            mesh,
            seed,
        });
    }

    if island.is_great_plateau_anchor() {
        let broken_edge_lip = plateau_waterfall_lip_offset(island, IslandPlateauRegion::BrokenEdge);
        let north_rim_lip = plateau_waterfall_lip_offset(island, IslandPlateauRegion::CliffRim);
        if let Some(low_basin) = island.plateau_region_position(IslandPlateauRegion::LowBasin) {
            let rotation_y = 0.22;
            let mesh = IslandWaterVisualMesh::LakeSurface {
                radius_x: island.half_extents.x * 0.24,
                radius_z: island.half_extents.y * 0.17,
            };
            let seed = 31_000 + island_index as u32 * 191;
            specs.push(IslandWaterVisualSpec {
                kind: IslandWaterVisualKind::PlateauLakeSurface,
                label: "low basin lake",
                translation: terrain_clear_horizontal_water_translation(
                    island, low_basin, rotation_y, mesh, seed,
                ),
                rotation_y,
                wind_phase: 4.7,
                wind_motion_scale: 1.45,
                mesh,
                seed,
            });
        }
        if let Some(high_shelf) = island.plateau_region_position(IslandPlateauRegion::HighShelf) {
            let rotation_y = -0.18;
            let mesh = IslandWaterVisualMesh::LakeSurface {
                radius_x: island.half_extents.x * 0.13,
                radius_z: island.half_extents.y * 0.09,
            };
            let seed = 32_000 + island_index as u32 * 193;
            specs.push(IslandWaterVisualSpec {
                kind: IslandWaterVisualKind::PlateauLakeSurface,
                label: "high shelf pool",
                translation: terrain_clear_horizontal_water_translation(
                    island, high_shelf, rotation_y, mesh, seed,
                ),
                rotation_y,
                wind_phase: 5.2,
                wind_motion_scale: 1.25,
                mesh,
                seed,
            });
        }
        for channel in [
            RiverChannelFeatureSpec {
                label: "high shelf spillway",
                source_offset: IslandPlateauRegion::HighShelf.sample_offset(),
                outlet_offset: Vec2::new(-0.20, 0.10),
                width_scale: 0.030,
                index: 0,
            },
            RiverChannelFeatureSpec {
                label: "meadow runnel",
                source_offset: Vec2::new(-0.20, 0.10),
                outlet_offset: IslandPlateauRegion::LowBasin.sample_offset(),
                width_scale: 0.028,
                index: 1,
            },
            RiverChannelFeatureSpec {
                label: "broken edge outflow",
                source_offset: IslandPlateauRegion::LowBasin.sample_offset(),
                outlet_offset: broken_edge_lip,
                width_scale: 0.032,
                index: 2,
            },
            RiverChannelFeatureSpec {
                label: "north rim overflow",
                source_offset: IslandPlateauRegion::HighShelf.sample_offset(),
                outlet_offset: north_rim_lip,
                width_scale: 0.026,
                index: 3,
            },
        ] {
            push_river_channel_spec(&mut specs, island_index, island, channel);
        }
        for waterfall in [
            PlateauWaterfallFeatureSpec {
                region: IslandPlateauRegion::BrokenEdge,
                ribbon_label: "broken edge waterfall",
                mist_label: "broken edge waterfall mist",
                width_scale: 0.17,
                index: 0,
            },
            PlateauWaterfallFeatureSpec {
                region: IslandPlateauRegion::CliffRim,
                ribbon_label: "north rim waterfall",
                mist_label: "north rim waterfall mist",
                width_scale: 0.13,
                index: 1,
            },
        ] {
            push_plateau_waterfall_specs(&mut specs, island_index, island, waterfall);
        }
    }

    let waterfall_story = water_story == Some(IslandWaterStory::WaterfallGarden)
        || water_story.is_none()
            && island.world_tags.water_feature == IslandWaterFeature::WaterfallSource;
    if waterfall_story && !island.is_great_plateau_anchor() {
        push_river_channel_spec(
            &mut specs,
            island_index,
            island,
            RiverChannelFeatureSpec {
                label: "waterfall feeder channel",
                source_offset: Vec2::new(0.04, -0.08),
                outlet_offset: ROUTE_EDGE_WATERFALL_CHANNEL_OUTLET_OFFSET,
                width_scale: 0.045,
                index: 0,
            },
        );
        push_route_edge_waterfall_specs(&mut specs, island_index, island);
    }
    let lake_story = matches!(
        water_story,
        Some(IslandWaterStory::ReflectingBasin | IslandWaterStory::ReedyLake)
    ) || water_story.is_none()
        && island.world_tags.water_feature == IslandWaterFeature::LakeBasin;
    if lake_story && !island.is_great_plateau_anchor() {
        push_route_lake_surface_specs(&mut specs, island_index, island);
        let lake_offset = route_lake_basin_offset(island);
        let source_offset = if lake_offset.x >= 0.0 {
            Vec2::new(-0.34, 0.28)
        } else {
            Vec2::new(0.30, -0.26)
        };
        push_river_channel_spec(
            &mut specs,
            island_index,
            island,
            RiverChannelFeatureSpec {
                label: "lake inlet channel",
                source_offset,
                outlet_offset: lake_offset.lerp(source_offset, 0.42),
                width_scale: 0.040,
                index: 0,
            },
        );
    }

    specs
}

pub(crate) fn island_lake_basin_visual_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandLakeBasinVisualSpec> {
    let mut specs = Vec::new();
    let water_specs = island_water_visual_specs(island_index, island);

    if island.is_great_plateau_anchor() {
        if let Some(low_basin) = island.plateau_region_position(IslandPlateauRegion::LowBasin) {
            let translation = water_specs
                .iter()
                .find(|spec| spec.label == "low basin lake")
                .map_or(low_basin + Vec3::Y * 0.035, |spec| {
                    spec.translation - Vec3::Y * 0.045
                });
            specs.push(IslandLakeBasinVisualSpec {
                label: "low basin lake basin",
                translation,
                rotation_y: 0.22,
                radius_x: island.half_extents.x * 0.255,
                radius_z: island.half_extents.y * 0.185,
                rim_width: island.half_extents.min_element() * 0.035,
                rim_height: island.thickness * 0.025,
                seed: 35_000 + island_index as u32 * 211,
            });
        }
        if let Some(high_shelf) = island.plateau_region_position(IslandPlateauRegion::HighShelf) {
            let translation = water_specs
                .iter()
                .find(|spec| spec.label == "high shelf pool")
                .map_or(high_shelf + Vec3::Y * 0.035, |spec| {
                    spec.translation - Vec3::Y * 0.045
                });
            specs.push(IslandLakeBasinVisualSpec {
                label: "high shelf lake basin",
                translation,
                rotation_y: -0.18,
                radius_x: island.half_extents.x * 0.145,
                radius_z: island.half_extents.y * 0.105,
                rim_width: island.half_extents.min_element() * 0.025,
                rim_height: island.thickness * 0.021,
                seed: 36_000 + island_index as u32 * 213,
            });
        }
    }

    let lake_story = authored_island_art_direction(island.name).is_some_and(|art_direction| {
        matches!(
            art_direction.water_story,
            IslandWaterStory::ReflectingBasin | IslandWaterStory::ReedyLake
        )
    }) || authored_island_art_direction(island.name).is_none()
        && island.world_tags.water_feature == IslandWaterFeature::LakeBasin;
    if lake_story && !island.is_great_plateau_anchor() {
        let label = if island.terrain_archetype == IslandTerrainArchetype::SapphireBasin {
            "sapphire lake basin"
        } else {
            "route lake basin"
        };
        let fallback = island_water_surface_position(island, route_lake_basin_offset(island))
            + Vec3::Y * 0.035;
        let translation = water_specs
            .iter()
            .find(|spec| spec.kind == IslandWaterVisualKind::RouteLakeSurface)
            .map_or(fallback, |spec| spec.translation - Vec3::Y * 0.040);
        specs.push(IslandLakeBasinVisualSpec {
            label,
            translation,
            rotation_y: -0.08,
            radius_x: island.half_extents.x * 0.19,
            radius_z: island.half_extents.y * 0.14,
            rim_width: island.half_extents.min_element() * 0.030,
            rim_height: island.thickness * 0.030,
            seed: 37_000 + island_index as u32 * 217,
        });
    }

    specs
}

pub(crate) fn route_cairn_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_cairn_stones(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        radius,
        height,
        seed,
    );

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn launch_beacon_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_cairn_stones(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        radius,
        height * 0.50,
        seed,
    );

    for crystal in 0..LAUNCH_BEACON_CRYSTAL_COUNT {
        let phase = crystal as f32 / LAUNCH_BEACON_CRYSTAL_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, crystal as u32, 701) * 0.42;
        let lean = Vec3::new(phase.cos(), 0.26, phase.sin()).normalize();
        let base = Vec3::new(
            phase.cos() * radius * (0.16 + random_unit(seed, crystal as u32, 709) * 0.18),
            height * (-0.05 + crystal as f32 * 0.055),
            phase.sin() * radius * (0.16 + random_unit(seed, crystal as u32, 719) * 0.18),
        );
        append_crystal_shard(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            base,
            lean,
            radius * (0.13 + random_unit(seed, crystal as u32, 727) * 0.06),
            height * (0.56 + random_unit(seed, crystal as u32, 733) * 0.14),
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn ruin_arch_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let lobe_vertices =
        (LANDMARK_LOBE_LATITUDE_SEGMENTS + 1) * (LANDMARK_LOBE_LONGITUDE_SEGMENTS + 1);
    let lobe_indices = LANDMARK_LOBE_LATITUDE_SEGMENTS * LANDMARK_LOBE_LONGITUDE_SEGMENTS * 6;
    let mut positions = Vec::with_capacity(RUIN_ARCH_STONE_COUNT * lobe_vertices);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(RUIN_ARCH_STONE_COUNT * lobe_indices);

    for side in [-1.0_f32, 1.0] {
        for layer in 0..3 {
            let t = layer as f32 / 2.0;
            let lean =
                random_unit(seed, layer as u32, if side < 0.0 { 1_307 } else { 1_313 }) - 0.5;
            let center = Vec3::new(
                side * width * (0.35 + lean * 0.035),
                height * (-0.31 + t * 0.22),
                (random_unit(seed, layer as u32, if side < 0.0 { 1_319 } else { 1_327 }) - 0.5)
                    * depth
                    * 0.18,
            );
            let lobe_radius = Vec3::new(
                width * (0.155 + random_unit(seed, layer as u32, 1_331) * 0.030),
                height * (0.115 + random_unit(seed, layer as u32, 1_337) * 0.020),
                depth * (0.44 + random_unit(seed, layer as u32, 1_339) * 0.11),
            );
            append_ellipsoid_lobe(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                center,
                lobe_radius,
                LANDMARK_LOBE_LATITUDE_SEGMENTS,
                LANDMARK_LOBE_LONGITUDE_SEGMENTS,
                seed.wrapping_add(layer as u32 * 91 + if side < 0.0 { 17 } else { 43 }),
                0.22,
            );
        }
    }

    for crown in 0..5 {
        let t = crown as f32 / 4.0;
        let angle = std::f32::consts::PI * (0.82 - t * 0.64);
        let center = Vec3::new(
            angle.cos() * width * 0.38,
            height * (-0.03 + angle.sin() * 0.54),
            (random_unit(seed, crown as u32, 1_401) - 0.5) * depth * 0.14,
        );
        let crown_scale = 1.0 - (t - 0.5).abs() * 0.20;
        let lobe_radius = Vec3::new(
            width * (0.145 + random_unit(seed, crown as u32, 1_409) * 0.035) * crown_scale,
            height * (0.105 + random_unit(seed, crown as u32, 1_419) * 0.020),
            depth * (0.48 + random_unit(seed, crown as u32, 1_421) * 0.14),
        );
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            lobe_radius,
            LANDMARK_LOBE_LATITUDE_SEGMENTS,
            LANDMARK_LOBE_LONGITUDE_SEGMENTS,
            seed.wrapping_add(crown as u32 * 103 + 211),
            0.24,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn landing_garden_marker_mesh(length: f32, width: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity((LANDING_GARDEN_MARKER_SEGMENTS + 1) * 3);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(LANDING_GARDEN_MARKER_SEGMENTS * 12);

    for segment in 0..=LANDING_GARDEN_MARKER_SEGMENTS {
        let t = segment as f32 / LANDING_GARDEN_MARKER_SEGMENTS as f32;
        let centered_t = t - 0.5;
        let edge_noise = random_unit(seed, segment as u32, 811) - 0.5;
        let half_width = width * (0.44 + edge_noise * 0.10);
        let x = centered_t * length;
        let arch = (1.0 - centered_t.abs() * 1.6).max(0.0);
        let center_y = 0.10 + arch.powf(1.8) * width * 0.26;
        let edge_y = 0.04 + (random_unit(seed, segment as u32, 823) - 0.5) * width * 0.035;
        let normal = Vec3::new(
            (random_unit(seed, segment as u32, 829) - 0.5) * 0.08,
            1.0,
            (random_unit(seed, segment as u32, 839) - 0.5) * 0.08,
        )
        .normalize();

        positions.extend([
            [x, edge_y, -half_width],
            [x, center_y, 0.0],
            [x, edge_y, half_width],
        ]);
        normals.extend([normal.to_array(); 3]);
        uvs.extend([[t, 0.0], [t, 0.5], [t, 1.0]]);
    }

    for segment in 0..LANDING_GARDEN_MARKER_SEGMENTS {
        let current = (segment * 3) as u32;
        let next = current + 3;
        indices.extend([
            current,
            next,
            current + 1,
            current + 1,
            next,
            next + 1,
            current + 1,
            next + 1,
            current + 2,
            current + 2,
            next + 1,
            next + 2,
        ]);
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn garden_ring_mesh(radius: f32, width: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity((GARDEN_RING_SEGMENTS + 1) * GARDEN_RING_BANDS);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(GARDEN_RING_SEGMENTS * (GARDEN_RING_BANDS - 1) * 6);
    let radius = radius.max(1.0);
    let width = width.max(0.25);
    let height = height.max(0.08);

    for segment in 0..=GARDEN_RING_SEGMENTS {
        let wrapped_segment = segment % GARDEN_RING_SEGMENTS;
        let t = segment as f32 / GARDEN_RING_SEGMENTS as f32;
        let angle = t * std::f32::consts::TAU;
        let radial = Vec2::new(angle.cos(), angle.sin());
        let tangent = Vec2::new(-angle.sin(), angle.cos());
        let edge_noise = random_unit(seed, wrapped_segment as u32, 1_609) - 0.5;
        let crest_noise = random_unit(seed, wrapped_segment as u32, 1_613) - 0.5;
        let skew = (random_unit(seed, wrapped_segment as u32, 1_619) - 0.5) * width * 0.22;
        let center_radius =
            radius * (1.0 + 0.035 * (angle * 5.0 + seed as f32 * 0.011).sin()) + skew;
        let ring_width = width * (0.82 + edge_noise * 0.26);
        let crest_height =
            height * (0.64 + random_unit(seed, wrapped_segment as u32, 1_631) * 0.54);
        let base_y = height * 0.04 + crest_noise * height * 0.08;
        let band_specs: [(f32, f32, f32); GARDEN_RING_BANDS] = [
            (-0.58, base_y, 0.58),
            (-0.18, crest_height, 0.18),
            (0.18, crest_height * (0.82 + edge_noise * 0.12), 0.18),
            (0.58, base_y + edge_noise * height * 0.05, 0.58),
        ];

        for (band, (radial_offset, y, slope)) in band_specs.into_iter().enumerate() {
            let lane_radius = center_radius + radial_offset * ring_width;
            let tangential_waver = tangent
                * ((random_unit(seed, wrapped_segment as u32 + band as u32 * 13, 1_637) - 0.5)
                    * width
                    * 0.08);
            let position = radial * lane_radius + tangential_waver;
            let horizontal_normal = radial * slope.copysign(radial_offset);
            let normal = Vec3::new(
                horizontal_normal.x,
                0.78 + (crest_height / height).clamp(0.0, 1.4) * 0.18,
                horizontal_normal.y,
            )
            .normalize();
            positions.push([position.x, y, position.y]);
            normals.push(normal.to_array());
            uvs.push([t, band as f32 / (GARDEN_RING_BANDS - 1) as f32]);
        }
    }

    for segment in 0..GARDEN_RING_SEGMENTS {
        let current = (segment * GARDEN_RING_BANDS) as u32;
        let next = current + GARDEN_RING_BANDS as u32;
        for band in 0..GARDEN_RING_BANDS - 1 {
            let band = band as u32;
            indices.extend([
                current + band,
                next + band,
                current + band + 1,
                current + band + 1,
                next + band,
                next + band + 1,
            ]);
        }
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn lake_basin_rim_mesh(
    radius_x: f32,
    radius_z: f32,
    rim_width: f32,
    rim_height: f32,
    seed: u32,
) -> Mesh {
    let mut positions = Vec::with_capacity((LAKE_BASIN_RIM_SEGMENTS + 1) * LAKE_BASIN_RIM_BANDS);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(LAKE_BASIN_RIM_SEGMENTS * (LAKE_BASIN_RIM_BANDS - 1) * 6);
    let radius_x = radius_x.max(1.0);
    let radius_z = radius_z.max(1.0);
    let rim_width = rim_width.max(0.35);
    let rim_height = rim_height.max(0.12);
    let band_specs: [(f32, f32, f32); LAKE_BASIN_RIM_BANDS] = [
        (-0.82, 0.08, -0.52),
        (-0.34, 0.34, -0.22),
        (0.08, 1.00, 0.10),
        (0.52, 0.48, 0.36),
        (0.96, 0.14, 0.58),
    ];

    for segment in 0..=LAKE_BASIN_RIM_SEGMENTS {
        let wrapped_segment = segment % LAKE_BASIN_RIM_SEGMENTS;
        let t = segment as f32 / LAKE_BASIN_RIM_SEGMENTS as f32;
        let angle = t * std::f32::consts::TAU;
        let radial = Vec2::new(angle.cos(), angle.sin());
        let tangent = Vec2::new(-angle.sin(), angle.cos());
        let basin_noise = random_unit(seed, wrapped_segment as u32, 1_701) - 0.5;
        let scallop = 0.035 * (angle * 6.0 + seed as f32 * 0.013).sin()
            + 0.020 * (angle * 11.0 + seed as f32 * 0.019).cos();

        for (band, (offset, height_factor, slope)) in band_specs.into_iter().enumerate() {
            let lane_noise =
                random_unit(seed, wrapped_segment as u32 + band as u32 * 19, 1_709) - 0.5;
            let lane_offset = offset * rim_width * (0.92 + basin_noise * 0.18);
            let lane_radius_x = radius_x * (1.0 + scallop) + lane_offset;
            let lane_radius_z = radius_z * (1.0 + scallop * 0.82) + lane_offset * 0.72;
            let shore_waver = tangent * lane_noise * rim_width * 0.10;
            let position =
                Vec2::new(radial.x * lane_radius_x, radial.y * lane_radius_z) + shore_waver;
            let y = rim_height
                * (height_factor
                    + lane_noise * 0.07
                    + basin_noise * 0.04
                    + (angle * 3.0 + band as f32).sin() * 0.018);
            let horizontal_scale = if offset < 0.0 { -1.0 } else { 1.0 };
            let horizontal_normal = radial * slope.abs() * horizontal_scale;
            let normal = Vec3::new(
                horizontal_normal.x,
                0.76 + height_factor * 0.16,
                horizontal_normal.y,
            )
            .normalize();

            positions.push([position.x, y, position.y]);
            normals.push(normal.to_array());
            uvs.push([t, band as f32 / (LAKE_BASIN_RIM_BANDS - 1) as f32]);
        }
    }

    for segment in 0..LAKE_BASIN_RIM_SEGMENTS {
        let current = (segment * LAKE_BASIN_RIM_BANDS) as u32;
        let next = current + LAKE_BASIN_RIM_BANDS as u32;
        for band in 0..LAKE_BASIN_RIM_BANDS - 1 {
            let band = band as u32;
            indices.extend([
                current + band,
                next + band,
                current + band + 1,
                current + band + 1,
                next + band,
                next + band + 1,
            ]);
        }
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn pond_surface_mesh(radius_x: f32, radius_z: f32, seed: u32) -> Mesh {
    irregular_water_surface_mesh(
        radius_x,
        radius_z,
        seed,
        POND_SURFACE_SEGMENTS,
        &POND_SURFACE_RINGS,
        0.15,
        0.040,
    )
}

pub(crate) fn lake_surface_mesh(radius_x: f32, radius_z: f32, seed: u32) -> Mesh {
    irregular_water_surface_mesh(
        radius_x,
        radius_z,
        seed,
        LAKE_SURFACE_SEGMENTS,
        &LAKE_SURFACE_RINGS,
        0.20,
        0.085,
    )
}

pub(crate) fn river_channel_surface_mesh(
    length: f32,
    width: f32,
    elevation_drop: f32,
    seed: u32,
) -> Mesh {
    let row_count = RIVER_CHANNEL_SEGMENTS + 1;
    let mut vertices = Vec::with_capacity(row_count * RIVER_CHANNEL_COLUMNS);
    let mut indices = Vec::with_capacity(RIVER_CHANNEL_SEGMENTS * (RIVER_CHANNEL_COLUMNS - 1) * 6);
    visit_river_channel_surface_vertices(length, width, elevation_drop, seed, |vertex| {
        vertices.push(vertex);
    });

    for segment in 0..RIVER_CHANNEL_SEGMENTS {
        for column in 0..RIVER_CHANNEL_COLUMNS - 1 {
            let current = (segment * RIVER_CHANNEL_COLUMNS + column) as u32;
            let right = current + 1;
            let next = current + RIVER_CHANNEL_COLUMNS as u32;
            let next_right = next + 1;
            indices.extend([current, next, right, right, next, next_right]);
        }
    }

    build_water_mesh(vertices, indices)
}

pub(crate) fn waterfall_ribbon_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let width = width.max(0.1);
    let height = height.max(0.1);
    let depth = depth.max(0.01);
    let mut vertices = Vec::with_capacity(WATERFALL_RIBBON_COLUMNS * WATERFALL_RIBBON_ROWS);
    let mut indices =
        Vec::with_capacity((WATERFALL_RIBBON_COLUMNS - 1) * (WATERFALL_RIBBON_ROWS - 1) * 6);
    let primary_phase = random_unit(seed, 0, 1_021) * std::f32::consts::TAU;
    let secondary_phase = random_unit(seed, 1, 1_027) * std::f32::consts::TAU;

    for row in 0..WATERFALL_RIBBON_ROWS {
        let v = row as f32 / (WATERFALL_RIBBON_ROWS - 1) as f32;
        let y = height * (0.5 - v);
        for column in 0..WATERFALL_RIBBON_COLUMNS {
            let u = column as f32 / (WATERFALL_RIBBON_COLUMNS - 1) as f32;
            let centered_u = u - 0.5;
            let taper = 1.0 - v * 0.18 + (v * std::f32::consts::PI).sin() * 0.10;
            let sheet_wave = (v * std::f32::consts::TAU * 2.8
                + u * std::f32::consts::TAU * 0.65
                + primary_phase)
                .sin();
            let cross_wave = (v * std::f32::consts::TAU * 7.2 - u * std::f32::consts::TAU * 1.35
                + secondary_phase)
                .sin();
            let x = centered_u * width * taper
                + (v * std::f32::consts::TAU * 2.1 + primary_phase).sin() * width * 0.022;
            let z = depth
                * (sheet_wave * 0.58
                    + cross_wave * 0.24
                    + (centered_u * std::f32::consts::PI).sin() * 0.12);
            let edge_foam = (centered_u.abs() * 2.0).powf(1.35);
            let impact_churn = smoothstep(0.68, 1.0, v);
            let sheet_churn = (0.5 + 0.5 * cross_wave) * (0.18 + impact_churn * 0.24);
            let foam_mask = edge_foam
                .max(0.18 + sheet_churn + impact_churn * 0.32)
                .clamp(0.0, 1.0);

            vertices.push(WaterSurfaceVertex {
                position: Vec3::new(x, y, z),
                uv: [u, v],
                foam_mask,
                flow_class: 1.0,
            });
        }
    }

    for row in 0..WATERFALL_RIBBON_ROWS - 1 {
        for column in 0..WATERFALL_RIBBON_COLUMNS - 1 {
            let current = (row * WATERFALL_RIBBON_COLUMNS + column) as u32;
            let right = current + 1;
            let next = current + WATERFALL_RIBBON_COLUMNS as u32;
            let next_right = next + 1;
            indices.extend([current, next, right, right, next, next_right]);
        }
    }

    build_water_mesh(vertices, indices)
}

pub(crate) fn waterfall_mist_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for lobe in 0..WATERFALL_MIST_LOBES {
        let t = lobe as f32 / WATERFALL_MIST_LOBES as f32;
        let phase = t * std::f32::consts::TAU + random_unit(seed, lobe as u32, 1_117) * 0.45;
        let ring_radius = radius * (0.12 + random_unit(seed, lobe as u32, 1_123) * 0.62);
        let center = Vec3::new(
            phase.cos() * ring_radius,
            height * (0.18 + random_unit(seed, lobe as u32, 1_131) * 0.38),
            phase.sin() * ring_radius * 0.72,
        );
        let lobe_radius = Vec3::new(
            radius * (0.22 + random_unit(seed, lobe as u32, 1_139) * 0.14),
            height * (0.20 + random_unit(seed, lobe as u32, 1_149) * 0.18),
            radius * (0.16 + random_unit(seed, lobe as u32, 1_151) * 0.12),
        );
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            lobe_radius,
            LANDMARK_LOBE_LATITUDE_SEGMENTS,
            LANDMARK_LOBE_LONGITUDE_SEGMENTS,
            seed.wrapping_add(lobe as u32 * 97),
            0.28,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

fn irregular_water_surface_mesh(
    radius_x: f32,
    radius_z: f32,
    seed: u32,
    segments: usize,
    rings: &[f32],
    edge_noise_scale: f32,
    ripple_scale: f32,
) -> Mesh {
    let mut vertices = Vec::with_capacity(1 + segments * rings.len());
    let mut indices = Vec::with_capacity(segments * (3 + rings.len().saturating_sub(1) * 6));
    visit_irregular_water_surface_vertices(
        radius_x,
        radius_z,
        seed,
        segments,
        rings,
        edge_noise_scale,
        ripple_scale,
        |vertex| vertices.push(vertex),
    );

    let ring_vertex_index = |ring_index: usize, segment: usize| -> u32 {
        1 + (ring_index * segments + segment % segments) as u32
    };

    for segment in 0..segments {
        indices.extend([
            0,
            ring_vertex_index(0, segment + 1),
            ring_vertex_index(0, segment),
        ]);
    }
    for ring_index in 0..rings.len().saturating_sub(1) {
        for segment in 0..segments {
            indices.extend([
                ring_vertex_index(ring_index, segment),
                ring_vertex_index(ring_index, segment + 1),
                ring_vertex_index(ring_index + 1, segment),
                ring_vertex_index(ring_index, segment + 1),
                ring_vertex_index(ring_index + 1, segment + 1),
                ring_vertex_index(ring_index + 1, segment),
            ]);
        }
    }

    build_water_mesh(vertices, indices)
}

fn push_river_channel_spec(
    specs: &mut Vec<IslandWaterVisualSpec>,
    island_index: usize,
    island: SkyIsland,
    channel: RiverChannelFeatureSpec,
) {
    let source = island_water_surface_position(island, channel.source_offset);
    let outlet = island_water_surface_position(island, channel.outlet_offset);
    let horizontal_delta = Vec2::new(outlet.x - source.x, outlet.z - source.z);
    let length = horizontal_delta.length();
    if length <= 0.5 {
        return;
    }

    let direction = horizontal_delta / length;
    let width = (island.half_extents.min_element() * channel.width_scale).clamp(1.2, 5.0);
    let elevation_drop = (source.y - outlet.y).max(width * 0.018).min(length * 0.08);
    let midpoint = (source + outlet) * 0.5;
    let rotation_y = direction.x.atan2(direction.y);
    let mesh = IslandWaterVisualMesh::RiverChannel {
        length,
        width,
        elevation_drop,
    };
    let seed = 40_000 + island_index as u32 * 227 + channel.index * 521;

    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::RiverChannel,
        label: channel.label,
        translation: terrain_clear_horizontal_water_translation(
            island, midpoint, rotation_y, mesh, seed,
        ),
        rotation_y,
        wind_phase: 5.5 + island_index as f32 * 0.029 + channel.index as f32 * 0.43,
        wind_motion_scale: 1.18,
        mesh,
        seed,
    });
}

fn push_plateau_waterfall_specs(
    specs: &mut Vec<IslandWaterVisualSpec>,
    island_index: usize,
    island: SkyIsland,
    waterfall: PlateauWaterfallFeatureSpec,
) {
    let Some(region_surface) = island.plateau_region_position(waterfall.region) else {
        return;
    };
    let sample = waterfall.region.sample_offset();
    let angle = sample.y.atan2(sample.x);
    let contour = island.footprint_contour_point(angle, false);
    let edge_surface = Vec3::new(
        contour.x,
        island.terrain_surface_y_at(Vec3::new(contour.x, region_surface.y, contour.y)),
        contour.y,
    );
    let outward = (Vec2::new(contour.x, contour.y) - Vec2::new(island.center.x, island.center.z))
        .normalize_or_zero();
    let outward = if outward.length_squared() > 0.001 {
        outward
    } else {
        Vec2::Y
    };
    let yaw = outward.x.atan2(outward.y);
    let height = island.thickness * 0.84;
    let width = island.half_extents.min_element() * waterfall.width_scale;
    let outward3 = Vec3::new(outward.x, 0.0, outward.y);
    let ribbon_clearance = (width * 0.15).max(3.0);
    let mist_clearance = (width * 0.25).max(5.0);
    let seed_base = 33_000 + island_index as u32 * 197 + waterfall.index * 1_009;

    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::PlateauWaterfallRibbon,
        label: waterfall.ribbon_label,
        translation: edge_surface + outward3 * ribbon_clearance - Vec3::Y * (height * 0.48),
        rotation_y: yaw,
        wind_phase: 6.1 + waterfall.index as f32 * 0.7,
        wind_motion_scale: 1.8,
        mesh: IslandWaterVisualMesh::WaterfallRibbon {
            width,
            height,
            depth: width * 0.05,
        },
        seed: seed_base,
    });
    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::PlateauWaterfallMist,
        label: waterfall.mist_label,
        translation: edge_surface + outward3 * mist_clearance - Vec3::Y * (height * 0.94),
        rotation_y: yaw,
        wind_phase: 6.8 + waterfall.index as f32 * 0.9,
        wind_motion_scale: 1.55,
        mesh: IslandWaterVisualMesh::WaterfallMist {
            radius: (width * 0.52).max(13.5),
            height: island.thickness * 0.08,
        },
        seed: seed_base + 503,
    });
}

fn plateau_waterfall_lip_offset(island: SkyIsland, region: IslandPlateauRegion) -> Vec2 {
    let sample = region.sample_offset();
    let angle = sample.y.atan2(sample.x);
    let contour = island.footprint_contour_point(angle, false);

    Vec2::new(
        (contour.x - island.center.x) / island.half_extents.x.max(0.001),
        (contour.y - island.center.z) / island.half_extents.y.max(0.001),
    ) * 0.98
}

fn push_route_lake_surface_specs(
    specs: &mut Vec<IslandWaterVisualSpec>,
    island_index: usize,
    island: SkyIsland,
) {
    let rotation_y = -0.08;
    let mesh = IslandWaterVisualMesh::LakeSurface {
        radius_x: island.half_extents.x * 0.18,
        radius_z: island.half_extents.y * 0.13,
    };
    let seed = 38_000 + island_index as u32 * 219;
    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::RouteLakeSurface,
        label: "route lake surface",
        translation: terrain_clear_horizontal_water_translation(
            island,
            island_water_surface_position(island, route_lake_basin_offset(island)),
            rotation_y,
            mesh,
            seed,
        ),
        rotation_y,
        wind_phase: 5.8 + island_index as f32 * 0.037,
        wind_motion_scale: 1.32,
        mesh,
        seed,
    });
}

fn push_route_edge_waterfall_specs(
    specs: &mut Vec<IslandWaterVisualSpec>,
    island_index: usize,
    island: SkyIsland,
) {
    let placement = route_edge_waterfall_placement(island);
    let seed_base = 39_000 + island_index as u32 * 223;

    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::RouteWaterfallRibbon,
        label: "route edge waterfall",
        translation: placement.ribbon_translation,
        rotation_y: placement.rotation_y,
        wind_phase: 7.4,
        wind_motion_scale: 1.65,
        mesh: IslandWaterVisualMesh::WaterfallRibbon {
            width: placement.width,
            height: placement.height,
            depth: placement.width * 0.07,
        },
        seed: seed_base,
    });
    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::RouteWaterfallMist,
        label: "route edge mist",
        translation: placement.mist_translation,
        rotation_y: placement.rotation_y,
        wind_phase: 8.1,
        wind_motion_scale: 1.45,
        mesh: IslandWaterVisualMesh::WaterfallMist {
            radius: placement.width * 0.48,
            height: placement.height * 0.10,
        },
        seed: seed_base + 509,
    });
}

fn route_lake_basin_offset(island: SkyIsland) -> Vec2 {
    if island.terrain_archetype == IslandTerrainArchetype::SapphireBasin {
        Vec2::new(0.06, -0.10)
    } else {
        Vec2::new(-0.08, 0.12)
    }
}

pub(crate) fn artifact_stair_run_mesh(length: f32, width: f32, rise: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let step_depth = length / ARTIFACT_STAIR_STEP_COUNT as f32;
    for step in 0..ARTIFACT_STAIR_STEP_COUNT {
        let t = step as f32 / (ARTIFACT_STAIR_STEP_COUNT - 1) as f32;
        let wobble = random_unit(seed, step as u32, 2_011) - 0.5;
        let step_height = rise * (0.12 + t * 0.88);
        let center = Vec3::new(
            wobble * width * 0.035,
            step_height * 0.5,
            -length * 0.5 + step_depth * (step as f32 + 0.5),
        );
        append_oriented_box(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            Vec3::new(
                width * (0.46 + random_unit(seed, step as u32, 2_019) * 0.06),
                step_height * 0.5,
                step_depth * (0.43 + random_unit(seed, step as u32, 2_023) * 0.08),
            ),
            wobble * 0.08,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn artifact_retaining_wall_mesh(
    length: f32,
    height: f32,
    depth: f32,
    seed: u32,
) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let segment_length = length / ARTIFACT_RETAINING_WALL_SEGMENTS as f32;
    for segment in 0..ARTIFACT_RETAINING_WALL_SEGMENTS {
        let centered = segment as f32 - (ARTIFACT_RETAINING_WALL_SEGMENTS - 1) as f32 * 0.5;
        let segment_height = height * (0.58 + random_unit(seed, segment as u32, 2_101) * 0.45);
        let chip = random_unit(seed, segment as u32, 2_107) - 0.5;
        append_oriented_box(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                centered * segment_length + chip * segment_length * 0.14,
                segment_height * 0.5,
                chip * depth * 0.10,
            ),
            Vec3::new(
                segment_length * (0.47 + random_unit(seed, segment as u32, 2_111) * 0.08),
                segment_height * 0.5,
                depth * (0.42 + random_unit(seed, segment as u32, 2_117) * 0.14),
            ),
            chip * 0.16,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn artifact_glyph_slab_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_oriented_box(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, height * 0.5, 0.0),
        Vec3::new(width * 0.5, height * 0.5, depth * 0.5),
        (random_unit(seed, 0, 2_201) - 0.5) * 0.06,
    );

    for stroke in 0..ARTIFACT_GLYPH_STROKE_COUNT {
        let t = stroke as f32 / (ARTIFACT_GLYPH_STROKE_COUNT - 1) as f32;
        let centered = t - 0.5;
        let horizontal = stroke % 3 == 0;
        let stroke_width = if horizontal {
            width * 0.18
        } else {
            width * 0.045
        };
        let stroke_height = if horizontal {
            height * 0.030
        } else {
            height * (0.13 + random_unit(seed, stroke as u32, 2_211) * 0.05)
        };
        let x = (random_unit(seed, stroke as u32, 2_217) - 0.5) * width * 0.42;
        let y = height * (0.20 + t * 0.58);
        append_oriented_box(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(x, y, depth * 0.54),
            Vec3::new(stroke_width, stroke_height, depth * 0.055),
            centered * 0.34,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn artifact_bridge_fragment_mesh(
    length: f32,
    width: f32,
    thickness: f32,
    seed: u32,
) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let block_length = length / ARTIFACT_BRIDGE_FRAGMENT_COUNT as f32;
    for block in 0..ARTIFACT_BRIDGE_FRAGMENT_COUNT {
        if block == 2 || block == 6 {
            continue;
        }
        let missing_sag = if block > 2 && block < 6 {
            -thickness * 0.30
        } else {
            0.0
        };
        let centered = block as f32 - (ARTIFACT_BRIDGE_FRAGMENT_COUNT - 1) as f32 * 0.5;
        let chip = random_unit(seed, block as u32, 2_301) - 0.5;
        append_oriented_box(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                centered * block_length,
                thickness * 0.5 + missing_sag,
                chip * width * 0.04,
            ),
            Vec3::new(
                block_length * (0.44 + random_unit(seed, block as u32, 2_307) * 0.10),
                thickness * (0.42 + random_unit(seed, block as u32, 2_311) * 0.18),
                width * (0.42 + random_unit(seed, block as u32, 2_317) * 0.10),
            ),
            chip * 0.10,
        );
    }

    for rail_side in [-1.0_f32, 1.0] {
        for post in [0_usize, 4, 8] {
            let centered = post as f32 - (ARTIFACT_BRIDGE_FRAGMENT_COUNT - 1) as f32 * 0.5;
            append_oriented_box(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                Vec3::new(
                    centered * block_length,
                    thickness * 1.45,
                    rail_side * width * 0.47,
                ),
                Vec3::new(block_length * 0.16, thickness * 0.72, thickness * 0.18),
                rail_side * 0.06,
            );
        }
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn artifact_banner_strips_mesh(width: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_oriented_box(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(width * 0.54, height * 0.018, height * 0.020),
        0.0,
    );

    for strip in 0..ARTIFACT_BANNER_STRIP_COUNT {
        let t = strip as f32 / (ARTIFACT_BANNER_STRIP_COUNT - 1) as f32;
        let x = (t - 0.5) * width;
        let strip_height = height * (0.56 + random_unit(seed, strip as u32, 2_401) * 0.38);
        let wind_lean = (random_unit(seed, strip as u32, 2_407) - 0.5) * width * 0.11;
        append_double_sided_detail_card(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(x + wind_lean, -strip_height * 0.48, 0.0),
            Vec3::X,
            Vec3::Y,
            width * (0.030 + random_unit(seed, strip as u32, 2_411) * 0.018),
            strip_height * 0.5,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn artifact_pebble_field_mesh(radius: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for pebble in 0..ARTIFACT_PEBBLE_COUNT {
        let angle = random_unit(seed, pebble as u32, 2_501) * std::f32::consts::TAU;
        let distance = radius * random_unit(seed, pebble as u32, 2_503).sqrt();
        let pebble_radius = radius * (0.035 + random_unit(seed, pebble as u32, 2_509) * 0.045);
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                angle.cos() * distance,
                pebble_radius * 0.34,
                angle.sin() * distance,
            ),
            Vec3::new(
                pebble_radius * (1.2 + random_unit(seed, pebble as u32, 2_521) * 0.8),
                pebble_radius * (0.34 + random_unit(seed, pebble as u32, 2_523) * 0.24),
                pebble_radius * (0.8 + random_unit(seed, pebble as u32, 2_527) * 0.7),
            ),
            3,
            7,
            seed.wrapping_add(pebble as u32 * 41),
            0.22,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn artifact_reed_patch_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for reed in 0..ARTIFACT_REED_COUNT {
        let angle = random_unit(seed, reed as u32, 2_601) * std::f32::consts::TAU;
        let distance = radius * random_unit(seed, reed as u32, 2_607).sqrt();
        let reed_height = height * (0.62 + random_unit(seed, reed as u32, 2_611) * 0.56);
        let center = Vec3::new(
            angle.cos() * distance,
            reed_height * 0.5,
            angle.sin() * distance,
        );
        let tangent_angle = angle + std::f32::consts::FRAC_PI_2;
        append_double_sided_detail_card(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            Vec3::new(tangent_angle.cos(), 0.0, tangent_angle.sin()),
            Vec3::Y,
            radius * (0.010 + random_unit(seed, reed as u32, 2_617) * 0.008),
            reed_height * 0.5,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

fn island_water_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let radius = normalized_offset.length();
    let playable_offset = if radius <= f32::EPSILON {
        Vec2::ZERO
    } else {
        let angle = normalized_offset.y.atan2(normalized_offset.x);
        let max_radius = island.playable_silhouette_scale(angle) * 0.94;
        normalized_offset / radius * radius.min(max_radius)
    };
    let x = island.center.x + island.half_extents.x * playable_offset.x;
    let z = island.center.z + island.half_extents.y * playable_offset.y;
    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
}

fn terrain_clear_horizontal_water_translation(
    island: SkyIsland,
    center: Vec3,
    rotation_y: f32,
    water_mesh: IslandWaterVisualMesh,
    seed: u32,
) -> Vec3 {
    let rotation = Quat::from_rotation_y(rotation_y);
    let mut required_translation_y = center.y;
    water_mesh.visit_horizontal_surface_positions(seed, |local| {
        let rotated = rotation * local;
        let world_probe = Vec3::new(center.x + rotated.x, island.center.y, center.z + rotated.z);
        required_translation_y =
            required_translation_y.max(island.mesh_top_y_at(world_probe) - local.y);
    });

    Vec3::new(
        center.x,
        required_translation_y + HORIZONTAL_WATER_TERRAIN_CLEARANCE_M,
        center.z,
    )
}

#[allow(clippy::too_many_arguments)]
fn visit_irregular_water_surface_vertices(
    radius_x: f32,
    radius_z: f32,
    seed: u32,
    segments: usize,
    rings: &[f32],
    edge_noise_scale: f32,
    ripple_scale: f32,
    mut visitor: impl FnMut(WaterSurfaceVertex),
) {
    visitor(WaterSurfaceVertex {
        position: Vec3::ZERO,
        uv: [0.5, 0.5],
        foam_mask: 0.0,
        flow_class: 0.0,
    });
    for ring in rings.iter().copied() {
        for segment in 0..segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let shoreline_noise = (random_unit(seed, segment as u32, 907) - 0.5) * edge_noise_scale
                + 0.035 * (angle * 5.0 + seed as f32 * 0.011).sin()
                + 0.018 * (angle * 11.0 + seed as f32 * 0.023).cos();
            let edge = 1.0 + shoreline_noise * ring * ring;
            let planar = Vec2::new(
                angle.cos() * radius_x * ring * edge,
                angle.sin() * radius_z * ring * edge,
            );
            let ripple = coherent_calm_water_relief(planar, seed, ripple_scale);
            let shore_foam = smoothstep(0.54, 1.0, ring);
            let crest_foam = (ripple / ripple_scale.max(0.001)).max(0.0) * 0.10 * ring;
            let foam_mask = if ring >= 0.999 {
                1.0
            } else {
                (shore_foam + crest_foam * (1.0 - shore_foam)).clamp(0.0, 1.0)
            };

            visitor(WaterSurfaceVertex {
                position: Vec3::new(planar.x, ripple, planar.y),
                uv: [
                    0.5 + angle.cos() * ring * 0.5,
                    0.5 + angle.sin() * ring * 0.5,
                ],
                foam_mask,
                flow_class: 0.0,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn visit_irregular_water_surface_positions(
    radius_x: f32,
    radius_z: f32,
    seed: u32,
    segments: usize,
    rings: &[f32],
    edge_noise_scale: f32,
    ripple_scale: f32,
    mut visitor: impl FnMut(Vec3),
) {
    visit_irregular_water_surface_vertices(
        radius_x,
        radius_z,
        seed,
        segments,
        rings,
        edge_noise_scale,
        ripple_scale,
        |vertex| visitor(vertex.position),
    );
}

fn coherent_calm_water_relief(position: Vec2, seed: u32, amplitude: f32) -> f32 {
    let primary_angle = random_unit(seed, 0, 931) * std::f32::consts::TAU;
    let secondary_angle = random_unit(seed, 1, 937) * std::f32::consts::TAU;
    let crossing_angle = random_unit(seed, 2, 941) * std::f32::consts::TAU;
    let primary_direction = Vec2::new(primary_angle.cos(), primary_angle.sin());
    let secondary_direction = Vec2::new(secondary_angle.cos(), secondary_angle.sin());
    let crossing_direction = Vec2::new(crossing_angle.cos(), crossing_angle.sin());
    let primary_phase = random_unit(seed, 3, 947) * std::f32::consts::TAU;
    let secondary_phase = random_unit(seed, 4, 953) * std::f32::consts::TAU;
    let crossing_phase = random_unit(seed, 5, 967) * std::f32::consts::TAU;
    let sample = (position.dot(primary_direction) * 1.18 + primary_phase).sin() * 0.56
        + (position.dot(secondary_direction) * 2.08 + secondary_phase).sin() * 0.29
        + (position.dot(crossing_direction) * 0.61 + crossing_phase).sin() * 0.15;
    let origin =
        primary_phase.sin() * 0.56 + secondary_phase.sin() * 0.29 + crossing_phase.sin() * 0.15;
    (sample - origin) * amplitude
}

fn smoothstep(edge_start: f32, edge_end: f32, value: f32) -> f32 {
    let t = ((value - edge_start) / (edge_end - edge_start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn visit_river_channel_surface_vertices(
    length: f32,
    width: f32,
    elevation_drop: f32,
    seed: u32,
    mut visitor: impl FnMut(WaterSurfaceVertex),
) {
    let length = length.max(0.1);
    let width = width.max(0.05);
    let elevation_drop = elevation_drop.max(0.0);
    let bend_sign = if random_unit(seed, 0, 971) < 0.5 {
        -1.0
    } else {
        1.0
    };
    let primary_bend = bend_sign * width * (0.48 + random_unit(seed, 1, 977) * 0.34);
    let wave_phase = random_unit(seed, 2, 983) * std::f32::consts::TAU;
    let secondary_wave_phase = random_unit(seed, 3, 991) * std::f32::consts::TAU;
    let width_phase = random_unit(seed, 4, 997) * std::f32::consts::TAU;

    for segment in 0..=RIVER_CHANNEL_SEGMENTS {
        let t = segment as f32 / RIVER_CHANNEL_SEGMENTS as f32;
        let longitudinal = (t - 0.5) * length;
        let envelope = (t * std::f32::consts::PI).sin();
        let envelope_derivative = std::f32::consts::PI * (t * std::f32::consts::PI).cos();
        let wander_phase = t * std::f32::consts::TAU + wave_phase;
        let center_x = envelope * primary_bend + envelope * wander_phase.sin() * width * 0.16;
        let center_derivative = envelope_derivative * primary_bend
            + width
                * 0.16
                * (envelope_derivative * wander_phase.sin()
                    + envelope * std::f32::consts::TAU * wander_phase.cos());
        let lateral_slope = center_derivative / length;
        let across = Vec3::new(1.0, 0.0, -lateral_slope).normalize_or_zero();
        let width_jitter = (t * std::f32::consts::TAU * 2.0 + width_phase).sin() * 0.055
            + (t * std::f32::consts::TAU * 5.0 + secondary_wave_phase).sin() * 0.025;
        let local_width = width * (0.84 + envelope * 0.16 + width_jitter);

        for column in 0..RIVER_CHANNEL_COLUMNS {
            let u = column as f32 / (RIVER_CHANNEL_COLUMNS - 1) as f32;
            let side = (u - 0.5) * local_width;
            let primary_wave = (longitudinal * 1.55 + side * 0.78 + wave_phase).sin();
            let crossing_wave = (longitudinal * 2.70 - side * 1.34 + secondary_wave_phase).sin();
            let ripple = envelope * width * (primary_wave * 0.009 + crossing_wave * 0.004);
            let center_y = elevation_drop * (0.5 - t) + ripple;
            let position = Vec3::new(center_x, center_y, longitudinal) + across * side;
            let edge_foam = smoothstep(0.76, 1.0, (u - 0.5).abs() * 2.0);
            let bend_churn = (lateral_slope.abs() * 0.65).min(0.24) * envelope;
            let wave_churn =
                ((primary_wave * 0.5 + 0.5) * 0.12 + crossing_wave.abs() * 0.05) * envelope;
            let interior_churn = (bend_churn + wave_churn) * (0.25 + edge_foam * 0.75);
            let foam_mask = edge_foam.max(interior_churn).clamp(0.0, 1.0);

            visitor(WaterSurfaceVertex {
                position,
                uv: [u, t],
                foam_mask,
                flow_class: 0.55,
            });
        }
    }
}

fn visit_river_channel_surface_positions(
    length: f32,
    width: f32,
    elevation_drop: f32,
    seed: u32,
    mut visitor: impl FnMut(Vec3),
) {
    visit_river_channel_surface_vertices(length, width, elevation_drop, seed, |vertex| {
        visitor(vertex.position);
    });
}

fn append_cairn_stones(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    radius: f32,
    height: f32,
    seed: u32,
) {
    let stone_height = height / (ROUTE_CAIRN_STONE_COUNT as f32 - 0.3);
    for stone in 0..ROUTE_CAIRN_STONE_COUNT {
        let t = stone as f32 / (ROUTE_CAIRN_STONE_COUNT - 1) as f32;
        let phase = random_unit(seed, stone as u32, 601) * std::f32::consts::TAU;
        let layer_radius = radius * (1.04 - t * 0.46);
        let center = Vec3::new(
            phase.cos() * radius * (0.08 + t * 0.06),
            -height * 0.5 + stone_height * (0.55 + stone as f32 * 0.95),
            phase.sin() * radius * (0.08 + t * 0.06),
        );

        append_ellipsoid_lobe(
            positions,
            normals,
            uvs,
            indices,
            center,
            Vec3::new(
                layer_radius * (0.92 + random_unit(seed, stone as u32, 613) * 0.18),
                stone_height * (0.42 + random_unit(seed, stone as u32, 617) * 0.18),
                layer_radius * (0.70 + random_unit(seed, stone as u32, 619) * 0.20),
            ),
            LANDMARK_LOBE_LATITUDE_SEGMENTS,
            LANDMARK_LOBE_LONGITUDE_SEGMENTS,
            seed.wrapping_add(stone as u32 * 83),
            0.24,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_crystal_shard(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    base: Vec3,
    lean: Vec3,
    radius: f32,
    height: f32,
) {
    let axis = (Vec3::Y + lean * 0.18).normalize_or_zero();
    if axis.length_squared() <= 0.0001 {
        return;
    }
    let side_seed = if axis.dot(Vec3::Y).abs() > 0.94 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let side = axis.cross(side_seed).normalize();
    let bitangent = side.cross(axis).normalize();
    let start = positions.len() as u32;
    let waist = base + axis * height * 0.36;
    let tip = base + axis * height;

    for (ring, (center, ring_radius)) in [(base, radius), (waist, radius * 0.58)]
        .into_iter()
        .enumerate()
    {
        for segment in 0..CRYSTAL_RING_SEGMENTS {
            let phase = segment as f32 / CRYSTAL_RING_SEGMENTS as f32 * std::f32::consts::TAU;
            let radial = side * phase.cos() + bitangent * phase.sin();
            positions.push((center + radial * ring_radius).to_array());
            normals.push(radial.normalize().to_array());
            uvs.push([
                segment as f32 / CRYSTAL_RING_SEGMENTS as f32,
                ring as f32 * 0.6,
            ]);
        }
    }

    let tip_index = positions.len() as u32;
    positions.push(tip.to_array());
    normals.push(axis.to_array());
    uvs.push([0.5, 1.0]);

    let bottom_center = positions.len() as u32;
    positions.push(base.to_array());
    normals.push((-axis).to_array());
    uvs.push([0.5, 0.0]);

    for segment in 0..CRYSTAL_RING_SEGMENTS {
        let next = (segment + 1) % CRYSTAL_RING_SEGMENTS;
        let base_current = start + segment as u32;
        let base_next = start + next as u32;
        let waist_current = start + CRYSTAL_RING_SEGMENTS as u32 + segment as u32;
        let waist_next = start + CRYSTAL_RING_SEGMENTS as u32 + next as u32;
        indices.extend([
            base_current,
            waist_current,
            base_next,
            base_next,
            waist_current,
            waist_next,
            waist_current,
            tip_index,
            waist_next,
            bottom_center,
            base_next,
            base_current,
        ]);
    }
}

fn append_oriented_box(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    half_extents: Vec3,
    yaw: f32,
) {
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let up = Vec3::Y;
    let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let top_tilt = (0.05 + half_extents.y / (half_extents.x + half_extents.z).max(0.1) * 0.18)
        .clamp(0.04, 0.28);
    let top_normal = (up + forward * top_tilt + right * yaw.sin() * 0.18).normalize();
    let corner = |sx: f32, sy: f32, sz: f32| {
        center
            + right * half_extents.x * sx
            + up * half_extents.y * sy
            + forward * half_extents.z * sz
    };
    let faces = [
        (
            right,
            [
                corner(1.0, -1.0, -1.0),
                corner(1.0, -1.0, 1.0),
                corner(1.0, 1.0, 1.0),
                corner(1.0, 1.0, -1.0),
            ],
        ),
        (
            -right,
            [
                corner(-1.0, -1.0, 1.0),
                corner(-1.0, -1.0, -1.0),
                corner(-1.0, 1.0, -1.0),
                corner(-1.0, 1.0, 1.0),
            ],
        ),
        (
            top_normal,
            [
                corner(-1.0, 1.0, -1.0),
                corner(1.0, 1.0, -1.0),
                corner(1.0, 1.0, 1.0),
                corner(-1.0, 1.0, 1.0),
            ],
        ),
        (
            -up,
            [
                corner(-1.0, -1.0, 1.0),
                corner(1.0, -1.0, 1.0),
                corner(1.0, -1.0, -1.0),
                corner(-1.0, -1.0, -1.0),
            ],
        ),
        (
            forward,
            [
                corner(1.0, -1.0, 1.0),
                corner(-1.0, -1.0, 1.0),
                corner(-1.0, 1.0, 1.0),
                corner(1.0, 1.0, 1.0),
            ],
        ),
        (
            -forward,
            [
                corner(-1.0, -1.0, -1.0),
                corner(1.0, -1.0, -1.0),
                corner(1.0, 1.0, -1.0),
                corner(-1.0, 1.0, -1.0),
            ],
        ),
    ];

    for (normal, face_positions) in faces {
        let start = positions.len() as u32;
        for (uv, position) in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            .into_iter()
            .zip(face_positions)
        {
            positions.push(position.to_array());
            normals.push(normal.to_array());
            uvs.push(uv);
        }
        indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
    }
}

fn build_mesh(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn build_water_mesh(vertices: Vec<WaterSurfaceVertex>, indices: Vec<u32>) -> Mesh {
    let positions = vertices
        .iter()
        .map(|vertex| vertex.position.to_array())
        .collect::<Vec<_>>();
    let uvs = vertices.iter().map(|vertex| vertex.uv).collect::<Vec<_>>();
    let colors = vertices
        .iter()
        .map(|vertex| water_vertex_color(vertex.foam_mask, vertex.flow_class))
        .collect::<Vec<_>>();
    let surface_channels = vertices
        .iter()
        .map(|vertex| [vertex.foam_mask, vertex.flow_class])
        .collect::<Vec<_>>();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, surface_channels);
    mesh.compute_smooth_normals();
    mesh.generate_tangents()
        .expect("water surface UVs should generate valid tangents");
    mesh
}

fn water_vertex_color(foam_mask: f32, flow_class: f32) -> [f32; 4] {
    let foam_mask = foam_mask.clamp(0.0, 1.0);
    let flow_class = flow_class.clamp(0.0, 1.0);
    let deep_tint = Vec3::new(0.82, 0.92, 1.0).lerp(Vec3::new(0.86, 0.95, 1.0), flow_class);
    let shore_tint = Vec3::new(0.95, 1.0, 0.98).lerp(Vec3::new(0.99, 1.0, 1.0), flow_class);
    let tint = deep_tint.lerp(shore_tint, foam_mask * 0.84);
    [tint.x, tint.y, tint.z, 1.0]
}

#[cfg(test)]
mod tests {
    use super::super::shared::DETAIL_CARD_VERTICES;
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values,
            _ => panic!("mesh should expose Float32x3 positions"),
        }
    }

    fn normals(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
            Some(VertexAttributeValues::Float32x3(values)) => values,
            _ => panic!("mesh should expose Float32x3 normals"),
        }
    }

    fn tangents(mesh: &Mesh) -> &[[f32; 4]] {
        match mesh.attribute(Mesh::ATTRIBUTE_TANGENT) {
            Some(VertexAttributeValues::Float32x4(values)) => values,
            _ => panic!("mesh should expose Float32x4 tangents"),
        }
    }

    fn colors(mesh: &Mesh) -> &[[f32; 4]] {
        match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(values)) => values,
            _ => panic!("mesh should expose Float32x4 colors"),
        }
    }

    fn surface_channels(mesh: &Mesh) -> &[[f32; 2]] {
        match mesh.attribute(Mesh::ATTRIBUTE_UV_1) {
            Some(VertexAttributeValues::Float32x2(values)) => values,
            _ => panic!("mesh should expose Float32x2 secondary UVs"),
        }
    }

    fn axis_range(positions: &[[f32; 3]], axis: usize) -> f32 {
        let (min, max) = positions.iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(min, max), position| (min.min(position[axis]), max.max(position[axis])),
        );
        max - min
    }

    fn radial_range(positions: &[[f32; 3]]) -> f32 {
        let (min, max) = positions.iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(min, max), position| {
                let radius = Vec2::new(position[0], position[2]).length();
                (min.min(radius), max.max(radius))
            },
        );
        max - min
    }

    fn u32_indices(mesh: &Mesh) -> &[u32] {
        match mesh.indices() {
            Some(Indices::U32(values)) => values,
            _ => panic!("mesh should expose U32 indices"),
        }
    }

    fn rendered_surface_y_at(mesh: &Mesh, world: Vec3) -> Option<f32> {
        let positions = positions(mesh);
        let point = world.xz();
        u32_indices(mesh)
            .chunks_exact(3)
            .filter_map(|triangle| {
                let a = Vec3::from_array(positions[triangle[0] as usize]);
                let b = Vec3::from_array(positions[triangle[1] as usize]);
                let c = Vec3::from_array(positions[triangle[2] as usize]);
                let edge_ab = b.xz() - a.xz();
                let edge_ac = c.xz() - a.xz();
                let relative = point - a.xz();
                let denominator = edge_ab.perp_dot(edge_ac);
                if denominator.abs() <= 0.000_001 {
                    return None;
                }
                let weight_b = relative.perp_dot(edge_ac) / denominator;
                let weight_c = edge_ab.perp_dot(relative) / denominator;
                let weight_a = 1.0 - weight_b - weight_c;
                (weight_a >= -0.0001 && weight_b >= -0.0001 && weight_c >= -0.0001)
                    .then_some(weight_a * a.y + weight_b * b.y + weight_c * c.y)
            })
            .max_by(f32::total_cmp)
    }

    fn channel_endpoints(spec: IslandWaterVisualSpec) -> (Vec2, Vec2) {
        let mesh = spec.build_mesh();
        let channel_positions = positions(&mesh);
        let center_column = RIVER_CHANNEL_COLUMNS / 2;
        let rotation = Quat::from_rotation_y(spec.rotation_y);
        let start =
            spec.translation + rotation * Vec3::from_array(channel_positions[center_column]);
        let end = spec.translation
            + rotation
                * Vec3::from_array(
                    channel_positions
                        [RIVER_CHANNEL_SEGMENTS * RIVER_CHANNEL_COLUMNS + center_column],
                );
        (Vec2::new(start.x, start.z), Vec2::new(end.x, end.z))
    }

    fn normalized_xz(island: SkyIsland, position: Vec3) -> Vec2 {
        Vec2::new(
            (position.x - island.center.x) / island.half_extents.x,
            (position.z - island.center.z) / island.half_extents.y,
        )
    }

    #[test]
    fn river_channel_surface_is_curved_low_profile_and_deterministic() {
        let first = river_channel_surface_mesh(30.0, 3.0, 0.6, 40_111);
        let second = river_channel_surface_mesh(30.0, 3.0, 0.6, 40_111);
        let first_positions = positions(&first);

        assert_eq!(
            first.count_vertices(),
            (RIVER_CHANNEL_SEGMENTS + 1) * RIVER_CHANNEL_COLUMNS
        );
        assert_eq!(
            u32_indices(&first).len(),
            RIVER_CHANNEL_SEGMENTS * (RIVER_CHANNEL_COLUMNS - 1) * 6
        );
        assert_eq!(first_positions, positions(&second));
        assert_eq!(normals(&first), normals(&second));
        assert_eq!(tangents(&first), tangents(&second));
        assert_eq!(colors(&first), colors(&second));
        assert_eq!(surface_channels(&first), surface_channels(&second));
        assert!(axis_range(first_positions, 2) > 29.0);
        assert!(axis_range(first_positions, 1) < 0.75);

        let center_column = RIVER_CHANNEL_COLUMNS / 2;
        let centerline = (0..=RIVER_CHANNEL_SEGMENTS)
            .map(|segment| first_positions[segment * RIVER_CHANNEL_COLUMNS + center_column][0])
            .collect::<Vec<_>>();
        let centerline_range = centerline
            .iter()
            .copied()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| {
                (min.min(x), max.max(x))
            });
        assert!(centerline_range.1 - centerline_range.0 > 1.0);
    }

    #[test]
    fn water_mesh_density_is_higher_but_bounded() {
        let pond = pond_surface_mesh(12.0, 8.0, 11_149);
        assert_eq!(
            pond.count_vertices(),
            1 + POND_SURFACE_SEGMENTS * POND_SURFACE_RINGS.len()
        );
        assert_eq!(
            u32_indices(&pond).len(),
            POND_SURFACE_SEGMENTS * (3 + (POND_SURFACE_RINGS.len() - 1) * 6)
        );
        assert!(pond.count_vertices() > 65);
        assert!(pond.count_vertices() <= 192);

        let lake = lake_surface_mesh(24.0, 15.0, 31_191);
        assert_eq!(
            lake.count_vertices(),
            1 + LAKE_SURFACE_SEGMENTS * LAKE_SURFACE_RINGS.len()
        );
        assert_eq!(
            u32_indices(&lake).len(),
            LAKE_SURFACE_SEGMENTS * (3 + (LAKE_SURFACE_RINGS.len() - 1) * 6)
        );
        assert!(lake.count_vertices() > 145);
        assert!(lake.count_vertices() <= 320);

        let river = river_channel_surface_mesh(30.0, 4.0, 1.2, 41_211);
        assert_eq!(
            river.count_vertices(),
            (RIVER_CHANNEL_SEGMENTS + 1) * RIVER_CHANNEL_COLUMNS
        );
        assert!(river.count_vertices() > 57);
        assert!(river.count_vertices() <= 256);

        let waterfall = waterfall_ribbon_mesh(16.0, 60.0, 1.4, 33_789);
        assert_eq!(
            waterfall.count_vertices(),
            WATERFALL_RIBBON_COLUMNS * WATERFALL_RIBBON_ROWS
        );
        assert!(waterfall.count_vertices() > 144);
        assert!(waterfall.count_vertices() <= 384);
    }

    #[test]
    fn water_meshes_generate_complete_surface_attributes() {
        let meshes = [
            (pond_surface_mesh(12.0, 8.0, 11_149), 0.0),
            (lake_surface_mesh(24.0, 15.0, 31_191), 0.0),
            (river_channel_surface_mesh(30.0, 4.0, 1.2, 41_211), 0.55),
            (waterfall_ribbon_mesh(16.0, 60.0, 1.4, 33_789), 1.0),
        ];

        for (mesh, expected_flow_class) in &meshes {
            assert_eq!(normals(mesh).len(), mesh.count_vertices());
            assert_eq!(tangents(mesh).len(), mesh.count_vertices());
            assert_eq!(colors(mesh).len(), mesh.count_vertices());
            assert_eq!(surface_channels(mesh).len(), mesh.count_vertices());

            for normal in normals(mesh) {
                let normal = Vec3::from_array(*normal);
                assert!(normal.is_finite());
                assert!((normal.length() - 1.0).abs() < 0.001);
            }
            for tangent in tangents(mesh) {
                let tangent_direction = Vec3::new(tangent[0], tangent[1], tangent[2]);
                assert!(tangent.iter().all(|component| component.is_finite()));
                assert!((tangent_direction.length() - 1.0).abs() < 0.001);
                assert!((tangent[3].abs() - 1.0).abs() < 0.001);
            }
            for color in colors(mesh) {
                assert!(color.iter().all(|channel| (0.0..=1.0).contains(channel)));
                assert_eq!(color[3], 1.0);
            }
            for channels in surface_channels(mesh) {
                assert!((0.0..=1.0).contains(&channels[0]));
                assert_eq!(channels[1], *expected_flow_class);
            }
        }
    }

    #[test]
    fn water_foam_masks_separate_centers_from_edges_and_churn() {
        let pond = pond_surface_mesh(12.0, 8.0, 11_149);
        let pond_channels = surface_channels(&pond);
        assert_eq!(pond_channels[0], [0.0, 0.0]);
        let pond_edge_start = 1 + POND_SURFACE_SEGMENTS * (POND_SURFACE_RINGS.len() - 1);
        assert!(
            pond_channels[pond_edge_start..]
                .iter()
                .all(|channels| channels[0] == 1.0)
        );

        let lake = lake_surface_mesh(24.0, 15.0, 31_191);
        let lake_channels = surface_channels(&lake);
        assert_eq!(lake_channels[0], [0.0, 0.0]);
        let lake_edge_start = 1 + LAKE_SURFACE_SEGMENTS * (LAKE_SURFACE_RINGS.len() - 1);
        assert!(
            lake_channels[lake_edge_start..]
                .iter()
                .all(|channels| channels[0] == 1.0)
        );

        let river = river_channel_surface_mesh(30.0, 4.0, 1.2, 41_211);
        let river_channels = surface_channels(&river);
        let river_center_average = (0..=RIVER_CHANNEL_SEGMENTS)
            .map(|row| river_channels[row * RIVER_CHANNEL_COLUMNS + RIVER_CHANNEL_COLUMNS / 2][0])
            .sum::<f32>()
            / (RIVER_CHANNEL_SEGMENTS + 1) as f32;
        let river_edge_average = (0..=RIVER_CHANNEL_SEGMENTS)
            .flat_map(|row| {
                [
                    river_channels[row * RIVER_CHANNEL_COLUMNS][0],
                    river_channels[row * RIVER_CHANNEL_COLUMNS + RIVER_CHANNEL_COLUMNS - 1][0],
                ]
            })
            .sum::<f32>()
            / ((RIVER_CHANNEL_SEGMENTS + 1) * 2) as f32;
        assert!(river_center_average < 0.20);
        assert!(river_edge_average > river_center_average + 0.75);

        let waterfall = waterfall_ribbon_mesh(16.0, 60.0, 1.4, 33_789);
        let waterfall_channels = surface_channels(&waterfall);
        let waterfall_center_average = (0..WATERFALL_RIBBON_ROWS)
            .map(|row| {
                waterfall_channels[row * WATERFALL_RIBBON_COLUMNS + WATERFALL_RIBBON_COLUMNS / 2][0]
            })
            .sum::<f32>()
            / WATERFALL_RIBBON_ROWS as f32;
        let waterfall_edge_average = (0..WATERFALL_RIBBON_ROWS)
            .flat_map(|row| {
                [
                    waterfall_channels[row * WATERFALL_RIBBON_COLUMNS][0],
                    waterfall_channels
                        [row * WATERFALL_RIBBON_COLUMNS + WATERFALL_RIBBON_COLUMNS - 1][0],
                ]
            })
            .sum::<f32>()
            / (WATERFALL_RIBBON_ROWS * 2) as f32;
        assert!(waterfall_center_average > 0.20);
        assert!(waterfall_edge_average > waterfall_center_average + 0.30);
    }

    #[test]
    fn water_surface_normals_vary_with_coherent_relief() {
        for (mesh, minimum_component_range) in [
            (pond_surface_mesh(12.0, 8.0, 11_149), 0.01),
            (lake_surface_mesh(24.0, 15.0, 31_191), 0.01),
            (river_channel_surface_mesh(30.0, 4.0, 1.2, 41_211), 0.02),
            (waterfall_ribbon_mesh(16.0, 60.0, 1.4, 33_789), 0.04),
        ] {
            let component_range = (0..3)
                .map(|axis| axis_range(normals(&mesh), axis))
                .fold(0.0_f32, f32::max);
            assert!(
                component_range > minimum_component_range,
                "water surface normal variation {component_range:.4} should exceed \
                 {minimum_component_range:.4}"
            );
        }
    }

    #[test]
    fn water_visual_specs_follow_water_tags_and_authored_channel_routes() {
        let dry = SkyIsland::new(
            "launch mesa",
            Vec3::ZERO,
            Vec2::new(56.0, 42.0),
            11.0,
            false,
        );
        assert!(island_water_visual_specs(0, dry).is_empty());

        let pond = SkyIsland::new(
            "landing garden",
            Vec3::ZERO,
            Vec2::new(72.0, 52.0),
            12.0,
            true,
        );
        let pond_specs = island_water_visual_specs(1, pond);
        assert_eq!(pond_specs.len(), 1);
        assert_eq!(pond_specs[0].kind, IslandWaterVisualKind::PondSurface);

        let lake = SkyIsland::new(
            "sapphire basin",
            Vec3::ZERO,
            Vec2::new(100.0, 64.0),
            20.0,
            false,
        );
        let lake_specs = island_water_visual_specs(2, lake);
        assert_eq!(
            lake_specs
                .iter()
                .filter(|spec| spec.kind == IslandWaterVisualKind::RouteLakeSurface)
                .count(),
            1
        );
        assert_eq!(
            lake_specs
                .iter()
                .filter(|spec| spec.kind == IslandWaterVisualKind::RiverChannel)
                .count(),
            1
        );
        assert!(
            lake_specs
                .iter()
                .all(|spec| spec.kind != IslandWaterVisualKind::PondSurface)
        );
        let lake_channel = lake_specs
            .iter()
            .find(|spec| spec.label == "lake inlet channel")
            .expect("lake basin should receive an inlet channel");
        let lake_offset = route_lake_basin_offset(lake);
        let source_offset = Vec2::new(-0.34, 0.28);
        let outlet_offset = lake_offset.lerp(source_offset, 0.42);
        let source = island_water_surface_position(lake, source_offset);
        let outlet = island_water_surface_position(lake, outlet_offset);
        let (channel_start, channel_end) = channel_endpoints(*lake_channel);
        assert!(channel_start.distance(Vec2::new(source.x, source.z)) < 0.15);
        assert!(channel_end.distance(Vec2::new(outlet.x, outlet.z)) < 0.15);

        let waterfall = SkyIsland::new(
            "cloudfall meadow",
            Vec3::ZERO,
            Vec2::new(90.0, 64.0),
            28.0,
            false,
        );
        let waterfall_specs = island_water_visual_specs(3, waterfall);
        for kind in [
            IslandWaterVisualKind::RiverChannel,
            IslandWaterVisualKind::RouteWaterfallRibbon,
            IslandWaterVisualKind::RouteWaterfallMist,
        ] {
            assert_eq!(
                waterfall_specs
                    .iter()
                    .filter(|spec| spec.kind == kind)
                    .count(),
                1
            );
        }
        assert!(
            waterfall_specs
                .iter()
                .all(|spec| spec.kind != IslandWaterVisualKind::PondSurface)
        );
        assert!(
            waterfall_specs
                .iter()
                .any(|spec| spec.label == "waterfall feeder channel")
        );
        let expected_waterfall = route_edge_waterfall_placement(waterfall);
        let ribbon = waterfall_specs
            .iter()
            .find(|spec| spec.kind == IslandWaterVisualKind::RouteWaterfallRibbon)
            .expect("waterfall story should generate a ribbon");
        let mist = waterfall_specs
            .iter()
            .find(|spec| spec.kind == IslandWaterVisualKind::RouteWaterfallMist)
            .expect("waterfall story should generate mist");
        assert_eq!(ribbon.translation, expected_waterfall.ribbon_translation);
        assert_eq!(mist.translation, expected_waterfall.mist_translation);

        let plateau = SkyIsland::new(
            "great sky plateau",
            Vec3::ZERO,
            Vec2::new(230.0, 155.0),
            72.0,
            false,
        );
        let plateau_specs = island_water_visual_specs(4, plateau);
        assert_eq!(
            plateau_specs
                .iter()
                .filter(|spec| spec.kind == IslandWaterVisualKind::PondSurface)
                .count(),
            0
        );
        assert_eq!(
            plateau_specs
                .iter()
                .filter(|spec| spec.kind == IslandWaterVisualKind::RiverChannel)
                .count(),
            4
        );
        assert_eq!(
            plateau_specs
                .iter()
                .filter(|spec| spec.kind == IslandWaterVisualKind::PlateauLakeSurface)
                .count(),
            2
        );
        assert_eq!(
            plateau_specs
                .iter()
                .filter(|spec| spec.kind == IslandWaterVisualKind::PlateauWaterfallRibbon)
                .count(),
            2
        );
        assert_eq!(
            plateau_specs
                .iter()
                .filter(|spec| spec.kind == IslandWaterVisualKind::PlateauWaterfallMist)
                .count(),
            2
        );
        for label in [
            "high shelf spillway",
            "meadow runnel",
            "broken edge outflow",
            "broken edge waterfall",
            "north rim overflow",
            "north rim waterfall",
        ] {
            assert!(plateau_specs.iter().any(|spec| spec.label == label));
        }

        let high_shelf_spillway = *plateau_specs
            .iter()
            .find(|spec| spec.label == "high shelf spillway")
            .expect("plateau should drain its high shelf pool");
        let meadow_runnel = *plateau_specs
            .iter()
            .find(|spec| spec.label == "meadow runnel")
            .expect("plateau should carry water across the meadow");
        let broken_edge_outflow = *plateau_specs
            .iter()
            .find(|spec| spec.label == "broken edge outflow")
            .expect("plateau should drain the low basin toward the broken edge");
        let north_rim_overflow = *plateau_specs
            .iter()
            .find(|spec| spec.label == "north rim overflow")
            .expect("plateau should branch its high-shelf water toward the north rim");
        let (high_start, high_end) = channel_endpoints(high_shelf_spillway);
        let (meadow_start, meadow_end) = channel_endpoints(meadow_runnel);
        let (outflow_start, outflow_end) = channel_endpoints(broken_edge_outflow);
        let (north_start, north_end) = channel_endpoints(north_rim_overflow);
        let high_shelf =
            island_water_surface_position(plateau, IslandPlateauRegion::HighShelf.sample_offset());
        let low_basin =
            island_water_surface_position(plateau, IslandPlateauRegion::LowBasin.sample_offset());
        let broken_edge_lip = island_water_surface_position(
            plateau,
            plateau_waterfall_lip_offset(plateau, IslandPlateauRegion::BrokenEdge),
        );
        let north_rim_lip = island_water_surface_position(
            plateau,
            plateau_waterfall_lip_offset(plateau, IslandPlateauRegion::CliffRim),
        );

        assert!(high_start.distance(Vec2::new(high_shelf.x, high_shelf.z)) < 0.15);
        assert!(north_start.distance(Vec2::new(high_shelf.x, high_shelf.z)) < 0.15);
        assert!(high_end.distance(meadow_start) < 0.15);
        assert!(meadow_end.distance(Vec2::new(low_basin.x, low_basin.z)) < 0.15);
        assert!(meadow_end.distance(outflow_start) < 0.15);
        assert!(outflow_end.distance(Vec2::new(broken_edge_lip.x, broken_edge_lip.z)) < 0.15);
        assert!(north_end.distance(Vec2::new(north_rim_lip.x, north_rim_lip.z)) < 0.15);

        for spec in lake_specs
            .iter()
            .chain(&waterfall_specs)
            .chain(&plateau_specs)
            .filter(|spec| spec.kind == IslandWaterVisualKind::RiverChannel)
        {
            let mesh = spec.build_mesh();
            assert_eq!(
                mesh.count_vertices(),
                (RIVER_CHANNEL_SEGMENTS + 1) * RIVER_CHANNEL_COLUMNS
            );
        }
    }

    #[test]
    fn every_authored_water_story_is_realized_or_explicitly_dry() {
        let route = nau_engine::world::SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let profile = authored_island_art_direction(island.name)
                .expect("every route island should have authored water direction");
            let water = island_water_visual_specs(island_index, island);
            let lake_basins = island_lake_basin_visual_specs(island_index, island);

            match profile.water_story {
                IslandWaterStory::DryWindCarved => {
                    assert!(
                        water.is_empty(),
                        "{} should remain an intentionally dry island",
                        island.name
                    );
                    assert!(lake_basins.is_empty());
                }
                IslandWaterStory::SpringPond
                | IslandWaterStory::MistPool
                | IslandWaterStory::CaveSeep => {
                    assert!(
                        water
                            .iter()
                            .any(|spec| spec.kind == IslandWaterVisualKind::PondSurface),
                        "{} should realize its authored pool",
                        island.name
                    );
                }
                IslandWaterStory::ReflectingBasin | IslandWaterStory::ReedyLake => {
                    assert!(
                        water
                            .iter()
                            .any(|spec| spec.kind == IslandWaterVisualKind::RouteLakeSurface),
                        "{} should realize its authored lake",
                        island.name
                    );
                    assert!(
                        !lake_basins.is_empty(),
                        "{} should receive a structural lake basin",
                        island.name
                    );
                }
                IslandWaterStory::CascadeRun | IslandWaterStory::WaterfallGarden => {
                    assert!(
                        water.iter().any(|spec| {
                            matches!(
                                spec.kind,
                                IslandWaterVisualKind::RiverChannel
                                    | IslandWaterVisualKind::RouteWaterfallRibbon
                                    | IslandWaterVisualKind::PlateauWaterfallRibbon
                            )
                        }),
                        "{} should realize its authored cascade network",
                        island.name
                    );
                }
            }
        }
    }

    #[test]
    fn horizontal_water_footprints_enclose_every_vertex_and_shoreline_margin() {
        const SHORELINE_MARGIN_M: f32 = 1.35;
        let route = nau_engine::world::SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_water_visual_specs(island_index, island) {
                let Some(footprint) = spec.horizontal_footprint() else {
                    continue;
                };
                let mesh = spec.build_mesh();
                let rotation = Quat::from_rotation_y(spec.rotation_y);

                for position in positions(&mesh) {
                    let local = Vec3::from_array(*position);
                    let world = spec.translation + rotation * local;
                    assert!(
                        footprint.contains_world_xz(world.xz(), 0.0),
                        "{} {} footprint misses water vertex {:?}",
                        island.name,
                        spec.label,
                        local
                    );

                    let local_xz = local.xz();
                    let outward = if local_xz.length_squared() > f32::EPSILON {
                        local_xz.normalize()
                    } else {
                        Vec2::X
                    };
                    let shoreline_world = world
                        + rotation
                            * Vec3::new(
                                outward.x * SHORELINE_MARGIN_M,
                                0.0,
                                outward.y * SHORELINE_MARGIN_M,
                            );
                    assert!(
                        footprint.contains_world_xz(shoreline_world.xz(), SHORELINE_MARGIN_M),
                        "{} {} footprint misses its {:.2} m shoreline margin at {:?}",
                        island.name,
                        spec.label,
                        SHORELINE_MARGIN_M,
                        local
                    );
                }
            }
        }
    }

    #[test]
    fn horizontal_water_vertices_clear_the_authored_terrain() {
        let route = nau_engine::world::SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_water_visual_specs(island_index, island)
                .into_iter()
                .filter(|spec| spec.horizontal_footprint().is_some())
            {
                let mesh = spec.build_mesh();
                let rotation = Quat::from_rotation_y(spec.rotation_y);
                for position in positions(&mesh) {
                    let world = spec.translation + rotation * Vec3::from_array(*position);
                    let clearance = world.y - island.mesh_top_y_at(world);
                    assert!(
                        clearance >= HORIZONTAL_WATER_TERRAIN_CLEARANCE_M - 0.001,
                        "{} {} water vertex clearance was {:.3} m",
                        island.name,
                        spec.label,
                        clearance
                    );
                }
            }
        }
    }

    #[test]
    fn horizontal_water_vertices_clear_the_rendered_terrain_mesh() {
        let route = nau_engine::world::SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let terrain_mesh = crate::generated_content::island_terrain_mesh(island_index, island);
            for spec in island_water_visual_specs(island_index, island)
                .into_iter()
                .filter(|spec| spec.horizontal_footprint().is_some())
            {
                let mesh = spec.build_mesh();
                let rotation = Quat::from_rotation_y(spec.rotation_y);
                for position in positions(&mesh) {
                    let world = spec.translation + rotation * Vec3::from_array(*position);
                    let rendered_y = rendered_surface_y_at(&terrain_mesh, world)
                        .expect("horizontal water should project onto its island terrain mesh");
                    let clearance = world.y - rendered_y;
                    assert!(
                        clearance >= 0.03,
                        "{} {} water vertex clears analytic terrain but intersects the rendered \
                         terrain mesh by {:.3} m",
                        island.name,
                        spec.label,
                        -clearance
                    );
                }
            }
        }
    }

    #[test]
    fn horizontal_water_triangles_face_upward() {
        for mesh in [
            pond_surface_mesh(12.0, 8.0, 11_149),
            lake_surface_mesh(24.0, 15.0, 31_191),
            river_channel_surface_mesh(30.0, 4.0, 1.2, 41_211),
        ] {
            let positions = positions(&mesh);
            for triangle in u32_indices(&mesh).chunks_exact(3) {
                let a = Vec3::from_array(positions[triangle[0] as usize]);
                let b = Vec3::from_array(positions[triangle[1] as usize]);
                let c = Vec3::from_array(positions[triangle[2] as usize]);
                assert!(
                    (b - a).cross(c - a).y > 0.0,
                    "horizontal water triangle should face upward"
                );
            }
        }
    }

    #[test]
    fn cave_seep_pool_is_player_readable() {
        let route = nau_engine::world::SkyRoute::default();
        let (island_index, island) = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .find(|(_, island)| island.name == "underbridge cay")
            .expect("underbridge cay exists");
        let pool = island_water_visual_specs(island_index, island)
            .into_iter()
            .find(|spec| spec.label == "cave seep pool")
            .expect("underbridge cay should have a cave seep pool");
        let IslandWaterVisualMesh::PondSurface { radius_x, radius_z } = pool.mesh else {
            panic!("cave seep should use a pond surface");
        };

        assert!(radius_x >= 3.0);
        assert!(radius_z >= 1.3);
        assert!(pool.translation.z > island.center.z);
        assert!(
            pool.translation.y - island.mesh_top_y_at(pool.translation)
                >= HORIZONTAL_WATER_TERRAIN_CLEARANCE_M - 0.001
        );
    }

    #[test]
    fn mist_pool_clears_the_fog_cairn_visual_footprint() {
        use super::super::hero_landmarks::island_hero_landmark_spec;

        let route = nau_engine::world::SkyRoute::default();
        let (island_index, island) = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .find(|(_, island)| island.name == "mist stepping stone")
            .expect("mist stepping stone exists");
        let pool = island_water_visual_specs(island_index, island)
            .into_iter()
            .find(|spec| spec.label == "mist pool")
            .expect("mist stepping stone should have a mist pool");
        let IslandWaterVisualMesh::PondSurface { radius_x, radius_z } = pool.mesh else {
            panic!("mist pool should use a pond surface");
        };
        let hero = island_hero_landmark_spec(island_index, island).expect("fog cairn hero");
        let separation = (pool.translation - hero.translation).abs();
        let clearance = 0.25;

        assert!(
            separation.x >= hero.visual_half_extents.x + radius_x + clearance
                || separation.z >= hero.visual_half_extents.z + radius_z + clearance,
            "mist pool should not be hidden beneath the fog cairn footprint"
        );
    }

    #[test]
    fn great_plateau_artifacts_form_large_authored_precincts() {
        let plateau = SkyIsland::new(
            "great sky plateau",
            Vec3::new(12.0, 80.0, -18.0),
            Vec2::new(230.0, 155.0),
            72.0,
            false,
        );
        let specs = island_artifact_visual_specs(31, plateau);

        assert_eq!(specs.len(), 6);
        for kind in [
            IslandArtifactVisualKind::AncientStairRun,
            IslandArtifactVisualKind::RetainingWall,
            IslandArtifactVisualKind::GlyphSlab,
            IslandArtifactVisualKind::BridgeFragment,
            IslandArtifactVisualKind::ReedPatch,
            IslandArtifactVisualKind::PebbleField,
        ] {
            assert_eq!(specs.iter().filter(|spec| spec.kind == kind).count(), 1);
        }

        let stair = specs
            .iter()
            .find(|spec| spec.kind == IslandArtifactVisualKind::AncientStairRun)
            .expect("plateau should have an arrival stair");
        let wall = specs
            .iter()
            .find(|spec| spec.kind == IslandArtifactVisualKind::RetainingWall)
            .expect("plateau should have high-shelf retaining stonework");
        let bridge = specs
            .iter()
            .find(|spec| spec.kind == IslandArtifactVisualKind::BridgeFragment)
            .expect("plateau should have a broken-edge bridge fragment");
        let reeds = specs
            .iter()
            .find(|spec| spec.kind == IslandArtifactVisualKind::ReedPatch)
            .expect("plateau should have a low-basin reed bed");
        let pebbles = specs
            .iter()
            .find(|spec| spec.kind == IslandArtifactVisualKind::PebbleField)
            .expect("plateau should have a low-basin pebble shore");

        assert!(normalized_xz(plateau, stair.translation).x < -0.30);
        assert!(normalized_xz(plateau, wall.translation).x < -0.50);
        assert!(normalized_xz(plateau, bridge.translation).x > 0.40);
        assert_eq!(
            plateau.plateau_region_at_normalized_offset(normalized_xz(plateau, reeds.translation)),
            Some(IslandPlateauRegion::LowBasin)
        );
        assert_eq!(
            plateau
                .plateau_region_at_normalized_offset(normalized_xz(plateau, pebbles.translation)),
            Some(IslandPlateauRegion::LowBasin)
        );

        assert!(matches!(
            stair.mesh,
            IslandArtifactVisualMesh::AncientStairRun {
                length,
                width,
                rise
            } if length >= 28.0 && width >= 7.0 && rise >= 6.0
        ));
        assert!(matches!(
            wall.mesh,
            IslandArtifactVisualMesh::RetainingWall {
                length,
                height,
                depth
            } if length >= 34.0 && height >= 7.0 && depth >= 3.0
        ));
        assert!(matches!(
            bridge.mesh,
            IslandArtifactVisualMesh::BridgeFragment {
                length,
                width,
                thickness
            } if length >= 27.0 && width >= 7.0 && thickness >= 1.5
        ));
        assert!(matches!(
            reeds.mesh,
            IslandArtifactVisualMesh::ReedPatch { radius, height }
                if radius >= 7.0 && height >= 4.0
        ));
        assert!(matches!(
            pebbles.mesh,
            IslandArtifactVisualMesh::PebbleField { radius } if radius >= 9.0
        ));
    }

    #[test]
    fn artifact_collision_policies_cover_every_family() {
        assert_eq!(IslandArtifactVisualKind::ALL.len(), 7);
        for kind in IslandArtifactVisualKind::ALL {
            let expected = match kind {
                IslandArtifactVisualKind::RetainingWall | IslandArtifactVisualKind::GlyphSlab => {
                    IslandArtifactCollisionPolicy::SolidAabb
                }
                IslandArtifactVisualKind::AncientStairRun
                | IslandArtifactVisualKind::BridgeFragment => {
                    IslandArtifactCollisionPolicy::NonBlockingRouteAffordance
                }
                IslandArtifactVisualKind::BannerStrips
                | IslandArtifactVisualKind::PebbleField
                | IslandArtifactVisualKind::ReedPatch => {
                    IslandArtifactCollisionPolicy::NonBlockingDecoration
                }
            };
            assert_eq!(kind.collision_policy(), expected);
        }
    }

    #[test]
    fn solid_artifact_aabbs_are_grounded_and_bounded() {
        let plateau = SkyIsland::new(
            "great sky plateau",
            Vec3::new(12.0, 80.0, -18.0),
            Vec2::new(230.0, 155.0),
            72.0,
            false,
        );
        let specs = island_artifact_visual_specs(31, plateau);

        for spec in specs {
            let aabb = spec.solid_world_aabb();
            assert_eq!(
                aabb.is_some(),
                spec.kind.collision_policy() == IslandArtifactCollisionPolicy::SolidAabb,
                "{} must follow its explicit collision policy",
                spec.kind.visual_name()
            );
            let Some((center, half_extents)) = aabb else {
                continue;
            };
            assert!(center.is_finite());
            assert!(half_extents.is_finite());
            assert!(half_extents.cmpgt(Vec3::ZERO).all());
            assert!((center.y - half_extents.y - spec.translation.y).abs() < 0.001);
            assert!(plateau.contains_horizontal(center));
            assert!(half_extents.x < plateau.half_extents.x);
            assert!(half_extents.z < plateau.half_extents.y);
        }
    }

    #[test]
    fn artifact_visual_specs_are_deterministic() {
        let island = SkyIsland::new(
            "broken stair",
            Vec3::new(12.0, 40.0, -8.0),
            Vec2::new(22.0, 15.0),
            12.0,
            false,
        );
        let first = island_artifact_visual_specs(7, island);
        let second = island_artifact_visual_specs(7, island);

        assert!(!first.is_empty());
        assert_eq!(first.len(), second.len());
        for (first, second) in first.into_iter().zip(second) {
            assert_eq!(first.kind, second.kind);
            assert_eq!(first.kind.visual_name(), first.label);
            assert_eq!(first.kind.label(), second.kind.label());
            assert_eq!(first.translation, second.translation);
            assert_eq!(first.rotation_y, second.rotation_y);
            assert_eq!(first.material, second.material);

            let first_mesh = first.build_mesh();
            let second_mesh = second.build_mesh();
            assert_eq!(positions(&first_mesh), positions(&second_mesh));
        }
    }

    #[test]
    fn artifact_mesh_generators_preserve_high_fidelity_density() {
        let stair = artifact_stair_run_mesh(9.0, 2.4, 2.2, 61_111);
        assert_eq!(stair.count_vertices(), ARTIFACT_STAIR_STEP_COUNT * 24);
        assert!(axis_range(positions(&stair), 1) > 2.0);
        assert!(axis_range(positions(&stair), 2) > 8.0);

        let wall = artifact_retaining_wall_mesh(8.0, 2.4, 1.1, 61_222);
        assert_eq!(wall.count_vertices(), ARTIFACT_RETAINING_WALL_SEGMENTS * 24);
        assert!(axis_range(positions(&wall), 0) > 7.0);
        assert!(axis_range(positions(&wall), 1) > 1.8);

        let glyph = artifact_glyph_slab_mesh(2.2, 4.0, 0.7, 61_333);
        assert_eq!(
            glyph.count_vertices(),
            (1 + ARTIFACT_GLYPH_STROKE_COUNT) * 24
        );
        assert!(axis_range(positions(&glyph), 1) > 3.8);

        let bridge = artifact_bridge_fragment_mesh(10.0, 2.6, 0.6, 61_444);
        assert!(bridge.count_vertices() >= ARTIFACT_BRIDGE_FRAGMENT_COUNT * 18);
        assert!(axis_range(positions(&bridge), 0) > 9.0);
        assert!(axis_range(positions(&bridge), 1) > 0.8);

        let banners = artifact_banner_strips_mesh(3.8, 2.8, 61_555);
        assert_eq!(
            banners.count_vertices(),
            24 + ARTIFACT_BANNER_STRIP_COUNT * DETAIL_CARD_VERTICES
        );
        assert!(axis_range(positions(&banners), 1) > 1.5);

        let pebbles = artifact_pebble_field_mesh(2.2, 61_666);
        assert!(pebbles.count_vertices() >= ARTIFACT_PEBBLE_COUNT * 30);
        assert!(radial_range(positions(&pebbles)) > 1.5);

        let reeds = artifact_reed_patch_mesh(1.8, 1.6, 61_777);
        assert_eq!(
            reeds.count_vertices(),
            ARTIFACT_REED_COUNT * DETAIL_CARD_VERTICES
        );
        assert!(axis_range(positions(&reeds), 1) > 0.8);
        assert!(radial_range(positions(&reeds)) > 1.0);
    }
}
