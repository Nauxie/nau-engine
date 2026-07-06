use super::{
    super::random_unit,
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::{
    IslandLandmarkRole, IslandPlateauRegion, IslandRouteRole, IslandTerrainArchetype,
    IslandWaterFeature, SkyIsland,
};

pub(crate) const ROUTE_CAIRN_STONE_COUNT: usize = 5;
pub(crate) const RUIN_ARCH_STONE_COUNT: usize = 11;
pub(crate) const LAUNCH_BEACON_CRYSTAL_COUNT: usize = 4;
pub(crate) const LANDING_GARDEN_MARKER_SEGMENTS: usize = 12;
pub(crate) const GARDEN_RING_SEGMENTS: usize = 36;
pub(crate) const GARDEN_RING_BANDS: usize = 4;
pub(crate) const LAKE_BASIN_RIM_SEGMENTS: usize = 48;
pub(crate) const LAKE_BASIN_RIM_BANDS: usize = 5;
pub(crate) const POND_SURFACE_SEGMENTS: usize = 32;
pub(crate) const LAKE_SURFACE_SEGMENTS: usize = 48;
pub(crate) const WATERFALL_RIBBON_COLUMNS: usize = 8;
pub(crate) const WATERFALL_RIBBON_ROWS: usize = 18;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IslandWaterVisualKind {
    PondSurface,
    PlateauLakeSurface,
    PlateauWaterfallRibbon,
    PlateauWaterfallMist,
    RouteWaterfallRibbon,
    RouteWaterfallMist,
    RouteLakeSurface,
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
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IslandWaterVisualMesh {
    PondSurface { radius_x: f32, radius_z: f32 },
    LakeSurface { radius_x: f32, radius_z: f32 },
    WaterfallRibbon { width: f32, height: f32, depth: f32 },
    WaterfallMist { radius: f32, height: f32 },
}

impl IslandWaterVisualMesh {
    pub(crate) fn build(self, seed: u32) -> Mesh {
        match self {
            Self::PondSurface { radius_x, radius_z } => pond_surface_mesh(radius_x, radius_z, seed),
            Self::LakeSurface { radius_x, radius_z } => lake_surface_mesh(radius_x, radius_z, seed),
            Self::WaterfallRibbon {
                width,
                height,
                depth,
            } => waterfall_ribbon_mesh(width, height, depth, seed),
            Self::WaterfallMist { radius, height } => waterfall_mist_mesh(radius, height, seed),
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

impl IslandArtifactVisualKind {
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
    let pond_offset = if island.is_target {
        Vec2::new(-0.34, 0.18)
    } else {
        Vec2::new(0.18, 0.28)
    };
    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::PondSurface,
        label: "pond surface",
        translation: island_water_surface_position(island, pond_offset) + Vec3::Y * 0.055,
        rotation_y: 0.0,
        wind_phase: 3.4,
        wind_motion_scale: 1.0,
        mesh: IslandWaterVisualMesh::PondSurface {
            radius_x: island.half_extents.x * 0.12,
            radius_z: island.half_extents.y * 0.08,
        },
        seed: 11_000 + island_index as u32 * 149,
    });

    if island.is_great_plateau_anchor() {
        if let Some(low_basin) = island.plateau_region_position(IslandPlateauRegion::LowBasin) {
            specs.push(IslandWaterVisualSpec {
                kind: IslandWaterVisualKind::PlateauLakeSurface,
                label: "low basin lake",
                translation: low_basin + Vec3::Y * 0.08,
                rotation_y: 0.22,
                wind_phase: 4.7,
                wind_motion_scale: 1.45,
                mesh: IslandWaterVisualMesh::LakeSurface {
                    radius_x: island.half_extents.x * 0.24,
                    radius_z: island.half_extents.y * 0.17,
                },
                seed: 31_000 + island_index as u32 * 191,
            });
        }
        if let Some(high_shelf) = island.plateau_region_position(IslandPlateauRegion::HighShelf) {
            specs.push(IslandWaterVisualSpec {
                kind: IslandWaterVisualKind::PlateauLakeSurface,
                label: "high shelf pool",
                translation: high_shelf + Vec3::Y * 0.08,
                rotation_y: -0.18,
                wind_phase: 5.2,
                wind_motion_scale: 1.25,
                mesh: IslandWaterVisualMesh::LakeSurface {
                    radius_x: island.half_extents.x * 0.13,
                    radius_z: island.half_extents.y * 0.09,
                },
                seed: 32_000 + island_index as u32 * 193,
            });
        }
        for waterfall in [
            PlateauWaterfallFeatureSpec {
                region: IslandPlateauRegion::CliffRim,
                ribbon_label: "north rim waterfall",
                mist_label: "north rim waterfall mist",
                width_scale: 0.18,
                index: 0,
            },
            PlateauWaterfallFeatureSpec {
                region: IslandPlateauRegion::BrokenEdge,
                ribbon_label: "broken edge waterfall",
                mist_label: "broken edge waterfall mist",
                width_scale: 0.14,
                index: 1,
            },
        ] {
            push_plateau_waterfall_specs(&mut specs, island_index, island, waterfall);
        }
    }

    if island.world_tags.water_feature == IslandWaterFeature::WaterfallSource
        && !island.is_great_plateau_anchor()
    {
        push_route_edge_waterfall_specs(&mut specs, island_index, island);
    }
    if island.world_tags.water_feature == IslandWaterFeature::LakeBasin
        && !island.is_great_plateau_anchor()
    {
        push_route_lake_surface_specs(&mut specs, island_index, island);
    }

    specs
}

pub(crate) fn island_lake_basin_visual_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandLakeBasinVisualSpec> {
    let mut specs = Vec::new();

    if island.is_great_plateau_anchor() {
        if let Some(low_basin) = island.plateau_region_position(IslandPlateauRegion::LowBasin) {
            specs.push(IslandLakeBasinVisualSpec {
                label: "low basin lake basin",
                translation: low_basin + Vec3::Y * 0.035,
                rotation_y: 0.22,
                radius_x: island.half_extents.x * 0.255,
                radius_z: island.half_extents.y * 0.185,
                rim_width: island.half_extents.min_element() * 0.035,
                rim_height: island.thickness * 0.025,
                seed: 35_000 + island_index as u32 * 211,
            });
        }
        if let Some(high_shelf) = island.plateau_region_position(IslandPlateauRegion::HighShelf) {
            specs.push(IslandLakeBasinVisualSpec {
                label: "high shelf lake basin",
                translation: high_shelf + Vec3::Y * 0.035,
                rotation_y: -0.18,
                radius_x: island.half_extents.x * 0.145,
                radius_z: island.half_extents.y * 0.105,
                rim_width: island.half_extents.min_element() * 0.025,
                rim_height: island.thickness * 0.021,
                seed: 36_000 + island_index as u32 * 213,
            });
        }
    }

    if island.world_tags.water_feature == IslandWaterFeature::LakeBasin
        && !island.is_great_plateau_anchor()
    {
        let label = if island.terrain_archetype == IslandTerrainArchetype::SapphireBasin {
            "sapphire lake basin"
        } else {
            "route lake basin"
        };
        specs.push(IslandLakeBasinVisualSpec {
            label,
            translation: island_water_surface_position(island, route_lake_basin_offset(island))
                + Vec3::Y * 0.035,
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
        &[0.48, 1.0],
        0.15,
        0.012,
    )
}

pub(crate) fn lake_surface_mesh(radius_x: f32, radius_z: f32, seed: u32) -> Mesh {
    irregular_water_surface_mesh(
        radius_x,
        radius_z,
        seed,
        LAKE_SURFACE_SEGMENTS,
        &[0.34, 0.68, 1.0],
        0.20,
        0.030,
    )
}

pub(crate) fn waterfall_ribbon_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity(WATERFALL_RIBBON_COLUMNS * WATERFALL_RIBBON_ROWS);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices =
        Vec::with_capacity((WATERFALL_RIBBON_COLUMNS - 1) * (WATERFALL_RIBBON_ROWS - 1) * 6);

    for row in 0..WATERFALL_RIBBON_ROWS {
        let v = row as f32 / (WATERFALL_RIBBON_ROWS - 1) as f32;
        let y = height * (0.5 - v);
        let fall_waver = (v * 18.0 + seed as f32 * 0.019).sin();
        for column in 0..WATERFALL_RIBBON_COLUMNS {
            let u = column as f32 / (WATERFALL_RIBBON_COLUMNS - 1) as f32;
            let centered_u = u - 0.5;
            let strand_noise = random_unit(seed, column as u32, 1_021) - 0.5;
            let taper = 1.0 - v * 0.18 + (v * std::f32::consts::PI).sin() * 0.10;
            let x = centered_u * width * taper
                + (fall_waver + column as f32 * 0.19).sin() * width * 0.018;
            let z =
                (fall_waver * 0.55 + centered_u * 2.1).sin() * depth + strand_noise * depth * 0.28;
            let alpha_lane = (1.0 - centered_u.abs() * 1.35).max(0.0);
            let normal_lift = 0.05
                + alpha_lane * 0.34
                + (v * std::f32::consts::PI).sin() * 0.16
                + (fall_waver + column as f32 * 0.41).cos().abs() * 0.10;
            let normal = Vec3::new(
                (strand_noise * 0.24 + fall_waver * 0.08).clamp(-0.32, 0.32),
                normal_lift,
                1.0 + centered_u.abs() * 0.18,
            )
            .normalize();

            positions.push([x, y, z]);
            normals.push(normal.to_array());
            uvs.push([u, v * (1.0 + alpha_lane * 0.08)]);
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

    build_mesh(positions, normals, uvs, indices)
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
    let mut positions = Vec::with_capacity(1 + segments * rings.len());
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(segments * (3 + rings.len().saturating_sub(1) * 6));

    positions.push([0.0, 0.0, 0.0]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 0.5]);

    for (ring_index, ring) in rings.iter().copied().enumerate() {
        for segment in 0..segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let edge = 1.0
                + (random_unit(seed, segment as u32 + ring_index as u32 * 31, 907) - 0.5)
                    * edge_noise_scale
                    * ring
                + 0.035 * (angle * 5.0 + seed as f32 * 0.011).sin();
            let ripple = ((angle * 4.0 + seed as f32 * 0.017).sin()
                + (angle * 9.0 + ring_index as f32 * 0.71).sin() * 0.35)
                * ripple_scale
                * ring;
            let x = angle.cos() * radius_x * ring * edge;
            let z = angle.sin() * radius_z * ring * edge;

            positions.push([x, ripple, z]);
            normals.push(Vec3::Y.to_array());
            uvs.push([
                0.5 + angle.cos() * ring * 0.5,
                0.5 + angle.sin() * ring * 0.5,
            ]);
        }
    }

    let ring_vertex_index = |ring_index: usize, segment: usize| -> u32 {
        1 + (ring_index * segments + segment % segments) as u32
    };

    for segment in 0..segments {
        indices.extend([
            0,
            ring_vertex_index(0, segment),
            ring_vertex_index(0, segment + 1),
        ]);
    }
    for ring_index in 0..rings.len().saturating_sub(1) {
        for segment in 0..segments {
            indices.extend([
                ring_vertex_index(ring_index, segment),
                ring_vertex_index(ring_index + 1, segment),
                ring_vertex_index(ring_index, segment + 1),
                ring_vertex_index(ring_index, segment + 1),
                ring_vertex_index(ring_index + 1, segment),
                ring_vertex_index(ring_index + 1, segment + 1),
            ]);
        }
    }

    build_mesh(positions, normals, uvs, indices)
}

fn push_plateau_waterfall_specs(
    specs: &mut Vec<IslandWaterVisualSpec>,
    island_index: usize,
    island: SkyIsland,
    waterfall: PlateauWaterfallFeatureSpec,
) {
    let Some(surface) = island.plateau_region_position(waterfall.region) else {
        return;
    };
    let sample = waterfall.region.sample_offset();
    let outward = Vec2::new(sample.x, sample.y).normalize_or_zero();
    let outward = if outward.length_squared() > 0.001 {
        outward
    } else {
        Vec2::Y
    };
    let yaw = outward.x.atan2(outward.y);
    let height = island.thickness * 0.84;
    let width = island.half_extents.min_element() * waterfall.width_scale;
    let outward3 = Vec3::new(outward.x, 0.0, outward.y);
    let seed_base = 33_000 + island_index as u32 * 197 + waterfall.index * 1_009;

    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::PlateauWaterfallRibbon,
        label: waterfall.ribbon_label,
        translation: surface + outward3 * 6.0 - Vec3::Y * (height * 0.48),
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
        translation: surface + outward3 * 9.0 - Vec3::Y * (height * 0.94),
        rotation_y: yaw,
        wind_phase: 6.8 + waterfall.index as f32 * 0.9,
        wind_motion_scale: 1.55,
        mesh: IslandWaterVisualMesh::WaterfallMist {
            radius: width * 0.42,
            height: island.thickness * 0.08,
        },
        seed: seed_base + 503,
    });
}

fn push_route_lake_surface_specs(
    specs: &mut Vec<IslandWaterVisualSpec>,
    island_index: usize,
    island: SkyIsland,
) {
    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::RouteLakeSurface,
        label: "route lake surface",
        translation: island_water_surface_position(island, route_lake_basin_offset(island))
            + Vec3::Y * 0.075,
        rotation_y: -0.08,
        wind_phase: 5.8 + island_index as f32 * 0.037,
        wind_motion_scale: 1.32,
        mesh: IslandWaterVisualMesh::LakeSurface {
            radius_x: island.half_extents.x * 0.18,
            radius_z: island.half_extents.y * 0.13,
        },
        seed: 38_000 + island_index as u32 * 219,
    });
}

