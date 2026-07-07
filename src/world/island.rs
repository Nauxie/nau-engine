use bevy::prelude::{Component, Vec2, Vec3};

use super::{
    LOD_MID_DISTANCE_M, LOD_NEAR_DISTANCE_M, LodBand, PLAYER_STANDING_OFFSET, StreamActivation,
    StreamChunkCoord, TERRAIN_MAX_DROP_M, TERRAIN_MAX_RISE_M, TERRAIN_VISUAL_FOOTING_OFFSET_M,
};

pub const ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandTerrainArchetype {
    LaunchMesa,
    Shelf,
    GardenBasin,
    CrownRidge,
    WindOverlook,
    TerracedSpur,
    RefugeTableland,
    StormRavine,
    OrchardBasin,
    Needle,
    SapphireBasin,
    MistArch,
    BrokenStair,
    CloudGate,
    LaunchSpur,
    GardenApron,
    StormShard,
    OrchardSpur,
    MistStep,
    SkyPlateau,
}

impl IslandTerrainArchetype {
    pub const COUNT: usize = 20;

    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "launch mesa" => Some(Self::LaunchMesa),
            "midpoint shelf" | "stratos shelf" | "upper sky shelf" => Some(Self::Shelf),
            "landing garden" | "quiet lower garden" | "bluevault basin" | "sunspire garden" => {
                Some(Self::GardenBasin)
            }
            "distant crown" | "summit anvil" | "upper crown" | "crown gate" => {
                Some(Self::CrownRidge)
            }
            "great sky plateau" => Some(Self::SkyPlateau),
            "wind overlook" | "lowwind shelf" | "far horizon perch" | "return overlook" => {
                Some(Self::WindOverlook)
            }
            "copper stair" | "sunlit terrace" | "highgate stair" | "east windchain" => {
                Some(Self::TerracedSpur)
            }
            "western refuge" | "underbridge cay" => Some(Self::RefugeTableland),
            "storm porch" => Some(Self::StormRavine),
            "high orchard" | "cloudfall meadow" => Some(Self::OrchardBasin),
            "far needle" | "needle crownlet" | "thin air roost" => Some(Self::Needle),
            "sapphire basin" | "skyhook basin" => Some(Self::SapphireBasin),
            "mist arch" | "upper thermal ring" => Some(Self::MistArch),
            "broken stair" | "outer switchback" | "cloudbreak stair" | "descent stair" => {
                Some(Self::BrokenStair)
            }
            "cloud gate" => Some(Self::CloudGate),
            "launch spur" | "low reef" => Some(Self::LaunchSpur),
            "garden apron" => Some(Self::GardenApron),
            "storm shard" => Some(Self::StormShard),
            "orchard spur" => Some(Self::OrchardSpur),
            "mist stepping stone" => Some(Self::MistStep),
            _ => None,
        }
    }

    fn for_name_or_default(name: &str) -> Self {
        Self::for_name(name).unwrap_or(Self::Shelf)
    }

    pub fn index(self) -> usize {
        match self {
            Self::LaunchMesa => 0,
            Self::Shelf => 1,
            Self::GardenBasin => 2,
            Self::CrownRidge => 3,
            Self::WindOverlook => 4,
            Self::TerracedSpur => 5,
            Self::RefugeTableland => 6,
            Self::StormRavine => 7,
            Self::OrchardBasin => 8,
            Self::Needle => 9,
            Self::SapphireBasin => 10,
            Self::MistArch => 11,
            Self::BrokenStair => 12,
            Self::CloudGate => 13,
            Self::LaunchSpur => 14,
            Self::GardenApron => 15,
            Self::StormShard => 16,
            Self::OrchardSpur => 17,
            Self::MistStep => 18,
            Self::SkyPlateau => 19,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::LaunchMesa => "launch_mesa",
            Self::Shelf => "shelf",
            Self::GardenBasin => "garden_basin",
            Self::CrownRidge => "crown_ridge",
            Self::WindOverlook => "wind_overlook",
            Self::TerracedSpur => "terraced_spur",
            Self::RefugeTableland => "refuge_tableland",
            Self::StormRavine => "storm_ravine",
            Self::OrchardBasin => "orchard_basin",
            Self::Needle => "needle",
            Self::SapphireBasin => "sapphire_basin",
            Self::MistArch => "mist_arch",
            Self::BrokenStair => "broken_stair",
            Self::CloudGate => "cloud_gate",
            Self::LaunchSpur => "launch_spur",
            Self::GardenApron => "garden_apron",
            Self::StormShard => "storm_shard",
            Self::OrchardSpur => "orchard_spur",
            Self::MistStep => "mist_step",
            Self::SkyPlateau => "sky_plateau",
        }
    }

    fn silhouette_bias(self, angle: f32, phase: f32) -> f32 {
        match self {
            Self::LaunchMesa => -0.035 * (angle * 2.0 + phase).cos(),
            Self::Shelf => -0.11 * (angle * 1.0 + phase * 0.4).sin().max(0.0),
            Self::GardenBasin => 0.045 * (angle * 4.0 - phase).cos(),
            Self::CrownRidge => 0.095 * (angle * 6.0 + phase).sin(),
            Self::WindOverlook => 0.13 * (angle - phase * 0.12).cos().max(0.0) - 0.05,
            Self::TerracedSpur => 0.16 * (angle * 1.5 + phase).cos().max(0.0) - 0.06,
            Self::RefugeTableland => -0.065 * (angle * 3.0 + phase).sin().abs(),
            Self::StormRavine => -0.12 * (angle * 5.0 - phase).cos().abs(),
            Self::OrchardBasin => 0.055 * (angle * 5.0 + phase).sin(),
            Self::Needle => -0.14 + 0.06 * (angle * 7.0 + phase).sin(),
            Self::SapphireBasin => 0.075 * (angle * 3.0 - phase).cos(),
            Self::MistArch => {
                let open_bite = (angle - phase * 0.15).cos().max(0.0);
                let rim_lobes = (angle * 2.0 + phase).cos().abs();
                0.11 * rim_lobes - 0.14 * open_bite
            }
            Self::BrokenStair => {
                let stair_notch = (angle * 4.0 - phase).sin().max(0.0);
                let long_run = (angle * 1.35 + phase * 0.4).cos().max(0.0);
                0.13 * long_run - 0.10 * stair_notch
            }
            Self::CloudGate => {
                let gate_shoulders = (angle * 2.0 - phase * 0.5).cos().abs();
                let cleft = (angle * 5.0 + phase).sin().max(0.0);
                0.12 * gate_shoulders - 0.07 * cleft
            }
            Self::LaunchSpur => {
                let forward_lip = (angle - phase * 0.18).cos().max(0.0);
                let rear_cut = (angle * 3.0 + phase).sin().abs();
                0.14 * forward_lip - 0.08 * rear_cut
            }
            Self::GardenApron => {
                let scallop = (angle * 6.0 - phase).cos().max(0.0);
                -0.04 + 0.08 * scallop
            }
            Self::StormShard => {
                let shard = (angle * 2.5 + phase * 0.5).sin().abs();
                0.16 * shard - 0.12 * (angle * 5.0 - phase).cos().abs()
            }
            Self::OrchardSpur => {
                let branch = (angle * 2.0 + phase * 0.35).cos().max(0.0);
                0.12 * branch + 0.04 * (angle * 7.0 - phase).sin()
            }
            Self::MistStep => {
                let stepping_edge = (angle * 4.0 - phase * 0.25).sin().max(0.0);
                let airy_cut = (angle - phase * 0.2).cos().max(0.0);
                0.10 * stepping_edge - 0.13 * airy_cut
            }
            Self::SkyPlateau => {
                let high_shelf = angular_lobe(angle, 2.70, 0.78);
                let broken_edge = angular_lobe(angle, 0.35, 0.50);
                let underhang_bite = angular_lobe(angle, -2.36, 0.46);
                let broad_lobes = (angle * 3.0 + phase * 0.25).cos() * 0.045;
                broad_lobes + high_shelf * 0.14 - broken_edge * 0.10 - underhang_bite * 0.16
            }
        }
    }

    fn relief_bias_m(self, radius: f32, angle: f32, phase: f32) -> f32 {
        match self {
            Self::LaunchMesa => terrain_step(radius, 0.52, 0.75, 0.12),
            Self::Shelf => {
                terrain_step(radius, 0.32, 0.58, 0.16) - smoothstep(0.68, 1.0, radius) * 0.10
            }
            Self::GardenBasin => -basin(radius, 0.42, 0.18) + smoothstep(0.62, 0.94, radius) * 0.10,
            Self::CrownRidge => {
                let crown = (angle * 5.0 + phase).cos().abs();
                smoothstep(0.18, 0.62, radius) * crown * 0.18
            }
            Self::WindOverlook => {
                let windward = (angle - phase * 0.12).cos().max(0.0);
                windward * smoothstep(0.22, 0.74, radius) * 0.20
                    - smoothstep(0.74, 1.0, radius) * 0.08
            }
            Self::TerracedSpur => {
                let spur = (angle * 1.5 + phase).cos().max(0.0);
                terrain_step(radius, 0.26, 0.72, 0.12) + spur * radius * 0.13
            }
            Self::RefugeTableland => terrain_step(radius, 0.38, 0.82, 0.10),
            Self::StormRavine => {
                let cuts = (angle * 4.0 + phase * 0.7).sin().abs();
                -smoothstep(0.22, 0.88, radius) * cuts * 0.20
            }
            Self::OrchardBasin => {
                -basin(radius, 0.50, 0.10)
                    + (angle * 6.0 + phase).sin() * smoothstep(0.18, 0.72, radius) * 0.05
            }
            Self::Needle => (1.0 - radius).powf(1.8) * 0.23 - smoothstep(0.56, 1.0, radius) * 0.18,
            Self::SapphireBasin => {
                -basin(radius, 0.46, 0.22)
                    + smoothstep(0.70, 0.96, radius) * 0.14
                    + (angle * 3.0 - phase).cos() * radius * 0.06
            }
            Self::MistArch => {
                let rim = (angle * 2.0 + phase).cos().abs();
                let opening = (angle - phase * 0.15).cos().max(0.0);
                smoothstep(0.36, 0.86, radius) * rim * 0.20
                    - smoothstep(0.18, 0.62, radius) * opening * 0.18
                    - basin(radius, 0.32, 0.10)
            }
            Self::BrokenStair => {
                let run = (angle * 1.35 + phase * 0.4).cos().max(0.0);
                let step_bands = (radius * std::f32::consts::TAU * 5.0 + phase)
                    .sin()
                    .max(0.0);
                let quiet_shelf = (1.0 - run)
                    * smoothstep(0.26, 0.58, radius)
                    * (1.0 - smoothstep(0.72, 0.92, radius));
                terrain_step(radius, 0.20, 0.78, 0.11)
                    + run * smoothstep(0.22, 0.92, radius) * 0.09
                    + step_bands * smoothstep(0.28, 0.82, radius) * 0.05
                    - quiet_shelf * 0.14
            }
            Self::CloudGate => {
                let shoulders = (angle * 2.0 - phase * 0.5).cos().abs();
                let gate_cleft = (angle * 5.0 + phase).sin().max(0.0);
                smoothstep(0.26, 0.74, radius) * shoulders * 0.18
                    - smoothstep(0.38, 0.92, radius) * gate_cleft * 0.16
                    + (1.0 - radius).powf(2.1) * 0.12
            }
            Self::LaunchSpur => {
                let forward_lip = (angle - phase * 0.18).cos().max(0.0);
                let rear_cut = (angle * 3.0 + phase).sin().abs();
                terrain_step(radius, 0.24, 0.66, 0.10)
                    + forward_lip * smoothstep(0.32, 0.92, radius) * 0.14
                    - rear_cut * smoothstep(0.56, 1.0, radius) * 0.10
            }
            Self::GardenApron => {
                let scallop = (angle * 6.0 - phase).cos().max(0.0);
                -basin(radius, 0.36, 0.12)
                    + scallop * smoothstep(0.54, 0.96, radius) * 0.13
                    + terrain_step(radius, 0.62, 0.86, 0.06)
            }
            Self::StormShard => {
                let shard = (angle * 2.5 + phase * 0.5).sin().abs();
                let crack = (angle * 5.0 - phase).cos().abs();
                shard * smoothstep(0.18, 0.80, radius) * 0.22
                    - crack * smoothstep(0.36, 0.98, radius) * 0.18
                    - smoothstep(0.78, 1.0, radius) * 0.08
            }
            Self::OrchardSpur => {
                let branch = (angle * 2.0 + phase * 0.35).cos().max(0.0);
                -basin(radius, 0.48, 0.08)
                    + branch * smoothstep(0.24, 0.88, radius) * 0.16
                    + (angle * 7.0 - phase).sin() * smoothstep(0.30, 0.72, radius) * 0.04
            }
            Self::MistStep => {
                let step_bands = (radius * std::f32::consts::TAU * 4.0 + phase)
                    .sin()
                    .max(0.0);
                let airy_cut = (angle - phase * 0.2).cos().max(0.0);
                terrain_step(radius, 0.20, 0.72, 0.09)
                    + step_bands * smoothstep(0.22, 0.86, radius) * 0.07
                    - airy_cut * smoothstep(0.42, 0.92, radius) * 0.18
            }
            Self::SkyPlateau => {
                let high_shelf = angular_lobe(angle, 2.70, 0.78)
                    * smoothstep(0.24, 0.76, radius)
                    * (1.0 - smoothstep(0.90, 1.0, radius));
                let low_basin = angular_lobe(angle, -0.92, 0.74)
                    * smoothstep(0.18, 0.70, radius)
                    * (1.0 - smoothstep(0.86, 1.0, radius));
                let broken_edge = angular_lobe(angle, 0.35, 0.52) * smoothstep(0.58, 1.0, radius);
                let underhang_lip =
                    angular_lobe(angle, -2.36, 0.48) * smoothstep(0.44, 0.92, radius);
                let rim =
                    smoothstep(0.66, 0.86, radius) * 0.15 - smoothstep(0.88, 1.0, radius) * 0.26;
                let terraces =
                    (radius * std::f32::consts::TAU * 3.4 + phase * 0.18 + angle.sin() * 0.35)
                        .sin()
                        .max(0.0)
                        * smoothstep(0.20, 0.82, radius)
                        * 0.075;
                let terrace_edges =
                    (1.0 - smoothstep(
                        0.015,
                        0.115,
                        (radius * std::f32::consts::TAU * 7.0 + phase * 0.23)
                            .sin()
                            .abs(),
                    )) * smoothstep(0.24, 0.86, radius)
                        * 0.15;
                let radial_cracks =
                    (1.0 - smoothstep(
                        0.020,
                        0.120,
                        (angle * 9.0 + radius * 3.8 - phase * 0.15).sin().abs(),
                    )) * smoothstep(0.34, 0.94, radius)
                        * 0.20;
                let cliff_teeth = (angle * 15.0 + radius * 5.2 + phase * 0.33)
                    .sin()
                    .max(0.0)
                    .powf(2.6)
                    * smoothstep(0.56, 0.94, radius)
                    * 0.18;
                let mesh_scale_terraces = (radius * std::f32::consts::TAU * 12.0
                    + (angle * 4.0 + phase).sin() * 0.45)
                    .sin()
                    * smoothstep(0.30, 0.88, radius)
                    * 0.22;
                let angular_scarps = (angle * 22.0 + phase * 0.4 + radius * 2.0).sin()
                    * smoothstep(0.48, 0.96, radius)
                    * 0.16;

                high_shelf * 0.24 - low_basin * 0.30 - broken_edge * 0.28 - underhang_lip * 0.18
                    + rim
                    + terraces
                    + terrace_edges
                    + cliff_teeth
                    + mesh_scale_terraces
                    + angular_scarps
                    - radial_cracks
            }
        }
    }

    fn footprint_profile(self) -> IslandFootprintProfile {
        match self {
            Self::LaunchMesa => IslandFootprintProfile::new(
                3.0,
                0.055,
                4.0,
                0.040,
                FootprintShelf::new(-0.35, 0.72, 0.040),
                1.06,
            ),
            Self::Shelf => IslandFootprintProfile::new(
                2.0,
                0.070,
                3.0,
                0.055,
                FootprintShelf::new(-1.45, 0.78, 0.060),
                1.10,
            ),
            Self::GardenBasin => IslandFootprintProfile::new(
                5.0,
                0.055,
                4.0,
                0.045,
                FootprintShelf::new(-1.20, 0.92, 0.045),
                1.08,
            ),
            Self::CrownRidge => IslandFootprintProfile::new(
                6.0,
                0.075,
                5.0,
                0.030,
                FootprintShelf::new(-0.80, 0.70, 0.040),
                1.12,
            ),
            Self::WindOverlook => IslandFootprintProfile::new(
                2.0,
                0.095,
                3.0,
                0.050,
                FootprintShelf::new(-0.15, 0.68, 0.095),
                1.14,
            ),
            Self::TerracedSpur => IslandFootprintProfile::new(
                2.0,
                0.085,
                4.0,
                0.040,
                FootprintShelf::new(-0.55, 0.74, 0.105),
                1.16,
            ),
            Self::RefugeTableland => IslandFootprintProfile::new(
                3.0,
                0.045,
                6.0,
                0.060,
                FootprintShelf::new(-2.20, 0.84, 0.035),
                1.05,
            ),
            Self::StormRavine => IslandFootprintProfile::new(
                5.0,
                0.060,
                7.0,
                0.090,
                FootprintShelf::new(-1.05, 0.58, 0.035),
                1.02,
            ),
            Self::OrchardBasin => IslandFootprintProfile::new(
                5.0,
                0.070,
                4.0,
                0.045,
                FootprintShelf::new(-0.70, 0.94, 0.050),
                1.10,
            ),
            Self::Needle => IslandFootprintProfile::new(
                7.0,
                0.040,
                5.0,
                0.075,
                FootprintShelf::new(-1.35, 0.54, 0.020),
                0.92,
            ),
            Self::SapphireBasin => IslandFootprintProfile::new(
                3.0,
                0.060,
                4.0,
                0.055,
                FootprintShelf::new(-1.70, 0.86, 0.040),
                1.08,
            ),
            Self::MistArch => IslandFootprintProfile::new(
                2.0,
                0.105,
                3.0,
                0.100,
                FootprintShelf::new(-0.35, 0.60, 0.080),
                1.12,
            ),
            Self::BrokenStair => IslandFootprintProfile::new(
                4.0,
                0.080,
                5.0,
                0.070,
                FootprintShelf::new(-0.95, 0.66, 0.075),
                1.13,
            ),
            Self::CloudGate => IslandFootprintProfile::new(
                2.0,
                0.090,
                5.0,
                0.060,
                FootprintShelf::new(-0.20, 0.74, 0.080),
                1.12,
            ),
            Self::LaunchSpur => IslandFootprintProfile::new(
                2.0,
                0.090,
                4.0,
                0.055,
                FootprintShelf::new(-0.65, 0.68, 0.115),
                1.15,
            ),
            Self::GardenApron => IslandFootprintProfile::new(
                6.0,
                0.065,
                5.0,
                0.040,
                FootprintShelf::new(-1.10, 1.00, 0.055),
                1.10,
            ),
            Self::StormShard => IslandFootprintProfile::new(
                3.0,
                0.115,
                6.0,
                0.110,
                FootprintShelf::new(-0.85, 0.50, 0.040),
                1.06,
            ),
            Self::OrchardSpur => IslandFootprintProfile::new(
                2.0,
                0.095,
                5.0,
                0.050,
                FootprintShelf::new(-0.55, 0.72, 0.105),
                1.15,
            ),
            Self::MistStep => IslandFootprintProfile::new(
                4.0,
                0.070,
                4.0,
                0.085,
                FootprintShelf::new(-0.90, 0.58, 0.070),
                1.07,
            ),
            Self::SkyPlateau => IslandFootprintProfile::new(
                4.0,
                0.090,
                6.0,
                0.070,
                FootprintShelf::new(2.70, 0.86, 0.130),
                1.18,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandShapeLanguage {
    TerraceMesa,
    BrokenCrescent,
    RingGarden,
    CliffSlab,
    SpireCluster,
    LakeBasin,
    WaterfallShelf,
    UndercutCaveIsland,
    RuinFoundation,
    SteppedStairIsland,
    PlateauFragment,
    NeedlePerch,
    MeadowShelf,
    BridgeRemnant,
}

impl IslandShapeLanguage {
    pub const COUNT: usize = 14;

    pub fn index(self) -> usize {
        match self {
            Self::TerraceMesa => 0,
            Self::BrokenCrescent => 1,
            Self::RingGarden => 2,
            Self::CliffSlab => 3,
            Self::SpireCluster => 4,
            Self::LakeBasin => 5,
            Self::WaterfallShelf => 6,
            Self::UndercutCaveIsland => 7,
            Self::RuinFoundation => 8,
            Self::SteppedStairIsland => 9,
            Self::PlateauFragment => 10,
            Self::NeedlePerch => 11,
            Self::MeadowShelf => 12,
            Self::BridgeRemnant => 13,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TerraceMesa => "terrace_mesa",
            Self::BrokenCrescent => "broken_crescent",
            Self::RingGarden => "ring_garden",
            Self::CliffSlab => "cliff_slab",
            Self::SpireCluster => "spire_cluster",
            Self::LakeBasin => "lake_basin",
            Self::WaterfallShelf => "waterfall_shelf",
            Self::UndercutCaveIsland => "undercut_cave_island",
            Self::RuinFoundation => "ruin_foundation",
            Self::SteppedStairIsland => "stepped_stair_island",
            Self::PlateauFragment => "plateau_fragment",
            Self::NeedlePerch => "needle_perch",
            Self::MeadowShelf => "meadow_shelf",
            Self::BridgeRemnant => "bridge_remnant",
        }
    }

    fn silhouette_bias(self, angle: f32, phase: f32) -> f32 {
        match self {
            Self::TerraceMesa => {
                let broad_table = angular_lobe(angle, -0.20 + phase * 0.05, 0.98);
                let worn_step = angular_lobe(angle, 1.08 + phase * 0.03, 0.38);
                let rear_cut = angular_lobe(angle, 2.75 + phase * 0.04, 0.56);
                broad_table * 0.105 + worn_step * 0.035 - rear_cut * 0.080
            }
            Self::BrokenCrescent => {
                let outer_horn = angular_lobe(angle, 2.25 + phase * 0.06, 0.82);
                let inner_bite = angular_lobe(angle, -0.15 + phase * 0.04, 0.58);
                let broken_tip = angular_lobe(angle, -2.85 + phase * 0.02, 0.32);
                outer_horn * 0.140 + broken_tip * 0.055 - inner_bite * 0.190
            }
            Self::RingGarden => {
                let scallops = (angle * 8.0 + phase * 0.30).cos() * 0.052;
                let entry_gap = angular_lobe(angle, -1.65 + phase * 0.04, 0.34);
                scallops - entry_gap * 0.065
            }
            Self::CliffSlab => {
                let slab_face = angular_lobe(angle, -0.45 + phase * 0.04, 0.80);
                let sheared_back = angular_lobe(angle, 2.55 + phase * 0.03, 0.66);
                let chipped_corner = angular_lobe(angle, 1.25 + phase * 0.02, 0.34);
                slab_face * 0.165 + chipped_corner * 0.045 - sheared_back * 0.105
            }
            Self::SpireCluster => {
                let teeth = (angle * 9.0 + phase * 0.70).sin().max(0.0).powf(1.5);
                let secondary_teeth = (angle * 13.0 - phase * 0.40).cos().max(0.0).powf(2.0);
                -0.090 + teeth * 0.145 + secondary_teeth * 0.040
            }
            Self::LakeBasin => {
                let shore_bulge = angular_lobe(angle, -1.10 + phase * 0.03, 0.96);
                let spillway = angular_lobe(angle, 1.95 + phase * 0.02, 0.54);
                let reed_cove = angular_lobe(angle, 0.34 + phase * 0.03, 0.42);
                shore_bulge * 0.085 + reed_cove * 0.040 - spillway * 0.095
            }
            Self::WaterfallShelf => {
                let lip = angular_lobe(angle, 0.42 + phase * 0.02, 0.56);
                let fall_cut = angular_lobe(angle, -2.65 + phase * 0.03, 0.44);
                let side_shelf = angular_lobe(angle, -0.42 + phase * 0.02, 0.62);
                lip * 0.185 + side_shelf * 0.055 - fall_cut * 0.205
            }
            Self::UndercutCaveIsland => {
                let overhang_lip = angular_lobe(angle, 0.35 + phase * 0.04, 0.82);
                let cave_mouth = angular_lobe(angle, -2.35 + phase * 0.03, 0.48);
                let side_buttress = angular_lobe(angle, 2.20 + phase * 0.02, 0.36);
                overhang_lip * 0.105 + side_buttress * 0.045 - cave_mouth * 0.230
            }
            Self::RuinFoundation => {
                let foundation_sides = (angle * 4.0 + phase * 0.35).cos().abs();
                let collapsed_corner = angular_lobe(angle, 1.45 + phase * 0.02, 0.40);
                let entry_plaza = angular_lobe(angle, -0.35 + phase * 0.02, 0.46);
                foundation_sides * 0.095 + entry_plaza * 0.035 - collapsed_corner * 0.115
            }
            Self::SteppedStairIsland => {
                let stair_run = angular_lobe(angle, -0.75 + phase * 0.04, 0.78);
                let saw_cuts = (angle * 5.0 - phase * 0.40).sin().max(0.0);
                let landing = angular_lobe(angle, 1.82 + phase * 0.02, 0.36);
                stair_run * 0.150 + landing * 0.045 - saw_cuts * 0.090
            }
            Self::PlateauFragment => {
                let high_shelf = angular_lobe(angle, 2.55 + phase * 0.02, 1.02);
                let broken_face = angular_lobe(angle, -2.45 + phase * 0.02, 0.50);
                let mesa_lobes = (angle * 3.0 + phase * 0.18).cos() * 0.050;
                let cracked_corner = angular_lobe(angle, 0.62 + phase * 0.02, 0.42);
                high_shelf * 0.135 + mesa_lobes + cracked_corner * 0.040 - broken_face * 0.155
            }
            Self::NeedlePerch => {
                let needle_points = (angle * 7.0 + phase * 0.80).sin().max(0.0).powf(1.8);
                let secondary_points = (angle * 11.0 - phase * 0.20).cos().max(0.0).powf(2.4);
                -0.130 + needle_points * 0.130 + secondary_points * 0.035
            }
            Self::MeadowShelf => {
                let meadow_apron = angular_lobe(angle, -1.20 + phase * 0.04, 1.04);
                let soft_coves = (angle * 3.0 - phase * 0.25).sin().max(0.0);
                let path_opening = angular_lobe(angle, 0.95 + phase * 0.02, 0.40);
                meadow_apron * 0.105 + path_opening * 0.025 - soft_coves * 0.065
            }
            Self::BridgeRemnant => {
                let shoulder_a = angular_lobe(angle, 0.40 + phase * 0.02, 0.42);
                let shoulder_b = angular_lobe(angle, 2.70 + phase * 0.02, 0.42);
                let gap = angular_lobe(angle, -1.40 + phase * 0.02, 0.52);
                let fallen_span = angular_lobe(angle, 1.55 + phase * 0.02, 0.34);
                shoulder_a * 0.150 + shoulder_b * 0.135 + fallen_span * 0.050 - gap * 0.185
            }
        }
    }

    fn relief_bias_m(self, radius: f32, angle: f32, phase: f32) -> f32 {
        match self {
            Self::TerraceMesa => {
                let table = terrain_step(radius, 0.28, 0.74, 0.090);
                let outer_step =
                    (1.0 - smoothstep(
                        0.020,
                        0.130,
                        (radius * std::f32::consts::TAU * 5.0 + phase * 0.20)
                            .sin()
                            .abs(),
                    )) * smoothstep(0.30, 0.88, radius)
                        * 0.070;
                let eroded_notch = angular_lobe(angle, 2.75 + phase * 0.04, 0.56)
                    * smoothstep(0.46, 0.98, radius)
                    * 0.095;
                table + outer_step - eroded_notch
            }
            Self::BrokenCrescent => {
                let bite = angular_lobe(angle, -0.15 + phase * 0.04, 0.56);
                let horn = angular_lobe(angle, 2.25 + phase * 0.06, 0.82);
                let fracture =
                    (1.0 - smoothstep(
                        0.018,
                        0.105,
                        (angle * 6.0 + radius * 2.4 - phase * 0.30).sin().abs(),
                    )) * smoothstep(0.40, 0.98, radius)
                        * 0.075;
                horn * smoothstep(0.30, 0.88, radius) * 0.105
                    - bite * smoothstep(0.35, 0.98, radius) * 0.205
                    - fracture
                    + terrain_step(radius, 0.28, 0.70, 0.050)
            }
            Self::RingGarden => {
                let ring = smoothstep(0.42, 0.64, radius) * (1.0 - smoothstep(0.72, 0.90, radius));
                let gate = angular_lobe(angle, -1.65 + phase * 0.04, 0.34);
                let scallop = (angle * 8.0 + phase * 0.30).cos().max(0.0);
                ring * (0.125 + scallop * 0.035) - basin(radius, 0.34, 0.085) - gate * ring * 0.080
            }
            Self::CliffSlab => {
                let slab = angular_lobe(angle, -0.45 + phase * 0.04, 0.72);
                let back_cut = angular_lobe(angle, 2.55 + phase * 0.03, 0.66);
                slab * smoothstep(0.24, 0.86, radius) * 0.175
                    - back_cut * smoothstep(0.50, 1.0, radius) * 0.095
                    - smoothstep(0.82, 1.0, radius) * 0.115
            }
            Self::SpireCluster => {
                let teeth = (angle * 9.0 + phase * 0.70).sin().max(0.0).powf(1.7);
                let secondary = (angle * 13.0 - phase * 0.40).cos().max(0.0).powf(2.0);
                (1.0 - radius).powf(1.65) * 0.075
                    + teeth * smoothstep(0.18, 0.74, radius) * 0.185
                    + secondary * smoothstep(0.30, 0.82, radius) * 0.055
                    - smoothstep(0.70, 1.0, radius) * 0.135
            }
            Self::LakeBasin => {
                let basin_floor = basin(radius, 0.48, 0.155);
                let shore_rim =
                    smoothstep(0.66, 0.80, radius) * (1.0 - smoothstep(0.90, 1.0, radius));
                let spillway = angular_lobe(angle, 1.95 + phase * 0.02, 0.54)
                    * smoothstep(0.58, 1.0, radius)
                    * 0.105;
                shore_rim * 0.150 - basin_floor - spillway
            }
            Self::WaterfallShelf => {
                let lip = angular_lobe(angle, 0.42 + phase * 0.02, 0.52);
                let fall_cut = angular_lobe(angle, -2.65 + phase * 0.03, 0.42);
                let flow_channel =
                    (1.0 - smoothstep(
                        0.018,
                        0.115,
                        (angle + radius * 1.7 - phase * 0.15).sin().abs(),
                    )) * smoothstep(0.28, 0.98, radius)
                        * 0.090;
                lip * smoothstep(0.42, 0.94, radius) * 0.175
                    - fall_cut * smoothstep(0.42, 1.0, radius) * 0.230
                    - flow_channel
            }
            Self::UndercutCaveIsland => {
                let cave_bite = angular_lobe(angle, -2.35 + phase * 0.03, 0.46);
                let lip = angular_lobe(angle, 0.35 + phase * 0.04, 0.82);
                -cave_bite * smoothstep(0.34, 0.98, radius) * 0.235
                    + lip * smoothstep(0.42, 0.86, radius) * 0.100
                    + terrain_step(radius, 0.30, 0.68, 0.055)
            }
            Self::RuinFoundation => {
                let pads = (angle * 4.0 + phase * 0.35).cos().abs();
                let grid_cut =
                    (1.0 - smoothstep(
                        0.020,
                        0.120,
                        (angle * 4.0 + radius * 5.5 + phase * 0.16).sin().abs(),
                    )) * smoothstep(0.28, 0.86, radius)
                        * 0.070;
                pads * smoothstep(0.30, 0.82, radius) * 0.120
                    - grid_cut
                    - basin(radius, 0.28, 0.050)
            }
            Self::SteppedStairIsland => {
                let bands = (radius * std::f32::consts::TAU * 5.0 + phase * 0.32)
                    .sin()
                    .max(0.0);
                let run = angular_lobe(angle, -0.75 + phase * 0.04, 0.78);
                terrain_step(radius, 0.18, 0.84, 0.085)
                    + bands * smoothstep(0.20, 0.90, radius) * 0.090
                    + run * smoothstep(0.28, 0.92, radius) * 0.060
            }
            Self::PlateauFragment => {
                let shelf = angular_lobe(angle, 2.55 + phase * 0.02, 0.95);
                let undercut = angular_lobe(angle, -2.45 + phase * 0.02, 0.48);
                let cracked_rim =
                    (1.0 - smoothstep(
                        0.018,
                        0.125,
                        (angle * 7.0 + radius * 4.2 + phase * 0.25).sin().abs(),
                    )) * smoothstep(0.46, 0.98, radius)
                        * 0.085;
                shelf * smoothstep(0.22, 0.84, radius) * 0.125
                    - undercut * smoothstep(0.44, 1.0, radius) * 0.150
                    - cracked_rim
            }
            Self::NeedlePerch => {
                (1.0 - radius).powf(2.2) * 0.190 - smoothstep(0.58, 1.0, radius) * 0.150
            }
            Self::MeadowShelf => {
                let shelf = angular_lobe(angle, -1.20 + phase * 0.04, 0.98);
                let path =
                    (1.0 - smoothstep(
                        0.030,
                        0.150,
                        (angle - radius * 1.6 + phase * 0.18).sin().abs(),
                    )) * smoothstep(0.18, 0.82, radius)
                        * 0.050;
                shelf * smoothstep(0.30, 0.88, radius) * 0.100 - basin(radius, 0.44, 0.060) - path
            }
            Self::BridgeRemnant => {
                let shoulders = angular_lobe(angle, 0.40 + phase * 0.02, 0.42)
                    + angular_lobe(angle, 2.70 + phase * 0.02, 0.42);
                let gap = angular_lobe(angle, -1.40 + phase * 0.02, 0.52);
                let fallen_span = angular_lobe(angle, 1.55 + phase * 0.02, 0.34);
                shoulders * smoothstep(0.24, 0.84, radius) * 0.155
                    + fallen_span * smoothstep(0.34, 0.78, radius) * 0.060
                    - gap * smoothstep(0.38, 0.96, radius) * 0.195
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FootprintShelf {
    angle: f32,
    width: f32,
    strength: f32,
}

impl FootprintShelf {
    const fn new(angle: f32, width: f32, strength: f32) -> Self {
        Self {
            angle,
            width,
            strength,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IslandFootprintProfile {
    lobes: f32,
    lobe_strength: f32,
    coves: f32,
    cove_depth: f32,
    shelf_angle: f32,
    shelf_width: f32,
    shelf_strength: f32,
    playable_max_scale: f32,
}

impl IslandFootprintProfile {
    const fn new(
        lobes: f32,
        lobe_strength: f32,
        coves: f32,
        cove_depth: f32,
        shelf: FootprintShelf,
        playable_max_scale: f32,
    ) -> Self {
        Self {
            lobes,
            lobe_strength,
            coves,
            cove_depth,
            shelf_angle: shelf.angle,
            shelf_width: shelf.width,
            shelf_strength: shelf.strength,
            playable_max_scale,
        }
    }

    fn bias(self, angle: f32, phase: f32) -> f32 {
        let lobe = (angle * self.lobes + phase * 0.57).cos() * self.lobe_strength;
        let cove = (angle * self.coves - phase * 0.41).sin().max(0.0).powf(1.6) * self.cove_depth;
        let shelf_axis = self.shelf_angle + phase * 0.07;
        let shelf =
            angular_lobe(angle, shelf_axis, self.shelf_width).powf(1.25) * self.shelf_strength;
        let neck = angular_lobe(
            angle,
            shelf_axis + std::f32::consts::PI,
            self.shelf_width * 0.74,
        )
        .powf(1.1)
            * self.cove_depth
            * 0.55;

        lobe + shelf - cove - neck
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandScaleClass {
    Tiny,
    Small,
    Medium,
    Large,
    HugePlateau,
}

impl IslandScaleClass {
    pub const COUNT: usize = 5;

    pub fn label(self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::HugePlateau => "huge_plateau",
        }
    }

    pub fn from_footprint_area(area_m2: f32) -> Self {
        if area_m2 >= 6_000.0 {
            Self::HugePlateau
        } else if area_m2 >= 2_600.0 {
            Self::Large
        } else if area_m2 >= 1_200.0 {
            Self::Medium
        } else if area_m2 >= 600.0 {
            Self::Small
        } else {
            Self::Tiny
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandBiome {
    Meadow,
    Garden,
    Storm,
    Orchard,
    Lake,
    Mist,
    Alpine,
    Ruin,
}

impl IslandBiome {
    pub const COUNT: usize = 8;

    pub fn label(self) -> &'static str {
        match self {
            Self::Meadow => "meadow",
            Self::Garden => "garden",
            Self::Storm => "storm",
            Self::Orchard => "orchard",
            Self::Lake => "lake",
            Self::Mist => "mist",
            Self::Alpine => "alpine",
            Self::Ruin => "ruin",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandWaterFeature {
    Dry,
    Pond,
    LakeBasin,
    WaterfallSource,
}

impl IslandWaterFeature {
    pub const COUNT: usize = 4;

    pub fn label(self) -> &'static str {
        match self {
            Self::Dry => "dry",
            Self::Pond => "pond",
            Self::LakeBasin => "lake_basin",
            Self::WaterfallSource => "waterfall_source",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandVerticalProfile {
    Mesa,
    Shelf,
    Basin,
    Spire,
    Stair,
    Ravine,
    Arch,
    UnderhangCave,
    Plateau,
}

impl IslandVerticalProfile {
    pub const COUNT: usize = 9;

    pub fn label(self) -> &'static str {
        match self {
            Self::Mesa => "mesa",
            Self::Shelf => "shelf",
            Self::Basin => "basin",
            Self::Spire => "spire",
            Self::Stair => "stair",
            Self::Ravine => "ravine",
            Self::Arch => "arch",
            Self::UnderhangCave => "underhang_cave",
            Self::Plateau => "plateau",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandRouteRole {
    Launch,
    MainPath,
    Destination,
    RecoveryBranch,
    SteppingStone,
    UpdraftHub,
    SkyPlateau,
    Satellite,
}

impl IslandRouteRole {
    pub const COUNT: usize = 8;

    pub fn label(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::MainPath => "main_path",
            Self::Destination => "destination",
            Self::RecoveryBranch => "recovery_branch",
            Self::SteppingStone => "stepping_stone",
            Self::UpdraftHub => "updraft_hub",
            Self::SkyPlateau => "sky_plateau",
            Self::Satellite => "satellite",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandLandmarkRole {
    None,
    LaunchMesa,
    LandingGarden,
    WindGate,
    RuinArch,
    CaveMouth,
    LakeBasin,
    HighCrown,
    WaterfallEdge,
}

impl IslandLandmarkRole {
    pub const COUNT: usize = 9;

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::LaunchMesa => "launch_mesa",
            Self::LandingGarden => "landing_garden",
            Self::WindGate => "wind_gate",
            Self::RuinArch => "ruin_arch",
            Self::CaveMouth => "cave_mouth",
            Self::LakeBasin => "lake_basin",
            Self::HighCrown => "high_crown",
            Self::WaterfallEdge => "waterfall_edge",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum IslandPlateauRegion {
    MeadowPlateau,
    CliffRim,
    HighShelf,
    LowBasin,
    BrokenEdge,
    UnderhangEntry,
}

impl IslandPlateauRegion {
    pub const COUNT: usize = 6;

    pub fn label(self) -> &'static str {
        match self {
            Self::MeadowPlateau => "meadow_plateau",
            Self::CliffRim => "cliff_rim",
            Self::HighShelf => "high_shelf",
            Self::LowBasin => "low_basin",
            Self::BrokenEdge => "broken_edge",
            Self::UnderhangEntry => "underhang_entry",
        }
    }

    pub fn sample_offset(self) -> Vec2 {
        match self {
            Self::MeadowPlateau => Vec2::new(0.0, 0.0),
            Self::CliffRim => Vec2::new(0.0, 0.82),
            Self::HighShelf => Vec2::new(-0.58, 0.22),
            Self::LowBasin => Vec2::new(0.28, -0.42),
            Self::BrokenEdge => Vec2::new(0.64, 0.24),
            Self::UnderhangEntry => Vec2::new(-0.52, -0.46),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IslandUnderRouteSegment {
    pub island_name: &'static str,
    pub entry_region: IslandPlateauRegion,
    pub entry: Vec3,
    pub midpoint: Vec3,
    pub exit_region: IslandPlateauRegion,
    pub exit: Vec3,
    pub clearance_radius_m: f32,
    pub recommended_lift_point: Vec3,
}

impl IslandUnderRouteSegment {
    pub fn sample_points(self) -> [Vec3; 3] {
        [self.entry, self.midpoint, self.exit]
    }

    pub fn distance_to(self, point: Vec3) -> f32 {
        let points = self.sample_points();
        let entry_to_midpoint = point_segment_distance(point, points[0], points[1]);
        let midpoint_to_exit = point_segment_distance(point, points[1], points[2]);

        entry_to_midpoint.min(midpoint_to_exit)
    }

    pub fn contains_clearance(self, point: Vec3, padding_m: f32) -> bool {
        self.distance_to(point) <= self.clearance_radius_m + padding_m.max(0.0)
    }

    pub fn horizontal_length_m(self) -> f32 {
        let entry_to_mid = Vec2::new(
            self.midpoint.x - self.entry.x,
            self.midpoint.z - self.entry.z,
        )
        .length();
        let mid_to_exit =
            Vec2::new(self.exit.x - self.midpoint.x, self.exit.z - self.midpoint.z).length();

        entry_to_mid + mid_to_exit
    }
}

fn point_segment_distance(point: Vec3, start: Vec3, end: Vec3) -> f32 {
    let segment = end - start;
    let segment_length_sq = segment.length_squared();
    if segment_length_sq <= f32::EPSILON {
        return point.distance(start);
    }

    let t = ((point - start).dot(segment) / segment_length_sq).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct IslandWorldTags {
    pub scale_class: IslandScaleClass,
    pub biome: IslandBiome,
    pub water_feature: IslandWaterFeature,
    pub vertical_profile: IslandVerticalProfile,
    pub route_role: IslandRouteRole,
    pub landmark_role: IslandLandmarkRole,
}

impl IslandWorldTags {
    fn inferred(
        name: &str,
        half_extents: Vec2,
        thickness: f32,
        is_target: bool,
        terrain_archetype: IslandTerrainArchetype,
    ) -> Self {
        let scale_class = IslandScaleClass::from_footprint_area(half_extents.x * half_extents.y);

        Self {
            scale_class,
            biome: island_biome_for(name, terrain_archetype),
            water_feature: island_water_feature_for(name, terrain_archetype),
            vertical_profile: island_vertical_profile_for(
                name,
                scale_class,
                thickness,
                terrain_archetype,
            ),
            route_role: island_route_role_for(name, scale_class, is_target),
            landmark_role: island_landmark_role_for(name, terrain_archetype),
        }
    }

    pub fn labels(self) -> [&'static str; 6] {
        [
            self.scale_class.label(),
            self.biome.label(),
            self.water_feature.label(),
            self.vertical_profile.label(),
            self.route_role.label(),
            self.landmark_role.label(),
        ]
    }
}

fn island_biome_for(name: &str, terrain_archetype: IslandTerrainArchetype) -> IslandBiome {
    match terrain_archetype {
        IslandTerrainArchetype::GardenBasin | IslandTerrainArchetype::GardenApron => {
            IslandBiome::Garden
        }
        IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard => {
            IslandBiome::Storm
        }
        IslandTerrainArchetype::OrchardBasin | IslandTerrainArchetype::OrchardSpur => {
            IslandBiome::Orchard
        }
        IslandTerrainArchetype::SapphireBasin => IslandBiome::Lake,
        IslandTerrainArchetype::MistArch
        | IslandTerrainArchetype::MistStep
        | IslandTerrainArchetype::CloudGate => IslandBiome::Mist,
        IslandTerrainArchetype::CrownRidge | IslandTerrainArchetype::Needle => IslandBiome::Alpine,
        IslandTerrainArchetype::BrokenStair => IslandBiome::Ruin,
        IslandTerrainArchetype::SkyPlateau => IslandBiome::Meadow,
        _ if name.contains("anvil") || name.contains("crown") => IslandBiome::Alpine,
        _ => IslandBiome::Meadow,
    }
}

fn island_water_feature_for(
    name: &str,
    terrain_archetype: IslandTerrainArchetype,
) -> IslandWaterFeature {
    if name == "cloudfall meadow" {
        IslandWaterFeature::WaterfallSource
    } else if terrain_archetype == IslandTerrainArchetype::SapphireBasin
        || matches!(name, "bluevault basin" | "stratos shelf")
    {
        IslandWaterFeature::LakeBasin
    } else if matches!(
        terrain_archetype,
        IslandTerrainArchetype::GardenBasin
            | IslandTerrainArchetype::GardenApron
            | IslandTerrainArchetype::OrchardBasin
            | IslandTerrainArchetype::SkyPlateau
    ) {
        IslandWaterFeature::Pond
    } else {
        IslandWaterFeature::Dry
    }
}

fn island_vertical_profile_for(
    name: &str,
    scale_class: IslandScaleClass,
    thickness: f32,
    terrain_archetype: IslandTerrainArchetype,
) -> IslandVerticalProfile {
    if name == "underbridge cay" {
        return IslandVerticalProfile::UnderhangCave;
    }
    if scale_class == IslandScaleClass::HugePlateau || name.contains("anvil") {
        return IslandVerticalProfile::Plateau;
    }
    if thickness >= 34.0 && matches!(scale_class, IslandScaleClass::Large) {
        return IslandVerticalProfile::Plateau;
    }

    match terrain_archetype {
        IslandTerrainArchetype::LaunchMesa => IslandVerticalProfile::Mesa,
        IslandTerrainArchetype::Needle | IslandTerrainArchetype::StormShard => {
            IslandVerticalProfile::Spire
        }
        IslandTerrainArchetype::GardenBasin
        | IslandTerrainArchetype::OrchardBasin
        | IslandTerrainArchetype::SapphireBasin
        | IslandTerrainArchetype::GardenApron => IslandVerticalProfile::Basin,
        IslandTerrainArchetype::TerracedSpur
        | IslandTerrainArchetype::BrokenStair
        | IslandTerrainArchetype::MistStep
        | IslandTerrainArchetype::LaunchSpur => IslandVerticalProfile::Stair,
        IslandTerrainArchetype::StormRavine => IslandVerticalProfile::Ravine,
        IslandTerrainArchetype::MistArch | IslandTerrainArchetype::CloudGate => {
            IslandVerticalProfile::Arch
        }
        IslandTerrainArchetype::SkyPlateau => IslandVerticalProfile::Plateau,
        _ => IslandVerticalProfile::Shelf,
    }
}

fn island_route_role_for(
    name: &str,
    scale_class: IslandScaleClass,
    is_target: bool,
) -> IslandRouteRole {
    if is_target {
        return IslandRouteRole::Destination;
    }
    if name == "launch mesa" {
        return IslandRouteRole::Launch;
    }
    if matches!(name, "sunlit terrace" | "western refuge") {
        return IslandRouteRole::RecoveryBranch;
    }
    if matches!(
        name,
        "midpoint shelf"
            | "distant crown"
            | "wind overlook"
            | "copper stair"
            | "storm porch"
            | "high orchard"
            | "far needle"
            | "sapphire basin"
            | "broken stair"
            | "mist arch"
            | "cloud gate"
    ) {
        return IslandRouteRole::MainPath;
    }
    if matches!(
        name,
        "upper thermal ring"
            | "lowwind shelf"
            | "east windchain"
            | "far horizon perch"
            | "return overlook"
    ) {
        return IslandRouteRole::UpdraftHub;
    }
    if scale_class == IslandScaleClass::HugePlateau || name.contains("anvil") {
        return IslandRouteRole::SkyPlateau;
    }
    if matches!(
        scale_class,
        IslandScaleClass::Tiny | IslandScaleClass::Small
    ) {
        return IslandRouteRole::SteppingStone;
    }

    IslandRouteRole::Satellite
}

fn island_landmark_role_for(
    name: &str,
    terrain_archetype: IslandTerrainArchetype,
) -> IslandLandmarkRole {
    if name == "launch mesa" {
        IslandLandmarkRole::LaunchMesa
    } else if name == "landing garden" {
        IslandLandmarkRole::LandingGarden
    } else if name == "underbridge cay" {
        IslandLandmarkRole::CaveMouth
    } else if name == "cloudfall meadow" {
        IslandLandmarkRole::WaterfallEdge
    } else if terrain_archetype == IslandTerrainArchetype::SapphireBasin {
        IslandLandmarkRole::LakeBasin
    } else if matches!(
        terrain_archetype,
        IslandTerrainArchetype::MistArch
            | IslandTerrainArchetype::CloudGate
            | IslandTerrainArchetype::BrokenStair
    ) {
        IslandLandmarkRole::RuinArch
    } else if matches!(
        terrain_archetype,
        IslandTerrainArchetype::WindOverlook | IslandTerrainArchetype::MistStep
    ) {
        IslandLandmarkRole::WindGate
    } else if terrain_archetype == IslandTerrainArchetype::CrownRidge || name.contains("anvil") {
        IslandLandmarkRole::HighCrown
    } else if terrain_archetype == IslandTerrainArchetype::SkyPlateau {
        IslandLandmarkRole::CaveMouth
    } else {
        IslandLandmarkRole::None
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SkyIsland {
    pub name: &'static str,
    pub center: Vec3,
    pub half_extents: Vec2,
    pub thickness: f32,
    pub is_target: bool,
    pub terrain_archetype: IslandTerrainArchetype,
    pub world_tags: IslandWorldTags,
}

impl SkyIsland {
    pub fn new(
        name: &'static str,
        center: Vec3,
        half_extents: Vec2,
        thickness: f32,
        is_target: bool,
    ) -> Self {
        let terrain_archetype = IslandTerrainArchetype::for_name_or_default(name);
        let thickness = thickness.max(1.0);
        Self {
            name,
            center,
            half_extents,
            thickness,
            is_target,
            terrain_archetype,
            world_tags: IslandWorldTags::inferred(
                name,
                half_extents,
                thickness,
                is_target,
                terrain_archetype,
            ),
        }
    }

    pub fn base_area_m2(self) -> f32 {
        self.half_extents.x * self.half_extents.y
    }

    pub fn longest_span_m(self) -> f32 {
        self.half_extents.x.max(self.half_extents.y) * 2.0
    }

    pub fn has_major_water_feature(self) -> bool {
        matches!(
            self.world_tags.water_feature,
            IslandWaterFeature::LakeBasin | IslandWaterFeature::WaterfallSource
        )
    }

    pub fn has_underworld_route_potential(self) -> bool {
        self.world_tags.vertical_profile == IslandVerticalProfile::UnderhangCave
            || self.world_tags.landmark_role == IslandLandmarkRole::CaveMouth
            || self.terrain_archetype == IslandTerrainArchetype::SkyPlateau
    }

    pub fn is_great_plateau_anchor(self) -> bool {
        self.terrain_archetype == IslandTerrainArchetype::SkyPlateau
            && self.world_tags.scale_class == IslandScaleClass::HugePlateau
    }

    pub fn shape_language(self) -> IslandShapeLanguage {
        if self.name == "underbridge cay" {
            return IslandShapeLanguage::UndercutCaveIsland;
        }
        if self.name == "cloudfall meadow" {
            return IslandShapeLanguage::WaterfallShelf;
        }
        if self.name == "bluevault basin" || self.has_major_water_feature() {
            return IslandShapeLanguage::LakeBasin;
        }
        if self.name == "outer switchback" {
            return IslandShapeLanguage::RuinFoundation;
        }

        match self.terrain_archetype {
            IslandTerrainArchetype::LaunchMesa | IslandTerrainArchetype::LaunchSpur => {
                IslandShapeLanguage::TerraceMesa
            }
            IslandTerrainArchetype::Shelf => {
                if self.name == "stratos shelf" || self.name == "upper sky shelf" {
                    IslandShapeLanguage::CliffSlab
                } else {
                    IslandShapeLanguage::MeadowShelf
                }
            }
            IslandTerrainArchetype::GardenBasin | IslandTerrainArchetype::GardenApron => {
                IslandShapeLanguage::RingGarden
            }
            IslandTerrainArchetype::CrownRidge => {
                if self.name == "summit anvil" {
                    IslandShapeLanguage::PlateauFragment
                } else {
                    IslandShapeLanguage::SpireCluster
                }
            }
            IslandTerrainArchetype::WindOverlook => IslandShapeLanguage::CliffSlab,
            IslandTerrainArchetype::TerracedSpur
            | IslandTerrainArchetype::BrokenStair
            | IslandTerrainArchetype::MistStep => IslandShapeLanguage::SteppedStairIsland,
            IslandTerrainArchetype::RefugeTableland => IslandShapeLanguage::MeadowShelf,
            IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard => {
                IslandShapeLanguage::BrokenCrescent
            }
            IslandTerrainArchetype::OrchardBasin | IslandTerrainArchetype::OrchardSpur => {
                IslandShapeLanguage::MeadowShelf
            }
            IslandTerrainArchetype::Needle => IslandShapeLanguage::NeedlePerch,
            IslandTerrainArchetype::SapphireBasin => IslandShapeLanguage::LakeBasin,
            IslandTerrainArchetype::MistArch | IslandTerrainArchetype::CloudGate => {
                IslandShapeLanguage::BridgeRemnant
            }
            IslandTerrainArchetype::SkyPlateau => IslandShapeLanguage::PlateauFragment,
        }
    }

    pub fn plateau_region_at_normalized_offset(
        self,
        normalized_offset: Vec2,
    ) -> Option<IslandPlateauRegion> {
        if self.terrain_archetype != IslandTerrainArchetype::SkyPlateau {
            return None;
        }

        let radius = normalized_offset.length();
        if radius <= f32::EPSILON {
            return Some(IslandPlateauRegion::MeadowPlateau);
        }

        let angle = normalized_offset.y.atan2(normalized_offset.x);
        if radius > self.playable_silhouette_scale(angle) * 0.96 {
            return None;
        }

        if normalized_offset.x < -0.42 && normalized_offset.y < -0.34 {
            return Some(IslandPlateauRegion::UnderhangEntry);
        }
        if normalized_offset.x > 0.56 && normalized_offset.y > 0.12 {
            return Some(IslandPlateauRegion::BrokenEdge);
        }
        if normalized_offset.x < -0.46 && normalized_offset.y > 0.08 {
            return Some(IslandPlateauRegion::HighShelf);
        }
        if normalized_offset.x > 0.16 && normalized_offset.y < -0.24 {
            return Some(IslandPlateauRegion::LowBasin);
        }
        if radius >= 0.72 {
            return Some(IslandPlateauRegion::CliffRim);
        }

        Some(IslandPlateauRegion::MeadowPlateau)
    }

    pub fn plateau_region_position(self, region: IslandPlateauRegion) -> Option<Vec3> {
        let offset = region.sample_offset();
        if self.plateau_region_at_normalized_offset(offset) != Some(region) {
            return None;
        }

        let x = self.center.x + self.half_extents.x * offset.x;
        let z = self.center.z + self.half_extents.y * offset.y;
        let surface_position = Vec3::new(x, self.center.y, z);

        Some(Vec3::new(x, self.terrain_surface_y_at(surface_position), z))
    }

    pub fn under_route_segment(self) -> Option<IslandUnderRouteSegment> {
        if self.is_great_plateau_anchor() {
            return self.great_plateau_under_route_segment();
        }

        if self.name == "underbridge cay" {
            return Some(self.underbridge_cay_under_route_segment());
        }

        None
    }

    fn great_plateau_under_route_segment(self) -> Option<IslandUnderRouteSegment> {
        let entry_region = IslandPlateauRegion::UnderhangEntry;
        let exit_region = IslandPlateauRegion::MeadowPlateau;
        let entry_surface = self.plateau_region_position(entry_region)?;
        let entry = Vec3::new(
            entry_surface.x,
            entry_surface.y - self.thickness * 0.32,
            entry_surface.z,
        );
        let midpoint = Vec3::new(
            self.center.x - self.half_extents.x * 0.08,
            self.mesh_top_y() - self.thickness * 0.58,
            self.center.z - self.half_extents.y * 0.04,
        );
        let exit = Vec3::new(
            self.center.x + self.half_extents.x * 0.02,
            self.mesh_top_y() - self.thickness * 0.34,
            self.center.z + self.half_extents.y * 0.02,
        );

        Some(IslandUnderRouteSegment {
            island_name: self.name,
            entry_region,
            entry,
            midpoint,
            exit_region,
            exit,
            clearance_radius_m: (self.half_extents.min_element() * 0.08).clamp(10.0, 16.0),
            recommended_lift_point: exit,
        })
    }

    fn underbridge_cay_under_route_segment(self) -> IslandUnderRouteSegment {
        let entry = Vec3::new(
            self.center.x - self.half_extents.x * 1.08,
            self.mesh_top_y() - self.thickness * 0.56,
            self.center.z - self.half_extents.y * 0.24,
        );
        let midpoint = Vec3::new(
            self.center.x - self.half_extents.x * 0.08,
            self.mesh_top_y() - self.thickness * 0.78,
            self.center.z + self.half_extents.y * 0.02,
        );
        let exit = Vec3::new(
            self.center.x + self.half_extents.x * 1.12,
            self.mesh_top_y() - self.thickness * 0.52,
            self.center.z + self.half_extents.y * 0.28,
        );

        IslandUnderRouteSegment {
            island_name: self.name,
            entry_region: IslandPlateauRegion::UnderhangEntry,
            entry,
            midpoint,
            exit_region: IslandPlateauRegion::MeadowPlateau,
            exit,
            clearance_radius_m: (self.half_extents.min_element() * 0.36).clamp(4.8, 6.4),
            recommended_lift_point: Vec3::new(-34.0, 34.0, -86.0),
        }
    }

    pub fn floor_y(self) -> f32 {
        self.center.y
    }

    pub fn mesh_top_y(self) -> f32 {
        self.floor_y() - PLAYER_STANDING_OFFSET
    }

    pub fn terrain_surface_y_at(self, position: Vec3) -> f32 {
        let (radius, angle) = self.playable_polar_at(position);

        self.terrain_surface_y_at_polar(radius, angle)
    }

    pub fn terrain_surface_y_at_polar(self, radius: f32, angle: f32) -> f32 {
        self.floor_y() + self.terrain_relief_m(radius, angle)
    }

    pub fn mesh_top_y_at(self, position: Vec3) -> f32 {
        self.terrain_surface_y_at(position) - TERRAIN_VISUAL_FOOTING_OFFSET_M
    }

    pub fn mesh_top_y_at_polar(self, radius: f32, angle: f32) -> f32 {
        self.terrain_surface_y_at_polar(radius, angle) - TERRAIN_VISUAL_FOOTING_OFFSET_M
    }

    pub fn terrain_relief_m(self, radius: f32, angle: f32) -> f32 {
        let radius = radius.clamp(0.0, 1.0);
        if radius <= f32::EPSILON {
            return 0.0;
        }

        let angle = normalized_angle(angle);
        let phase = self.terrain_phase();
        let ridge = radius
            * ((angle * 3.0 + phase).sin() * 0.28 + (angle * 7.0 - phase * 0.5).cos() * 0.14);
        let shoulder = (radius * std::f32::consts::PI).sin() * 0.24;
        let center_falloff = ((1.0 - radius).powi(2) - 1.0) * 0.16;
        let edge_drop = -radius.powf(2.35) * 0.42;
        let ravines = terrain_ravine_relief_m(radius, angle, phase);
        let terrace = terrain_terrace_relief_m(radius, angle, phase);
        let braided_path = terrain_braided_path_relief_m(radius, angle, phase);
        let strata_crag = terrain_strata_crag_relief_m(radius, angle, phase);
        let micro = terrain_micro_relief_m(radius, angle, phase);
        let archetype = self.terrain_archetype.relief_bias_m(radius, angle, phase);
        let shape = self.shape_language().relief_bias_m(radius, angle, phase);

        (ridge
            + shoulder
            + center_falloff
            + edge_drop
            + ravines
            + terrace
            + braided_path
            + strata_crag
            + micro
            + archetype
            + shape)
            .clamp(-TERRAIN_MAX_DROP_M, TERRAIN_MAX_RISE_M)
    }

    pub fn contains_horizontal(self, position: Vec3) -> bool {
        let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
        let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
        let radius = Vec2::new(dx, dz).length();
        let angle = dz.atan2(dx);
        radius <= self.playable_silhouette_scale(angle)
    }

    pub fn horizontal_distance(self, position: Vec3) -> f32 {
        Vec2::new(position.x - self.center.x, position.z - self.center.z).length()
    }

    pub fn visual_edge_distance(self, position: Vec3) -> f32 {
        let horizontal = Vec2::new(position.x - self.center.x, position.z - self.center.z);
        let center_distance = horizontal.length();
        if center_distance <= f32::EPSILON {
            return 0.0;
        }

        let normalized = Vec2::new(
            horizontal.x / self.half_extents.x.max(0.001),
            horizontal.y / self.half_extents.y.max(0.001),
        );
        let angle = normalized.y.atan2(normalized.x);
        let contour = self.footprint_contour_point(angle, true);
        let contour_offset = contour - Vec2::new(self.center.x, self.center.z);

        (center_distance - contour_offset.length()).max(0.0)
    }

    pub fn lod_band(self, position: Vec3) -> LodBand {
        let distance = self.visual_edge_distance(position);
        if distance <= LOD_NEAR_DISTANCE_M {
            LodBand::Near
        } else if distance <= LOD_MID_DISTANCE_M {
            LodBand::Mid
        } else {
            LodBand::Far
        }
    }

    pub fn streaming_chunk(self) -> StreamChunkCoord {
        StreamChunkCoord::from_world(self.center)
    }

    pub fn stream_activation(self, position: Vec3) -> StreamActivation {
        let player_chunk = StreamChunkCoord::from_world(position);
        if self.streaming_chunk().is_inside_active_window(player_chunk) {
            StreamActivation::Active
        } else {
            StreamActivation::Inactive
        }
    }

    fn terrain_phase(self) -> f32 {
        self.center.x * 0.013
            + self.center.z * 0.009
            + self.half_extents.x * 0.021
            + self.half_extents.y * 0.017
    }

    pub fn visual_silhouette_scale(self, angle: f32) -> f32 {
        let angle = normalized_angle(angle);
        let phase = self.terrain_phase();
        (1.0 + 0.09 * (angle * 3.0 + phase).sin()
            + 0.055 * (angle * 7.0 - phase * 0.4).cos()
            + 0.032 * (angle * 11.0 + phase * 1.7).sin()
            + self.terrain_archetype.silhouette_bias(angle, phase)
            + self.shape_language().silhouette_bias(angle, phase)
            + self.footprint_profile().bias(angle, phase))
        .clamp(0.54, 1.34)
    }

    pub fn playable_silhouette_scale(self, angle: f32) -> f32 {
        debug_assert!(self.footprint_profile().playable_max_scale >= 0.54);
        self.visual_silhouette_scale(angle)
    }

    pub fn footprint_profile(self) -> IslandFootprintProfile {
        self.terrain_archetype.footprint_profile()
    }

    pub fn footprint_contour_point(self, angle: f32, visual: bool) -> Vec2 {
        let scale = if visual {
            self.visual_silhouette_scale(angle)
        } else {
            self.playable_silhouette_scale(angle)
        };

        Vec2::new(
            self.center.x + angle.cos() * self.half_extents.x * scale,
            self.center.z + angle.sin() * self.half_extents.y * scale,
        )
    }

    pub fn footprint_contour_samples(
        self,
        visual: bool,
    ) -> [Vec2; ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT] {
        std::array::from_fn(|index| {
            let angle =
                index as f32 / ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT as f32 * std::f32::consts::TAU;
            self.footprint_contour_point(angle, visual)
        })
    }

    fn playable_polar_at(self, position: Vec3) -> (f32, f32) {
        let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
        let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
        let local_radius = Vec2::new(dx, dz).length();
        let angle = dz.atan2(dx);
        let playable_radius = self.playable_silhouette_scale(angle).max(0.001);

        ((local_radius / playable_radius).clamp(0.0, 1.0), angle)
    }
}

fn terrain_ravine_relief_m(radius: f32, angle: f32, phase: f32) -> f32 {
    let radial_mask = smoothstep(0.18, 0.82, radius) * (1.0 - smoothstep(0.88, 1.0, radius));
    let primary_axis = (angle * 2.0 + phase * 1.3).sin().abs();
    let secondary_axis = (angle * 3.0 - phase * 0.7).cos().abs();
    let primary = 1.0 - smoothstep(0.02, 0.18, primary_axis);
    let secondary = 1.0 - smoothstep(0.02, 0.14, secondary_axis);

    -(primary * 0.14 + secondary * 0.08) * radial_mask
}

fn terrain_terrace_relief_m(radius: f32, angle: f32, phase: f32) -> f32 {
    let terrace_mask = smoothstep(0.24, 0.58, radius) * (1.0 - smoothstep(0.76, 0.96, radius));
    let terrace_wave =
        (radius * std::f32::consts::TAU * 4.4 + phase * 0.5 + (angle * 2.0 + phase).sin() * 0.45)
            .sin();
    terrace_wave * terrace_mask * 0.045
}

fn terrain_braided_path_relief_m(radius: f32, angle: f32, phase: f32) -> f32 {
    let path_mask = smoothstep(0.16, 0.30, radius) * (1.0 - smoothstep(0.86, 0.98, radius));
    let braid_a_axis = (angle + radius * 1.85 + phase * 0.37).sin().abs();
    let braid_b_axis = (angle - radius * 2.25 - phase * 0.19).cos().abs();
    let braid_a_cut = 1.0 - smoothstep(0.025, 0.115, braid_a_axis);
    let braid_b_cut = 1.0 - smoothstep(0.020, 0.095, braid_b_axis);
    let braid_a_berm =
        smoothstep(0.095, 0.145, braid_a_axis) * (1.0 - smoothstep(0.145, 0.240, braid_a_axis));
    let braid_b_berm =
        smoothstep(0.080, 0.130, braid_b_axis) * (1.0 - smoothstep(0.130, 0.220, braid_b_axis));

    path_mask
        * (braid_a_berm * 0.032 + braid_b_berm * 0.024 - braid_a_cut * 0.060 - braid_b_cut * 0.040)
}

fn terrain_strata_crag_relief_m(radius: f32, angle: f32, phase: f32) -> f32 {
    let strata_mask = smoothstep(0.22, 0.48, radius) * (1.0 - smoothstep(0.90, 1.0, radius));
    let strata_phase =
        radius * std::f32::consts::TAU * 8.0 + phase * 0.6 + (angle * 3.0 + phase).sin() * 0.7;
    let ledge = 1.0 - smoothstep(0.015, 0.170, strata_phase.sin().abs());
    let crag_high = (angle * 13.0 + radius * 15.0 + phase)
        .sin()
        .max(0.0)
        .powf(2.4);
    let crag_cut = (angle * 11.0 - radius * 10.0 - phase * 0.8)
        .cos()
        .max(0.0)
        .powf(2.2);

    strata_mask * (ledge * 0.020 + crag_high * 0.020 - crag_cut * 0.018)
}

fn terrain_micro_relief_m(radius: f32, angle: f32, phase: f32) -> f32 {
    let detail_mask = smoothstep(0.12, 0.46, radius) * (1.0 - smoothstep(0.94, 1.0, radius));
    let fine = (angle * 17.0 + radius * 11.0 + phase).sin() * 0.026
        + (angle * 23.0 - radius * 7.0 - phase * 0.8).cos() * 0.018
        + (angle * 31.0 + radius * 19.0 + phase * 0.3).sin() * 0.012;

    fine * detail_mask
}

fn terrain_step(radius: f32, start: f32, end: f32, height_m: f32) -> f32 {
    smoothstep(start, end, radius) * height_m - smoothstep(end, 1.0, radius) * height_m * 0.5
}

fn basin(radius: f32, center_radius: f32, depth_m: f32) -> f32 {
    let distance = ((radius - center_radius).abs() / center_radius.max(0.001)).clamp(0.0, 1.0);
    (1.0 - distance).powf(1.7) * depth_m
}

fn angular_lobe(angle: f32, center: f32, width: f32) -> f32 {
    let diff = (angle - center + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    (1.0 - diff.abs() / width.max(0.001)).clamp(0.0, 1.0)
}

fn normalized_angle(angle: f32) -> f32 {
    angle.rem_euclid(std::f32::consts::TAU)
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
