use bevy::prelude::{Component, Vec2, Vec3};

use super::{
    LOD_MID_DISTANCE_M, LOD_NEAR_DISTANCE_M, LodBand, PLAYER_STANDING_OFFSET, StreamActivation,
    StreamChunkCoord, TERRAIN_MAX_DROP_M, TERRAIN_MAX_RISE_M, TERRAIN_VISUAL_FOOTING_OFFSET_M,
};

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
}

impl IslandTerrainArchetype {
    pub const COUNT: usize = 14;

    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "launch mesa" => Some(Self::LaunchMesa),
            "midpoint shelf" => Some(Self::Shelf),
            "landing garden" => Some(Self::GardenBasin),
            "distant crown" => Some(Self::CrownRidge),
            "wind overlook" => Some(Self::WindOverlook),
            "copper stair" | "sunlit terrace" => Some(Self::TerracedSpur),
            "western refuge" => Some(Self::RefugeTableland),
            "storm porch" => Some(Self::StormRavine),
            "high orchard" => Some(Self::OrchardBasin),
            "far needle" => Some(Self::Needle),
            "sapphire basin" => Some(Self::SapphireBasin),
            "mist arch" => Some(Self::MistArch),
            "broken stair" => Some(Self::BrokenStair),
            "cloud gate" => Some(Self::CloudGate),
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
            Self::Needle => (1.0 - radius).powf(1.8) * 0.26 - smoothstep(0.56, 1.0, radius) * 0.18,
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
        }
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
}

impl SkyIsland {
    pub fn new(
        name: &'static str,
        center: Vec3,
        half_extents: Vec2,
        thickness: f32,
        is_target: bool,
    ) -> Self {
        Self {
            name,
            center,
            half_extents,
            thickness: thickness.max(1.0),
            is_target,
            terrain_archetype: IslandTerrainArchetype::for_name_or_default(name),
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

        let phase = self.terrain_phase();
        let ridge = radius
            * ((angle * 3.0 + phase).sin() * 0.28 + (angle * 7.0 - phase * 0.5).cos() * 0.14);
        let shoulder = (radius * std::f32::consts::PI).sin() * 0.24;
        let center_falloff = ((1.0 - radius).powi(2) - 1.0) * 0.16;
        let edge_drop = -radius.powf(2.35) * 0.42;
        let ravines = terrain_ravine_relief_m(radius, angle, phase);
        let terrace = terrain_terrace_relief_m(radius, angle, phase);
        let micro = terrain_micro_relief_m(radius, angle, phase);
        let archetype = self.terrain_archetype.relief_bias_m(radius, angle, phase);

        (ridge + shoulder + center_falloff + edge_drop + ravines + terrace + micro + archetype)
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

    pub fn lod_band(self, position: Vec3) -> LodBand {
        let distance = self.horizontal_distance(position);
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
        let phase = self.terrain_phase();
        (1.0 + 0.09 * (angle * 3.0 + phase).sin()
            + 0.055 * (angle * 7.0 - phase * 0.4).cos()
            + 0.032 * (angle * 11.0 + phase * 1.7).sin()
            + self.terrain_archetype.silhouette_bias(angle, phase))
        .clamp(0.68, 1.28)
    }

    pub fn playable_silhouette_scale(self, angle: f32) -> f32 {
        self.visual_silhouette_scale(angle).clamp(0.62, 1.0)
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

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