fn push_route_edge_waterfall_specs(
    specs: &mut Vec<IslandWaterVisualSpec>,
    island_index: usize,
    island: SkyIsland,
) {
    let source_offset = Vec2::new(-0.42, 0.16);
    let edge_offset = Vec2::new(-0.76, 0.28);
    let surface = island_water_surface_position(island, edge_offset);
    let outward = edge_offset.normalize_or_zero();
    let yaw = outward.x.atan2(outward.y);
    let outward3 = Vec3::new(outward.x, 0.0, outward.y);
    let height = (island.thickness * 1.25).clamp(18.0, 34.0);
    let width = island.half_extents.min_element() * 0.16;
    let source = island_water_surface_position(island, source_offset);
    let source_drop = (source.y - surface.y).max(0.0) * 0.12;
    let seed_base = 39_000 + island_index as u32 * 223;

    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::RouteWaterfallRibbon,
        label: "route edge waterfall",
        translation: surface + outward3 * 4.5 - Vec3::Y * (height * 0.46 + source_drop),
        rotation_y: yaw,
        wind_phase: 7.4,
        wind_motion_scale: 1.65,
        mesh: IslandWaterVisualMesh::WaterfallRibbon {
            width,
            height,
            depth: width * 0.07,
        },
        seed: seed_base,
    });
    specs.push(IslandWaterVisualSpec {
        kind: IslandWaterVisualKind::RouteWaterfallMist,
        label: "route edge mist",
        translation: surface + outward3 * 7.5 - Vec3::Y * (height * 0.90 + source_drop),
        rotation_y: yaw,
        wind_phase: 8.1,
        wind_motion_scale: 1.45,
        mesh: IslandWaterVisualMesh::WaterfallMist {
            radius: width * 0.48,
            height: height * 0.10,
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
