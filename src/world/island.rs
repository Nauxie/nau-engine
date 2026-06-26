use bevy::prelude::{Component, Vec2, Vec3};

use super::{
    LOD_MID_DISTANCE_M, LOD_NEAR_DISTANCE_M, LodBand, PLAYER_STANDING_OFFSET, StreamActivation,
    StreamChunkCoord, TERRAIN_MAX_DROP_M, TERRAIN_MAX_RISE_M, TERRAIN_VISUAL_FOOTING_OFFSET_M,
};

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SkyIsland {
    pub name: &'static str,
    pub center: Vec3,
    pub half_extents: Vec2,
    pub thickness: f32,
    pub is_target: bool,
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
        }
    }

    pub fn floor_y(self) -> f32 {
        self.center.y
    }

    pub fn mesh_top_y(self) -> f32 {
        self.floor_y() - PLAYER_STANDING_OFFSET
    }

    pub fn terrain_surface_y_at(self, position: Vec3) -> f32 {
        let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
        let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
        let radius = Vec2::new(dx, dz).length().clamp(0.0, 1.0);
        let angle = dz.atan2(dx);

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

        (ridge + shoulder + center_falloff + edge_drop + ravines + terrace + micro)
            .clamp(-TERRAIN_MAX_DROP_M, TERRAIN_MAX_RISE_M)
    }

    pub fn contains_horizontal(self, position: Vec3) -> bool {
        let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
        let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
        dx * dx + dz * dz <= 1.0
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

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
