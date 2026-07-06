use super::{
    super::random_unit,
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::{
    IslandPlateauRegion, IslandTerrainArchetype, IslandWaterFeature, SkyIsland,
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
    AncientGlyphSlab,
    BrokenColumnCluster,
    BridgeFragment,
    WindVane,
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
            Self::AncientGlyphSlab => "first_expedition_glyph_slab",
            Self::BrokenColumnCluster => "first_expedition_broken_columns",
            Self::BridgeFragment => "first_expedition_bridge_fragment",
            Self::WindVane => "first_expedition_wind_vane",
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
            Self::AncientGlyphSlab => "first expedition glyph slab",
            Self::BrokenColumnCluster => "first expedition broken columns",
            Self::BridgeFragment => "first expedition bridge fragment",
            Self::WindVane => "first expedition wind vane",
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
            FirstExpeditionSilhouetteMesh::GlyphSlab {
                width,
                height,
                depth,
            } => carved_glyph_slab_mesh(width, height, depth, self.seed),
            FirstExpeditionSilhouetteMesh::BrokenColumns { radius, height } => {
                broken_column_cluster_mesh(radius, height, self.seed)
            }
            FirstExpeditionSilhouetteMesh::BridgeFragment {
                length,
                width,
                height,
            } => bridge_fragment_mesh(length, width, height, self.seed),
            FirstExpeditionSilhouetteMesh::WindVane { height, span } => {
                wind_vane_mesh(height, span, self.seed)
            }
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
    GlyphSlab {
        width: f32,
        height: f32,
        depth: f32,
    },
    BrokenColumns {
        radius: f32,
        height: f32,
    },
    BridgeFragment {
        length: f32,
        width: f32,
        height: f32,
    },
    WindVane {
        height: f32,
        span: f32,
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
            vec![
                FirstExpeditionSilhouetteSpec {
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
                },
                FirstExpeditionSilhouetteSpec {
                    kind: FirstExpeditionSilhouetteKind::AncientGlyphSlab,
                    label: "ancient glyph slab",
                    translation: island_water_surface_position(island, Vec2::new(0.28, -0.18))
                        + Vec3::Y * 0.10,
                    rotation_y: 0.38,
                    mesh: FirstExpeditionSilhouetteMesh::GlyphSlab {
                        width: 5.0,
                        height: 7.2,
                        depth: 0.72,
                    },
                    seed: seed.wrapping_add(37),
                },
            ]
        }
        "cloud gate" => {
            let height = 15.5;
            vec![
                FirstExpeditionSilhouetteSpec {
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
                },
                FirstExpeditionSilhouetteSpec {
                    kind: FirstExpeditionSilhouetteKind::BrokenColumnCluster,
                    label: "broken column cluster",
                    translation: island_water_surface_position(island, Vec2::new(-0.30, 0.20))
                        + Vec3::Y * 0.06,
                    rotation_y: -0.48,
                    mesh: FirstExpeditionSilhouetteMesh::BrokenColumns {
                        radius: 0.92,
                        height: 7.4,
                    },
                    seed: seed.wrapping_add(53),
                },
            ]
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
        "mist stepping stone" => vec![FirstExpeditionSilhouetteSpec {
            kind: FirstExpeditionSilhouetteKind::BridgeFragment,
            label: "broken bridge fragment",
            translation: island_water_surface_position(island, Vec2::new(0.02, 0.22))
                + Vec3::Y * 0.08,
            rotation_y: 0.58,
            mesh: FirstExpeditionSilhouetteMesh::BridgeFragment {
                length: 9.5,
                width: 2.4,
                height: 1.15,
            },
            seed,
        }],
        "east windchain" => vec![FirstExpeditionSilhouetteSpec {
            kind: FirstExpeditionSilhouetteKind::WindVane,
            label: "stone wind vane",
            translation: island_water_surface_position(island, Vec2::new(0.34, -0.04))
                + Vec3::Y * 0.06,
            rotation_y: 0.72,
            mesh: FirstExpeditionSilhouetteMesh::WindVane {
                height: 7.8,
                span: 4.8,
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

fn carved_glyph_slab_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let width = width.max(1.0);
    let height = height.max(1.0);
    let depth = depth.max(0.18);

    append_weathered_block(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, height * 0.50, 0.0),
        Vec3::new(width * 0.50, height * 0.50, depth * 0.50),
        0.0,
        seed,
        0.080,
    );

    for chip in 0..4 {
        let side = if chip % 2 == 0 { -1.0 } else { 1.0 };
        let t = chip as f32 / 3.0;
        append_weathered_block(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                side * width * (0.20 + random_unit(seed, chip, 2_101) * 0.23),
                height * (0.10 + t * 0.25),
                -depth * 0.58,
            ),
            Vec3::new(width * 0.035, height * 0.12, depth * 0.10),
            side * 0.16,
            seed.wrapping_add(chip * 31),
            0.045,
        );
    }

    for stroke in 0..9 {
        let row = stroke / 3;
        let column = stroke % 3;
        let horizontal = stroke % 2 == 0;
        let x = (column as f32 - 1.0) * width * 0.20;
        let y = height * (0.28 + row as f32 * 0.18);
        let yaw = if horizontal { 0.0 } else { 0.18 };
        let half_extents = if horizontal {
            Vec3::new(width * 0.125, height * 0.020, depth * 0.085)
        } else {
            Vec3::new(width * 0.026, height * 0.105, depth * 0.085)
        };
        append_weathered_block(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(x, y, -depth * 0.66),
            half_extents,
            yaw,
            seed.wrapping_add(stroke as u32 * 43 + 503),
            0.020,
        );
    }

    for rubble in 0..4 {
        let side = if rubble % 2 == 0 { -1.0 } else { 1.0 };
        let x = side * width * (0.18 + random_unit(seed, rubble, 2_231) * 0.22);
        let z = -depth * (0.48 + random_unit(seed, rubble, 2_239) * 0.18);
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                x,
                height * (0.05 + random_unit(seed, rubble, 2_241) * 0.05),
                z,
            ),
            Vec3::new(
                width * (0.055 + random_unit(seed, rubble, 2_251) * 0.030),
                height * (0.030 + random_unit(seed, rubble, 2_257) * 0.020),
                depth * (0.20 + random_unit(seed, rubble, 2_263) * 0.08),
            ),
            LANDMARK_LOBE_LATITUDE_SEGMENTS,
            LANDMARK_LOBE_LONGITUDE_SEGMENTS,
            seed.wrapping_add(rubble * 79 + 911),
            0.24,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

fn broken_column_cluster_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let radius = radius.max(0.25);
    let height = height.max(1.5);

    for column in 0..3 {
        let phase =
            column as f32 / 3.0 * std::f32::consts::TAU + random_unit(seed, column, 2_401) * 0.32;
        let center_x = phase.cos() * radius * 0.90;
        let center_z = phase.sin() * radius * 0.72;
        let column_height = height * (0.44 + random_unit(seed, column, 2_409) * 0.36);
        let drum_count = 3 + column;
        let drum_height = column_height / drum_count as f32;

        for drum in 0..drum_count {
            let t = drum as f32 / drum_count as f32;
            let lean = Vec2::new(phase.cos(), phase.sin())
                * radius
                * (random_unit(seed, column * 17 + drum, 2_421) - 0.5)
                * 0.18
                * t;
            append_ellipsoid_lobe(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                Vec3::new(
                    center_x + lean.x,
                    drum_height * (0.56 + drum as f32),
                    center_z + lean.y,
                ),
                Vec3::new(
                    radius * (0.48 + random_unit(seed, drum, 2_431) * 0.10),
                    drum_height * 0.48,
                    radius * (0.40 + random_unit(seed, drum, 2_437) * 0.10),
                ),
                LANDMARK_LOBE_LATITUDE_SEGMENTS,
                LANDMARK_LOBE_LONGITUDE_SEGMENTS,
                seed.wrapping_add(column * 101 + drum * 17),
                0.16,
            );
        }

        append_weathered_block(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(center_x * 1.18, radius * 0.16, center_z * 1.20),
            Vec3::new(radius * 0.72, radius * 0.14, radius * 0.30),
            phase,
            seed.wrapping_add(column * 67 + 811),
            0.08,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

fn bridge_fragment_mesh(length: f32, width: f32, height: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let length = length.max(2.0);
    let width = width.max(0.5);
    let height = height.max(0.2);

    for plank in 0..4 {
        let lane = plank as f32 / 3.0 - 0.5;
        let broken_scale = 0.72 + random_unit(seed, plank, 2_701) * 0.28;
        append_weathered_block(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                (random_unit(seed, plank, 2_709) - 0.5) * length * 0.10,
                height * (0.34 + lane.abs() * 0.10),
                lane * width,
            ),
            Vec3::new(length * 0.46 * broken_scale, height * 0.16, width * 0.10),
            (random_unit(seed, plank, 2_719) - 0.5) * 0.18,
            seed.wrapping_add(plank * 73),
            0.050,
        );
    }

    for support in 0..3 {
        let x = (support as f32 - 1.0) * length * 0.30;
        append_weathered_block(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(x, height * 0.18, 0.0),
            Vec3::new(width * 0.14, height * 0.18, width * 0.66),
            0.05 * support as f32,
            seed.wrapping_add(support * 89 + 271),
            0.055,
        );
    }

    for post in 0..4 {
        let x = if post < 2 {
            -length * 0.42
        } else {
            length * 0.38
        };
        let z = if post % 2 == 0 {
            -width * 0.62
        } else {
            width * 0.58
        };
        let post_height = height * (1.00 + random_unit(seed, post, 2_801) * 0.58);
        append_weathered_block(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(x, post_height * 0.50, z),
            Vec3::new(width * 0.10, post_height * 0.50, width * 0.10),
            (random_unit(seed, post, 2_809) - 0.5) * 0.24,
            seed.wrapping_add(post * 97 + 503),
            0.050,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

fn wind_vane_mesh(height: f32, span: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    let height = height.max(2.0);
    let span = span.max(1.0);
    let post_width = span * 0.055;

    append_weathered_block(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, height * 0.48, 0.0),
        Vec3::new(post_width, height * 0.48, post_width),
        0.05,
        seed,
        0.040,
    );
    append_weathered_block(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, height * 0.72, 0.0),
        Vec3::new(span * 0.50, post_width * 0.85, post_width * 0.85),
        0.0,
        seed.wrapping_add(97),
        0.040,
    );

    for vane in 0..4 {
        let angle = vane as f32 / 4.0 * std::f32::consts::TAU;
        let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
        let tangent = Vec3::new(-angle.sin(), 0.0, angle.cos());
        let center = radial * span * 0.38 + Vec3::Y * height * (0.72 + vane as f32 * 0.012);
        append_double_sided_detail_card(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            tangent,
            Vec3::Y,
            span * (0.15 + random_unit(seed, vane, 2_901) * 0.035),
            height * 0.115,
        );
    }

    append_crystal_shard(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, height * 0.90, 0.0),
        Vec3::Y,
        span * 0.045,
        height * 0.16,
    );

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

#[allow(clippy::too_many_arguments)]
fn append_weathered_block(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    half_extents: Vec3,
    yaw: f32,
    seed: u32,
    corner_jitter: f32,
) {
    let x_axis = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let y_axis = Vec3::Y;
    let z_axis = Vec3::new(yaw.sin(), 0.0, yaw.cos());
    let jitter_scale = half_extents.min_element().max(0.01) * corner_jitter;
    let corner = |index: u32, sx: f32, sy: f32, sz: f32| {
        let jitter = Vec3::new(
            random_unit(seed, index, 3_101) - 0.5,
            random_unit(seed, index, 3_109) - 0.5,
            random_unit(seed, index, 3_121) - 0.5,
        ) * jitter_scale;
        center
            + x_axis * (sx * half_extents.x)
            + y_axis * (sy * half_extents.y)
            + z_axis * (sz * half_extents.z)
            + jitter
    };
    let corners = [
        corner(0, -1.0, -1.0, -1.0),
        corner(1, 1.0, -1.0, -1.0),
        corner(2, 1.0, 1.0, -1.0),
        corner(3, -1.0, 1.0, -1.0),
        corner(4, -1.0, -1.0, 1.0),
        corner(5, 1.0, -1.0, 1.0),
        corner(6, 1.0, 1.0, 1.0),
        corner(7, -1.0, 1.0, 1.0),
    ];

    for (face_indices, normal) in [
        ([0_usize, 1, 2, 3], -z_axis),
        ([1, 5, 6, 2], x_axis),
        ([5, 4, 7, 6], z_axis),
        ([4, 0, 3, 7], -x_axis),
        ([3, 2, 6, 7], y_axis),
        ([4, 5, 1, 0], -y_axis),
    ] {
        let start = positions.len() as u32;
        for (uv, corner_index) in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            .into_iter()
            .zip(face_indices)
        {
            positions.push(corners[corner_index].to_array());
            normals.push(normal.to_array());
            uvs.push(uv);
        }
        indices.extend([
            start,
            start + 1,
            start + 2,
            start,
            start + 2,
            start + 3,
            start + 2,
            start + 1,
            start,
            start + 3,
            start + 2,
            start,
        ]);
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
