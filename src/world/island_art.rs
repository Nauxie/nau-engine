use super::route::SKY_ROUTE_ISLAND_COUNT;
#[cfg(test)]
use super::route::SkyRoute;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandPaletteFamily {
    VerdantSun,
    CopperOrchard,
    StormSlate,
    MistJade,
    SapphireWetland,
    AlpineFrost,
    RuinOchre,
    CloudSilver,
    PlateauBloom,
}

impl IslandPaletteFamily {
    pub fn label(self) -> &'static str {
        match self {
            Self::VerdantSun => "verdant_sun",
            Self::CopperOrchard => "copper_orchard",
            Self::StormSlate => "storm_slate",
            Self::MistJade => "mist_jade",
            Self::SapphireWetland => "sapphire_wetland",
            Self::AlpineFrost => "alpine_frost",
            Self::RuinOchre => "ruin_ochre",
            Self::CloudSilver => "cloud_silver",
            Self::PlateauBloom => "plateau_bloom",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandSurfacePattern {
    TerracedCourt,
    BraidedCauseway,
    RadialGarden,
    CrownedRidge,
    WindwardRibs,
    OrchardRows,
    NeedleHalo,
    BasinRings,
    ProcessionalAxis,
    PortalCourt,
    UnderhangThreshold,
    ThermalSpiral,
    CascadeTerraces,
    PlateauDistricts,
    SummitSanctum,
}

impl IslandSurfacePattern {
    pub fn label(self) -> &'static str {
        match self {
            Self::TerracedCourt => "terraced_court",
            Self::BraidedCauseway => "braided_causeway",
            Self::RadialGarden => "radial_garden",
            Self::CrownedRidge => "crowned_ridge",
            Self::WindwardRibs => "windward_ribs",
            Self::OrchardRows => "orchard_rows",
            Self::NeedleHalo => "needle_halo",
            Self::BasinRings => "basin_rings",
            Self::ProcessionalAxis => "processional_axis",
            Self::PortalCourt => "portal_court",
            Self::UnderhangThreshold => "underhang_threshold",
            Self::ThermalSpiral => "thermal_spiral",
            Self::CascadeTerraces => "cascade_terraces",
            Self::PlateauDistricts => "plateau_districts",
            Self::SummitSanctum => "summit_sanctum",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandFloraIdentity {
    FernGrove,
    FlowerThicket,
    ReedBed,
    WindShrub,
    BroadleafPatch,
    MushroomRing,
}

impl IslandFloraIdentity {
    pub fn label(self) -> &'static str {
        match self {
            Self::FernGrove => "fern_grove",
            Self::FlowerThicket => "flower_thicket",
            Self::ReedBed => "reed_bed",
            Self::WindShrub => "wind_shrub",
            Self::BroadleafPatch => "broadleaf_patch",
            Self::MushroomRing => "mushroom_ring",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandFormationIdentity {
    BasaltCrown,
    WeatheredArch,
    BoulderSpine,
    StackedMonoliths,
    CrystalOutcrop,
}

impl IslandFormationIdentity {
    pub fn label(self) -> &'static str {
        match self {
            Self::BasaltCrown => "basalt_crown",
            Self::WeatheredArch => "weathered_arch",
            Self::BoulderSpine => "boulder_spine",
            Self::StackedMonoliths => "stacked_monoliths",
            Self::CrystalOutcrop => "crystal_outcrop",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandRuinIdentity {
    Colonnade,
    SunkenSanctum,
    Watchtower,
    BrokenAqueduct,
    ProcessionalStairs,
}

impl IslandRuinIdentity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Colonnade => "colonnade",
            Self::SunkenSanctum => "sunken_sanctum",
            Self::Watchtower => "watchtower",
            Self::BrokenAqueduct => "broken_aqueduct",
            Self::ProcessionalStairs => "processional_stairs",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandWaterStory {
    DryWindCarved,
    SpringPond,
    ReflectingBasin,
    ReedyLake,
    CascadeRun,
    WaterfallGarden,
    MistPool,
    CaveSeep,
}

impl IslandWaterStory {
    pub fn label(self) -> &'static str {
        match self {
            Self::DryWindCarved => "dry_wind_carved",
            Self::SpringPond => "spring_pond",
            Self::ReflectingBasin => "reflecting_basin",
            Self::ReedyLake => "reedy_lake",
            Self::CascadeRun => "cascade_run",
            Self::WaterfallGarden => "waterfall_garden",
            Self::MistPool => "mist_pool",
            Self::CaveSeep => "cave_seep",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandHeroLandmark {
    BeaconCourt,
    CairnCauseway,
    BloomAmphitheater,
    CrownObservatory,
    AeolianHarp,
    CopperArcade,
    SolarGrove,
    RefugeCanopy,
    StormTotemField,
    OrchardPergola,
    NeedleHalo,
    SapphireShrine,
    FracturedProcession,
    VeiledPortal,
    CloudGateChoir,
    FlightTrainingCircle,
    PetalArcade,
    LightningMonolith,
    ArborCrown,
    FogCairn,
    RootedThreshold,
    WindbreakRibs,
    MoonGardenShrine,
    CatchSailTerrace,
    ThermalOrrery,
    CrownletSeat,
    SkyhookAqueduct,
    StratosLookout,
    CascadeTemple,
    PilgrimGate,
    RoostSpire,
    SkyforgeAnvil,
    WaystoneGallery,
    ChimeColonnade,
    BluevaultSanctuary,
    SwitchbackBastion,
    SolarOrrery,
    CloudbreakPortal,
    SkyCitadel,
    HorizonLens,
    ZenithSanctum,
}

impl IslandHeroLandmark {
    pub fn label(self) -> &'static str {
        match self {
            Self::BeaconCourt => "beacon_court",
            Self::CairnCauseway => "cairn_causeway",
            Self::BloomAmphitheater => "bloom_amphitheater",
            Self::CrownObservatory => "crown_observatory",
            Self::AeolianHarp => "aeolian_harp",
            Self::CopperArcade => "copper_arcade",
            Self::SolarGrove => "solar_grove",
            Self::RefugeCanopy => "refuge_canopy",
            Self::StormTotemField => "storm_totem_field",
            Self::OrchardPergola => "orchard_pergola",
            Self::NeedleHalo => "needle_halo",
            Self::SapphireShrine => "sapphire_shrine",
            Self::FracturedProcession => "fractured_procession",
            Self::VeiledPortal => "veiled_portal",
            Self::CloudGateChoir => "cloud_gate_choir",
            Self::FlightTrainingCircle => "flight_training_circle",
            Self::PetalArcade => "petal_arcade",
            Self::LightningMonolith => "lightning_monolith",
            Self::ArborCrown => "arbor_crown",
            Self::FogCairn => "fog_cairn",
            Self::RootedThreshold => "rooted_threshold",
            Self::WindbreakRibs => "windbreak_ribs",
            Self::MoonGardenShrine => "moon_garden_shrine",
            Self::CatchSailTerrace => "catch_sail_terrace",
            Self::ThermalOrrery => "thermal_orrery",
            Self::CrownletSeat => "crownlet_seat",
            Self::SkyhookAqueduct => "skyhook_aqueduct",
            Self::StratosLookout => "stratos_lookout",
            Self::CascadeTemple => "cascade_temple",
            Self::PilgrimGate => "pilgrim_gate",
            Self::RoostSpire => "roost_spire",
            Self::SkyforgeAnvil => "skyforge_anvil",
            Self::WaystoneGallery => "waystone_gallery",
            Self::ChimeColonnade => "chime_colonnade",
            Self::BluevaultSanctuary => "bluevault_sanctuary",
            Self::SwitchbackBastion => "switchback_bastion",
            Self::SolarOrrery => "solar_orrery",
            Self::CloudbreakPortal => "cloudbreak_portal",
            Self::SkyCitadel => "sky_citadel",
            Self::HorizonLens => "horizon_lens",
            Self::ZenithSanctum => "zenith_sanctum",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IslandArtDirection {
    pub island_name: &'static str,
    pub epithet: &'static str,
    pub environmental_story: &'static str,
    pub palette_family: IslandPaletteFamily,
    pub palette_hue_shift_degrees: i16,
    pub palette_warmth_percent: i8,
    pub terrain_contrast_percent: u8,
    pub surface_pattern: IslandSurfacePattern,
    pub hero_landmark: IslandHeroLandmark,
    pub hero_anchor: [f32; 2],
    pub hero_rotation_degrees: i16,
    pub hero_scale: f32,
    pub flora_kinds: [IslandFloraIdentity; 3],
    pub flora_count: u8,
    pub flora_anchor: [f32; 2],
    pub formation_kinds: [IslandFormationIdentity; 2],
    pub formation_count: u8,
    pub formation_anchor: [f32; 2],
    pub ruin_kinds: [IslandRuinIdentity; 2],
    pub ruin_count: u8,
    pub ruin_anchor: [f32; 2],
    pub water_story: IslandWaterStory,
    pub review_heading_degrees: i16,
    pub signature_seed: u32,
}

impl IslandArtDirection {
    pub fn signature(self) -> u64 {
        fn mix_bytes(hash: &mut u64, bytes: &[u8]) {
            for byte in bytes {
                *hash ^= u64::from(*byte);
                *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }

        fn mix_u64(hash: &mut u64, value: u64) {
            mix_bytes(hash, &value.to_le_bytes());
        }

        fn mix_str(hash: &mut u64, value: &str) {
            mix_u64(hash, value.len() as u64);
            mix_bytes(hash, value.as_bytes());
        }

        fn mix_anchor(hash: &mut u64, anchor: [f32; 2]) {
            mix_u64(hash, u64::from(anchor[0].to_bits()));
            mix_u64(hash, u64::from(anchor[1].to_bits()));
        }

        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        mix_str(&mut hash, "nau.island_art_direction.v2");
        mix_str(&mut hash, self.island_name);
        mix_str(&mut hash, self.epithet);
        mix_str(&mut hash, self.environmental_story);
        mix_u64(&mut hash, self.palette_family as u64);
        mix_u64(&mut hash, self.palette_hue_shift_degrees as i64 as u64);
        mix_u64(&mut hash, self.palette_warmth_percent as i64 as u64);
        mix_u64(&mut hash, u64::from(self.terrain_contrast_percent));
        mix_u64(&mut hash, self.surface_pattern as u64);
        mix_u64(&mut hash, self.hero_landmark as u64);
        mix_anchor(&mut hash, self.hero_anchor);
        mix_u64(&mut hash, self.hero_rotation_degrees as i64 as u64);
        mix_u64(&mut hash, u64::from(self.hero_scale.to_bits()));
        for kind in self.flora_kinds {
            mix_u64(&mut hash, kind as u64);
        }
        mix_u64(&mut hash, u64::from(self.flora_count));
        mix_anchor(&mut hash, self.flora_anchor);
        for kind in self.formation_kinds {
            mix_u64(&mut hash, kind as u64);
        }
        mix_u64(&mut hash, u64::from(self.formation_count));
        mix_anchor(&mut hash, self.formation_anchor);
        for kind in self.ruin_kinds {
            mix_u64(&mut hash, kind as u64);
        }
        mix_u64(&mut hash, u64::from(self.ruin_count));
        mix_anchor(&mut hash, self.ruin_anchor);
        mix_u64(&mut hash, self.water_story as u64);
        mix_u64(&mut hash, self.review_heading_degrees as i64 as u64);
        mix_u64(&mut hash, u64::from(self.signature_seed));
        hash
    }
}

#[allow(clippy::too_many_arguments)]
const fn art(
    island_name: &'static str,
    epithet: &'static str,
    environmental_story: &'static str,
    palette_family: IslandPaletteFamily,
    palette_hue_shift_degrees: i16,
    palette_warmth_percent: i8,
    terrain_contrast_percent: u8,
    surface_pattern: IslandSurfacePattern,
    hero_landmark: IslandHeroLandmark,
    hero_anchor: [f32; 2],
    hero_rotation_degrees: i16,
    hero_scale: f32,
    flora_kinds: [IslandFloraIdentity; 3],
    flora_count: u8,
    flora_anchor: [f32; 2],
    formation_kinds: [IslandFormationIdentity; 2],
    formation_count: u8,
    formation_anchor: [f32; 2],
    ruin_kinds: [IslandRuinIdentity; 2],
    ruin_count: u8,
    ruin_anchor: [f32; 2],
    water_story: IslandWaterStory,
    review_heading_degrees: i16,
    signature_seed: u32,
) -> IslandArtDirection {
    IslandArtDirection {
        island_name,
        epithet,
        environmental_story,
        palette_family,
        palette_hue_shift_degrees,
        palette_warmth_percent,
        terrain_contrast_percent,
        surface_pattern,
        hero_landmark,
        hero_anchor,
        hero_rotation_degrees,
        hero_scale,
        flora_kinds,
        flora_count,
        flora_anchor,
        formation_kinds,
        formation_count,
        formation_anchor,
        ruin_kinds,
        ruin_count,
        ruin_anchor,
        water_story,
        review_heading_degrees,
        signature_seed,
    }
}

use IslandFloraIdentity::{
    BroadleafPatch, FernGrove, FlowerThicket, MushroomRing, ReedBed, WindShrub,
};
use IslandFormationIdentity::{
    BasaltCrown, BoulderSpine, CrystalOutcrop, StackedMonoliths, WeatheredArch,
};
use IslandHeroLandmark::*;
use IslandPaletteFamily::*;
use IslandRuinIdentity::{
    BrokenAqueduct, Colonnade, ProcessionalStairs, SunkenSanctum, Watchtower,
};
use IslandSurfacePattern::*;
use IslandWaterStory::*;

pub const ISLAND_ART_DIRECTIONS: [IslandArtDirection; SKY_ROUTE_ISLAND_COUNT] = [
    art(
        "launch mesa",
        "Beacon Nursery",
        "A sheltered takeoff court where young broad-canopy trees frame the first horizon.",
        VerdantSun,
        -8,
        12,
        76,
        TerracedCourt,
        BeaconCourt,
        [0.18, -0.20],
        12,
        1.08,
        [BroadleafPatch, FlowerThicket, FernGrove],
        3,
        [-0.34, 0.22],
        [BoulderSpine, CrystalOutcrop],
        1,
        [0.52, 0.24],
        [Colonnade, Watchtower],
        1,
        [-0.46, -0.34],
        DryWindCarved,
        28,
        0x1001,
    ),
    art(
        "midpoint shelf",
        "The First Causeway",
        "Braided cairn paths split around a wind-smoothed shelf and reunite at the route edge.",
        CloudSilver,
        -3,
        -4,
        68,
        BraidedCauseway,
        CairnCauseway,
        [0.10, 0.08],
        -24,
        0.94,
        [FernGrove, WindShrub, BroadleafPatch],
        2,
        [-0.28, 0.24],
        [BoulderSpine, WeatheredArch],
        1,
        [0.48, -0.24],
        [ProcessionalStairs, Colonnade],
        0,
        [-0.42, -0.34],
        DryWindCarved,
        -18,
        0x1002,
    ),
    art(
        "landing garden",
        "Bloom Amphitheater",
        "A safe circular landing lawn descends through flower tiers into a reflective spring.",
        PlateauBloom,
        7,
        18,
        64,
        RadialGarden,
        BloomAmphitheater,
        [0.04, -0.04],
        8,
        1.10,
        [FlowerThicket, BroadleafPatch, MushroomRing],
        3,
        [-0.38, 0.18],
        [CrystalOutcrop, BoulderSpine],
        1,
        [0.46, 0.28],
        [SunkenSanctum, Colonnade],
        1,
        [-0.28, -0.42],
        SpringPond,
        42,
        0x1003,
    ),
    art(
        "distant crown",
        "Crown Observatory",
        "An abandoned skywatch stands between alpine fins that form the route's first crown.",
        AlpineFrost,
        -10,
        -12,
        88,
        CrownedRidge,
        CrownObservatory,
        [0.02, 0.10],
        34,
        1.18,
        [WindShrub, MushroomRing, FernGrove],
        2,
        [-0.42, -0.18],
        [StackedMonoliths, CrystalOutcrop],
        2,
        [0.44, 0.12],
        [Watchtower, Colonnade],
        1,
        [-0.10, -0.48],
        DryWindCarved,
        66,
        0x1004,
    ),
    art(
        "wind overlook",
        "Aeolian Harp",
        "Stone ribs and weathered strings turn the western crosswind into a visible instrument.",
        StormSlate,
        4,
        -18,
        92,
        WindwardRibs,
        AeolianHarp,
        [0.20, 0.02],
        -12,
        1.02,
        [WindShrub, FernGrove, BroadleafPatch],
        2,
        [-0.36, 0.22],
        [BoulderSpine, BasaltCrown],
        1,
        [0.48, -0.28],
        [Colonnade, Watchtower],
        0,
        [-0.44, -0.30],
        DryWindCarved,
        -58,
        0x1005,
    ),
    art(
        "copper stair",
        "Copper Arcade",
        "Oxidized stair arcades climb through amber grass toward the orchard rise.",
        RuinOchre,
        11,
        22,
        84,
        ProcessionalAxis,
        CopperArcade,
        [0.12, -0.08],
        38,
        1.08,
        [FernGrove, MushroomRing, FlowerThicket],
        2,
        [-0.40, 0.26],
        [BoulderSpine, StackedMonoliths],
        1,
        [0.48, 0.22],
        [ProcessionalStairs, Colonnade],
        2,
        [-0.22, -0.42],
        DryWindCarved,
        22,
        0x1006,
    ),
    art(
        "sunlit terrace",
        "Solar Grove",
        "An open sundial grove shelters a recovery terrace beneath warm orchard canopies.",
        CopperOrchard,
        5,
        28,
        62,
        OrchardRows,
        SolarGrove,
        [0.08, 0.14],
        -16,
        1.04,
        [BroadleafPatch, FlowerThicket, MushroomRing],
        3,
        [-0.42, -0.10],
        [CrystalOutcrop, BoulderSpine],
        1,
        [0.50, -0.16],
        [Colonnade, SunkenSanctum],
        0,
        [-0.20, 0.46],
        SpringPond,
        -34,
        0x1007,
    ),
    art(
        "western refuge",
        "Refuge Canopy",
        "A low stone shelter and wind-bent grove create a deliberate safe harbor for missed glides.",
        VerdantSun,
        -14,
        -6,
        70,
        TerracedCourt,
        RefugeCanopy,
        [0.02, 0.06],
        52,
        0.98,
        [FernGrove, BroadleafPatch, WindShrub],
        2,
        [-0.36, -0.28],
        [WeatheredArch, BoulderSpine],
        1,
        [0.46, 0.26],
        [Watchtower, Colonnade],
        1,
        [-0.42, 0.24],
        DryWindCarved,
        74,
        0x1008,
    ),
    art(
        "storm porch",
        "Storm Totem Field",
        "Black basalt totems lean with the prevailing storm and mark a hard-edged porch.",
        StormSlate,
        -8,
        -24,
        100,
        WindwardRibs,
        StormTotemField,
        [0.16, -0.10],
        -42,
        1.12,
        [WindShrub, FernGrove, MushroomRing],
        2,
        [-0.40, 0.16],
        [BasaltCrown, BoulderSpine],
        2,
        [0.46, 0.20],
        [Watchtower, Colonnade],
        1,
        [-0.28, -0.44],
        DryWindCarved,
        -76,
        0x1009,
    ),
    art(
        "high orchard",
        "Orchard Pergola",
        "Long fruit-tree rows converge on a ruined pergola overlooking the first lake route.",
        CopperOrchard,
        -4,
        24,
        66,
        OrchardRows,
        OrchardPergola,
        [0.06, 0.10],
        14,
        1.14,
        [BroadleafPatch, FlowerThicket, MushroomRing],
        3,
        [-0.46, 0.04],
        [BoulderSpine, CrystalOutcrop],
        1,
        [0.50, -0.12],
        [Colonnade, SunkenSanctum],
        1,
        [-0.18, -0.48],
        SpringPond,
        36,
        0x100a,
    ),
    art(
        "far needle",
        "Needle Halo",
        "A ring of fractured monoliths makes the thin alpine needle legible from the basin below.",
        AlpineFrost,
        8,
        -20,
        104,
        IslandSurfacePattern::NeedleHalo,
        IslandHeroLandmark::NeedleHalo,
        [0.00, 0.08],
        0,
        1.18,
        [WindShrub, MushroomRing, FernGrove],
        2,
        [-0.44, -0.20],
        [StackedMonoliths, CrystalOutcrop],
        2,
        [0.38, 0.20],
        [Watchtower, Colonnade],
        0,
        [-0.20, -0.46],
        DryWindCarved,
        88,
        0x100b,
    ),
    art(
        "sapphire basin",
        "Sapphire Shrine",
        "Willows and reed terraces encircle a blue basin shrine built at the mistward shore.",
        SapphireWetland,
        -6,
        -8,
        72,
        BasinRings,
        SapphireShrine,
        [0.18, 0.04],
        -30,
        1.16,
        [ReedBed, FlowerThicket, FernGrove],
        3,
        [-0.40, 0.22],
        [CrystalOutcrop, BoulderSpine],
        2,
        [0.46, -0.20],
        [SunkenSanctum, Colonnade],
        1,
        [-0.24, -0.44],
        ReedyLake,
        -46,
        0x100c,
    ),
    art(
        "broken stair",
        "Fractured Procession",
        "A ceremonial stair has split into parallel ruin lines threaded by ferns and fallen stones.",
        RuinOchre,
        -12,
        4,
        96,
        ProcessionalAxis,
        FracturedProcession,
        [0.10, -0.12],
        44,
        1.14,
        [FernGrove, MushroomRing, WindShrub],
        2,
        [-0.42, 0.16],
        [StackedMonoliths, WeatheredArch],
        2,
        [0.46, 0.22],
        [ProcessionalStairs, Colonnade],
        2,
        [-0.24, -0.46],
        DryWindCarved,
        18,
        0x100d,
    ),
    art(
        "mist arch",
        "Veiled Portal",
        "A moss-dark portal gathers drifting mist around broken aqueduct ribs.",
        MistJade,
        4,
        -12,
        78,
        PortalCourt,
        VeiledPortal,
        [0.06, 0.00],
        -18,
        1.20,
        [MushroomRing, FernGrove, ReedBed],
        3,
        [-0.44, 0.22],
        [WeatheredArch, StackedMonoliths],
        2,
        [0.46, -0.18],
        [BrokenAqueduct, Watchtower],
        2,
        [-0.24, -0.42],
        MistPool,
        -20,
        0x100e,
    ),
    art(
        "cloud gate",
        "Cloud Gate Choir",
        "Paired stone arches and wind chimes turn the route threshold into a resonant gate.",
        CloudSilver,
        10,
        -18,
        86,
        PortalCourt,
        CloudGateChoir,
        [0.08, -0.02],
        26,
        1.18,
        [MushroomRing, WindShrub, FernGrove],
        3,
        [-0.42, -0.18],
        [WeatheredArch, BoulderSpine],
        2,
        [0.46, 0.20],
        [BrokenAqueduct, Colonnade],
        2,
        [-0.20, 0.44],
        DryWindCarved,
        12,
        0x100f,
    ),
    art(
        "launch spur",
        "Flight Training Circle",
        "A compact ring of markers and clipped shrubs creates a readable practice circuit.",
        VerdantSun,
        16,
        10,
        72,
        RadialGarden,
        FlightTrainingCircle,
        [0.02, 0.02],
        -8,
        0.82,
        [WindShrub, FernGrove, FlowerThicket],
        2,
        [-0.36, 0.20],
        [BoulderSpine, CrystalOutcrop],
        1,
        [0.40, -0.22],
        [Colonnade, Watchtower],
        0,
        [-0.28, -0.40],
        DryWindCarved,
        58,
        0x1010,
    ),
    art(
        "garden apron",
        "Petal Arcade",
        "Low flower alleys and a partial arcade invite recovery toward the landing garden above.",
        PlateauBloom,
        -5,
        20,
        62,
        RadialGarden,
        PetalArcade,
        [0.12, 0.02],
        18,
        0.98,
        [FlowerThicket, BroadleafPatch, FernGrove],
        3,
        [-0.42, -0.12],
        [CrystalOutcrop, BoulderSpine],
        1,
        [0.46, 0.20],
        [SunkenSanctum, Colonnade],
        1,
        [-0.18, -0.44],
        SpringPond,
        34,
        0x1011,
    ),
    art(
        "storm shard",
        "Lightning Monolith",
        "A single lightning-split monolith dominates a tiny black-rock stepping stone.",
        StormSlate,
        14,
        -28,
        112,
        CrownedRidge,
        LightningMonolith,
        [0.00, 0.04],
        -36,
        1.02,
        [WindShrub, MushroomRing, FernGrove],
        2,
        [-0.38, 0.18],
        [BasaltCrown, StackedMonoliths],
        2,
        [0.36, -0.18],
        [Watchtower, Colonnade],
        0,
        [-0.24, -0.38],
        DryWindCarved,
        -64,
        0x1012,
    ),
    art(
        "orchard spur",
        "Arbor Crown",
        "A compact orchard arbor crowns the side spur with a warm, recognizable silhouette.",
        CopperOrchard,
        9,
        30,
        60,
        OrchardRows,
        ArborCrown,
        [0.02, 0.08],
        22,
        0.92,
        [BroadleafPatch, FlowerThicket, MushroomRing],
        3,
        [-0.38, -0.18],
        [BoulderSpine, CrystalOutcrop],
        1,
        [0.40, 0.20],
        [Colonnade, SunkenSanctum],
        1,
        [-0.20, -0.40],
        SpringPond,
        48,
        0x1013,
    ),
    art(
        "mist stepping stone",
        "Fog Cairn",
        "A low cairn lantern emerges through silver moss, making the tiny mist crossing unmistakable.",
        MistJade,
        -10,
        -16,
        76,
        BraidedCauseway,
        FogCairn,
        [0.02, 0.02],
        -14,
        0.78,
        [MushroomRing, FernGrove, ReedBed],
        2,
        [-0.34, 0.18],
        [WeatheredArch, StackedMonoliths],
        1,
        [0.36, -0.18],
        [Colonnade, Watchtower],
        0,
        [-0.24, -0.36],
        MistPool,
        -38,
        0x1014,
    ),
    art(
        "underbridge cay",
        "Rooted Threshold",
        "Hanging roots, damp stone ribs, and a cave threshold make the under-route feel inhabited.",
        MistJade,
        12,
        -20,
        88,
        UnderhangThreshold,
        RootedThreshold,
        [0.10, -0.06],
        58,
        1.12,
        [MushroomRing, FernGrove, ReedBed],
        3,
        [-0.40, 0.20],
        [WeatheredArch, BoulderSpine],
        2,
        [0.44, -0.18],
        [BrokenAqueduct, Colonnade],
        1,
        [-0.20, -0.44],
        CaveSeep,
        76,
        0x1015,
    ),
    art(
        "low reef",
        "Windbreak Ribs",
        "Low basalt ribs shelter recovery grass while preserving a clear relaunch axis.",
        StormSlate,
        2,
        -10,
        90,
        WindwardRibs,
        WindbreakRibs,
        [0.14, 0.04],
        -26,
        0.92,
        [ReedBed, WindShrub, FernGrove],
        2,
        [-0.40, -0.16],
        [BasaltCrown, BoulderSpine],
        1,
        [0.42, 0.18],
        [Colonnade, Watchtower],
        0,
        [-0.22, -0.40],
        DryWindCarved,
        -52,
        0x1016,
    ),
    art(
        "quiet lower garden",
        "Moon Garden Shrine",
        "Pale flowers and mushroom rings encircle a quiet moonstone shrine below the main route.",
        PlateauBloom,
        -14,
        4,
        58,
        RadialGarden,
        MoonGardenShrine,
        [0.04, 0.06],
        32,
        1.28,
        [FlowerThicket, MushroomRing, BroadleafPatch],
        3,
        [-0.42, 0.16],
        [CrystalOutcrop, WeatheredArch],
        2,
        [0.44, -0.18],
        [SunkenSanctum, Colonnade],
        2,
        [-0.18, -0.44],
        SpringPond,
        62,
        0x1017,
    ),
    art(
        "lowwind shelf",
        "Catch-Sail Terrace",
        "Stone sail frames catch the low current and point recovering players back toward the climb.",
        CloudSilver,
        -16,
        -8,
        74,
        WindwardRibs,
        CatchSailTerrace,
        [0.12, 0.00],
        18,
        1.24,
        [WindShrub, FernGrove, BroadleafPatch],
        3,
        [-0.38, 0.20],
        [BoulderSpine, BasaltCrown],
        2,
        [0.42, -0.18],
        [Colonnade, Watchtower],
        1,
        [-0.22, -0.40],
        DryWindCarved,
        24,
        0x1018,
    ),
    art(
        "upper thermal ring",
        "Thermal Orrery",
        "Concentric stone vanes orbit the updraft and turn lift direction into architecture.",
        CloudSilver,
        6,
        -14,
        82,
        ThermalSpiral,
        ThermalOrrery,
        [0.02, 0.04],
        -20,
        1.16,
        [WindShrub, FlowerThicket, FernGrove],
        3,
        [-0.42, -0.16],
        [CrystalOutcrop, StackedMonoliths],
        2,
        [0.44, 0.18],
        [Colonnade, Watchtower],
        1,
        [-0.20, -0.44],
        DryWindCarved,
        -30,
        0x1019,
    ),
    art(
        "needle crownlet",
        "Crownlet Seat",
        "A dangerous stone seat is wedged into a tiny crown of leaning alpine shards.",
        AlpineFrost,
        15,
        -24,
        108,
        IslandSurfacePattern::NeedleHalo,
        CrownletSeat,
        [0.00, 0.04],
        12,
        0.80,
        [WindShrub, MushroomRing, FernGrove],
        2,
        [-0.34, 0.18],
        [StackedMonoliths, CrystalOutcrop],
        2,
        [0.34, -0.16],
        [Watchtower, Colonnade],
        0,
        [-0.22, -0.36],
        DryWindCarved,
        84,
        0x101a,
    ),
    art(
        "skyhook basin",
        "Skyhook Aqueduct",
        "A hooked aqueduct crosses the optional lake basin and ends in a flooded sanctum.",
        SapphireWetland,
        12,
        -6,
        78,
        BasinRings,
        SkyhookAqueduct,
        [0.10, -0.02],
        46,
        1.20,
        [ReedBed, FlowerThicket, FernGrove],
        3,
        [-0.44, 0.18],
        [CrystalOutcrop, WeatheredArch],
        2,
        [0.46, -0.18],
        [BrokenAqueduct, SunkenSanctum],
        2,
        [-0.18, -0.46],
        ReedyLake,
        44,
        0x101b,
    ),
    art(
        "stratos shelf",
        "Stratos Lookout",
        "A broad observatory ledge studies the cloud sea beside a cold, reed-fringed tarn.",
        AlpineFrost,
        -2,
        -10,
        84,
        CrownedRidge,
        StratosLookout,
        [0.14, 0.06],
        -36,
        1.12,
        [WindShrub, FernGrove, MushroomRing],
        2,
        [-0.42, -0.18],
        [BoulderSpine, StackedMonoliths],
        2,
        [0.46, 0.20],
        [Watchtower, Colonnade],
        1,
        [-0.20, -0.44],
        ReflectingBasin,
        -68,
        0x101c,
    ),
    art(
        "cloudfall meadow",
        "Cascade Temple",
        "A meadow temple terraces the stream into a visible source, lip, fall, and plunge story.",
        SapphireWetland,
        5,
        8,
        74,
        CascadeTerraces,
        CascadeTemple,
        [0.08, -0.02],
        24,
        1.26,
        [ReedBed, FlowerThicket, BroadleafPatch],
        3,
        [-0.44, 0.20],
        [CrystalOutcrop, BoulderSpine],
        2,
        [0.46, -0.18],
        [BrokenAqueduct, Colonnade],
        2,
        [-0.18, -0.46],
        WaterfallGarden,
        30,
        0x101d,
    ),
    art(
        "highgate stair",
        "Pilgrim Gate",
        "A high ceremonial gate compresses the ascent before releasing players toward the sunspire.",
        RuinOchre,
        14,
        2,
        98,
        ProcessionalAxis,
        PilgrimGate,
        [0.10, -0.06],
        54,
        1.14,
        [FernGrove, MushroomRing, WindShrub],
        2,
        [-0.42, 0.16],
        [StackedMonoliths, WeatheredArch],
        2,
        [0.46, 0.18],
        [ProcessionalStairs, Colonnade],
        2,
        [-0.20, -0.44],
        DryWindCarved,
        58,
        0x101e,
    ),
    art(
        "thin air roost",
        "Roost Spire",
        "A solitary watch spire and wind-shaped shrubs make the exposed optional perch readable.",
        AlpineFrost,
        -18,
        -26,
        106,
        IslandSurfacePattern::NeedleHalo,
        RoostSpire,
        [0.02, 0.06],
        -10,
        0.94,
        [WindShrub, MushroomRing, FernGrove],
        2,
        [-0.36, -0.18],
        [StackedMonoliths, BasaltCrown],
        2,
        [0.38, 0.18],
        [Watchtower, Colonnade],
        1,
        [-0.20, -0.38],
        DryWindCarved,
        -82,
        0x101f,
    ),
    art(
        "summit anvil",
        "Skyforge Anvil",
        "Basalt forge stones form an anvil silhouette at the start of the upper ascent.",
        StormSlate,
        18,
        -20,
        114,
        CrownedRidge,
        SkyforgeAnvil,
        [0.04, 0.04],
        28,
        1.24,
        [WindShrub, FernGrove, MushroomRing],
        2,
        [-0.42, 0.18],
        [BasaltCrown, StackedMonoliths],
        2,
        [0.44, -0.18],
        [Watchtower, Colonnade],
        1,
        [-0.18, -0.44],
        DryWindCarved,
        16,
        0x1020,
    ),
    art(
        "upper sky shelf",
        "Waystone Gallery",
        "A gallery of tall waystones frames the bluevault approach without obstructing the shelf.",
        CloudSilver,
        18,
        -6,
        82,
        BraidedCauseway,
        WaystoneGallery,
        [0.12, 0.04],
        -28,
        1.12,
        [FernGrove, WindShrub, FlowerThicket],
        2,
        [-0.42, -0.18],
        [BoulderSpine, CrystalOutcrop],
        2,
        [0.46, 0.18],
        [Colonnade, Watchtower],
        1,
        [-0.18, -0.44],
        DryWindCarved,
        -40,
        0x1021,
    ),
    art(
        "east windchain",
        "Chime Colonnade",
        "A broken colonnade hung with wind chimes links the eastern climb into one visual chain.",
        CloudSilver,
        -8,
        -16,
        88,
        WindwardRibs,
        ChimeColonnade,
        [0.10, -0.02],
        40,
        1.16,
        [WindShrub, FlowerThicket, FernGrove],
        3,
        [-0.44, 0.18],
        [BoulderSpine, CrystalOutcrop],
        2,
        [0.46, -0.18],
        [Colonnade, BrokenAqueduct],
        1,
        [-0.18, -0.46],
        DryWindCarved,
        50,
        0x1022,
    ),
    art(
        "bluevault basin",
        "Bluevault Sanctuary",
        "A flooded sanctuary and pale crystal shore turn the final basin into a plateau prologue.",
        SapphireWetland,
        -14,
        -4,
        80,
        BasinRings,
        BluevaultSanctuary,
        [0.08, 0.02],
        -22,
        1.28,
        [ReedBed, FlowerThicket, BroadleafPatch],
        3,
        [-0.46, 0.18],
        [CrystalOutcrop, WeatheredArch],
        2,
        [0.48, -0.16],
        [SunkenSanctum, BrokenAqueduct],
        2,
        [-0.18, -0.48],
        ReedyLake,
        -24,
        0x1023,
    ),
    art(
        "outer switchback",
        "Switchback Bastion",
        "A ruined bastion breaks into staggered landings that mirror the dangerous return path.",
        RuinOchre,
        20,
        -2,
        110,
        ProcessionalAxis,
        SwitchbackBastion,
        [0.10, -0.06],
        62,
        1.18,
        [WindShrub, MushroomRing, FernGrove],
        2,
        [-0.42, 0.18],
        [StackedMonoliths, BoulderSpine],
        2,
        [0.46, -0.18],
        [ProcessionalStairs, Watchtower],
        2,
        [-0.18, -0.46],
        DryWindCarved,
        70,
        0x1024,
    ),
    art(
        "sunspire garden",
        "Solar Orrery",
        "Golden planting rings orbit a sunstone instrument that marks commitment to the plateau climb.",
        PlateauBloom,
        18,
        30,
        68,
        ThermalSpiral,
        SolarOrrery,
        [0.04, 0.04],
        -12,
        1.24,
        [FlowerThicket, BroadleafPatch, MushroomRing],
        3,
        [-0.44, -0.16],
        [CrystalOutcrop, BoulderSpine],
        2,
        [0.46, 0.18],
        [Colonnade, SunkenSanctum],
        2,
        [-0.18, -0.46],
        SpringPond,
        -12,
        0x1025,
    ),
    art(
        "cloudbreak stair",
        "Cloudbreak Portal",
        "A final portal and split stair frame the underside of the great plateau.",
        RuinOchre,
        -18,
        -6,
        104,
        PortalCourt,
        CloudbreakPortal,
        [0.10, -0.04],
        34,
        1.22,
        [FernGrove, MushroomRing, WindShrub],
        2,
        [-0.42, 0.18],
        [WeatheredArch, StackedMonoliths],
        2,
        [0.46, -0.18],
        [ProcessionalStairs, BrokenAqueduct],
        2,
        [-0.18, -0.46],
        DryWindCarved,
        38,
        0x1026,
    ),
    art(
        "great sky plateau",
        "Sky Citadel",
        "Distinct meadow, basin, shelf, ruin, cave, river, and waterfall districts form a complete hub.",
        PlateauBloom,
        2,
        10,
        86,
        PlateauDistricts,
        SkyCitadel,
        [-0.18, 0.08],
        16,
        1.42,
        [FlowerThicket, ReedBed, FernGrove],
        3,
        [-0.46, 0.22],
        [BoulderSpine, StackedMonoliths],
        2,
        [0.48, -0.16],
        [Colonnade, Watchtower],
        2,
        [-0.22, -0.48],
        WaterfallGarden,
        18,
        0x1027,
    ),
    art(
        "far horizon perch",
        "Horizon Lens",
        "A circular sighting lens aligns the far perch with the crown and the plateau waterfall.",
        AlpineFrost,
        20,
        -18,
        102,
        CrownedRidge,
        HorizonLens,
        [0.04, 0.06],
        -46,
        1.14,
        [WindShrub, FernGrove, MushroomRing],
        2,
        [-0.40, -0.18],
        [StackedMonoliths, CrystalOutcrop],
        2,
        [0.42, 0.18],
        [Watchtower, Colonnade],
        1,
        [-0.18, -0.42],
        DryWindCarved,
        -74,
        0x1028,
    ),
    art(
        "upper crown",
        "Zenith Sanctum",
        "A wind-carved summit sanctum and crystal crown provide the route's final optional silhouette.",
        AlpineFrost,
        -22,
        -12,
        116,
        SummitSanctum,
        ZenithSanctum,
        [0.02, 0.04],
        6,
        1.32,
        [WindShrub, MushroomRing, FlowerThicket],
        3,
        [-0.42, 0.18],
        [StackedMonoliths, CrystalOutcrop],
        2,
        [0.44, -0.18],
        [Watchtower, Colonnade],
        2,
        [-0.18, -0.44],
        DryWindCarved,
        10,
        0x1029,
    ),
];

pub fn island_art_directions() -> &'static [IslandArtDirection] {
    &ISLAND_ART_DIRECTIONS
}

pub fn authored_island_art_direction(island_name: &str) -> Option<IslandArtDirection> {
    ISLAND_ART_DIRECTIONS
        .iter()
        .copied()
        .find(|profile| profile.island_name == island_name)
}

pub fn authored_island_art_direction_at(index: usize) -> Option<IslandArtDirection> {
    ISLAND_ART_DIRECTIONS.get(index).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_route_island_has_one_ordered_art_direction() {
        let route = SkyRoute::default();
        assert_eq!(route.islands().len(), ISLAND_ART_DIRECTIONS.len());

        for (island, profile) in route.islands().iter().zip(ISLAND_ART_DIRECTIONS) {
            assert_eq!(island.name, profile.island_name);
            assert_eq!(
                authored_island_art_direction(island.name),
                Some(profile),
                "{} should resolve its authored art direction",
                island.name
            );
        }
    }

    #[test]
    fn art_directions_have_unique_identity_and_plan_signatures() {
        let mut names = HashSet::new();
        let mut epithets = HashSet::new();
        let mut hero_landmarks = HashSet::new();
        let mut signatures = HashSet::new();
        let mut seeds = HashSet::new();

        for profile in ISLAND_ART_DIRECTIONS {
            assert!(names.insert(profile.island_name));
            assert!(epithets.insert(profile.epithet));
            assert!(hero_landmarks.insert(profile.hero_landmark));
            assert!(signatures.insert(profile.signature()));
            assert!(seeds.insert(profile.signature_seed));
            assert!(!profile.environmental_story.trim().is_empty());
        }
    }

    #[test]
    fn art_direction_signature_covers_every_authored_field() {
        let original = ISLAND_ART_DIRECTIONS[0];
        let original_signature = original.signature();
        let assert_changed = |field: &str, changed: IslandArtDirection| {
            assert_ne!(
                changed.signature(),
                original_signature,
                "{field} must affect the art-direction signature"
            );
        };

        assert_changed(
            "island_name",
            IslandArtDirection {
                island_name: "changed island",
                ..original
            },
        );
        assert_changed(
            "epithet",
            IslandArtDirection {
                epithet: "changed epithet",
                ..original
            },
        );
        assert_changed(
            "environmental_story",
            IslandArtDirection {
                environmental_story: "changed story",
                ..original
            },
        );
        assert_changed(
            "palette_family",
            IslandArtDirection {
                palette_family: IslandPaletteFamily::StormSlate,
                ..original
            },
        );
        assert_changed(
            "palette_hue_shift_degrees",
            IslandArtDirection {
                palette_hue_shift_degrees: original.palette_hue_shift_degrees + 1,
                ..original
            },
        );
        assert_changed(
            "palette_warmth_percent",
            IslandArtDirection {
                palette_warmth_percent: original.palette_warmth_percent + 1,
                ..original
            },
        );
        assert_changed(
            "terrain_contrast_percent",
            IslandArtDirection {
                terrain_contrast_percent: original.terrain_contrast_percent + 1,
                ..original
            },
        );
        assert_changed(
            "surface_pattern",
            IslandArtDirection {
                surface_pattern: IslandSurfacePattern::BraidedCauseway,
                ..original
            },
        );
        assert_changed(
            "hero_landmark",
            IslandArtDirection {
                hero_landmark: IslandHeroLandmark::SolarOrrery,
                ..original
            },
        );
        assert_changed(
            "hero_anchor",
            IslandArtDirection {
                hero_anchor: [original.hero_anchor[0] + 0.01, original.hero_anchor[1]],
                ..original
            },
        );
        assert_changed(
            "hero_rotation_degrees",
            IslandArtDirection {
                hero_rotation_degrees: original.hero_rotation_degrees + 1,
                ..original
            },
        );
        assert_changed(
            "hero_scale",
            IslandArtDirection {
                hero_scale: original.hero_scale + 0.01,
                ..original
            },
        );
        assert_changed(
            "flora_kinds",
            IslandArtDirection {
                flora_kinds: [
                    IslandFloraIdentity::MushroomRing,
                    original.flora_kinds[1],
                    original.flora_kinds[2],
                ],
                ..original
            },
        );
        assert_changed(
            "flora_count",
            IslandArtDirection {
                flora_count: original.flora_count + 1,
                ..original
            },
        );
        assert_changed(
            "flora_anchor",
            IslandArtDirection {
                flora_anchor: [original.flora_anchor[0], original.flora_anchor[1] + 0.01],
                ..original
            },
        );
        assert_changed(
            "formation_kinds",
            IslandArtDirection {
                formation_kinds: [
                    IslandFormationIdentity::CrystalOutcrop,
                    original.formation_kinds[1],
                ],
                ..original
            },
        );
        assert_changed(
            "formation_count",
            IslandArtDirection {
                formation_count: original.formation_count + 1,
                ..original
            },
        );
        assert_changed(
            "formation_anchor",
            IslandArtDirection {
                formation_anchor: [
                    original.formation_anchor[0] + 0.01,
                    original.formation_anchor[1],
                ],
                ..original
            },
        );
        assert_changed(
            "ruin_kinds",
            IslandArtDirection {
                ruin_kinds: [IslandRuinIdentity::BrokenAqueduct, original.ruin_kinds[1]],
                ..original
            },
        );
        assert_changed(
            "ruin_count",
            IslandArtDirection {
                ruin_count: original.ruin_count + 1,
                ..original
            },
        );
        assert_changed(
            "ruin_anchor",
            IslandArtDirection {
                ruin_anchor: [original.ruin_anchor[0], original.ruin_anchor[1] + 0.01],
                ..original
            },
        );
        assert_changed(
            "water_story",
            IslandArtDirection {
                water_story: IslandWaterStory::WaterfallGarden,
                ..original
            },
        );
        assert_changed(
            "review_heading_degrees",
            IslandArtDirection {
                review_heading_degrees: original.review_heading_degrees + 1,
                ..original
            },
        );
        assert_changed(
            "signature_seed",
            IslandArtDirection {
                signature_seed: original.signature_seed + 1,
                ..original
            },
        );
    }

    #[test]
    fn every_profile_exceeds_the_old_generic_minimum_detail_contract() {
        for profile in ISLAND_ART_DIRECTIONS {
            assert!(
                (2..=3).contains(&profile.flora_count),
                "{} must author at least two flora clusters",
                profile.island_name
            );
            assert!(
                (1..=2).contains(&profile.formation_count),
                "{} must author at least one geological formation",
                profile.island_name
            );
            assert!(profile.ruin_count <= 2);
            assert!((0.72..=1.5).contains(&profile.hero_scale));

            for anchor in [
                profile.hero_anchor,
                profile.flora_anchor,
                profile.formation_anchor,
                profile.ruin_anchor,
            ] {
                assert!(
                    anchor[0].abs() <= 0.70 && anchor[1].abs() <= 0.70,
                    "{} has an out-of-bounds authored anchor {anchor:?}",
                    profile.island_name
                );
            }
        }
    }

    #[test]
    fn profile_palette_signatures_do_not_repeat() {
        let mut signatures = HashSet::new();
        for profile in ISLAND_ART_DIRECTIONS {
            let signature = (
                profile.palette_family,
                profile.palette_hue_shift_degrees,
                profile.palette_warmth_percent,
                profile.terrain_contrast_percent,
            );
            assert!(
                signatures.insert(signature),
                "{} repeats another island's palette signature",
                profile.island_name
            );
        }
    }
}
