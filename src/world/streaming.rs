use bevy::prelude::Vec3;

use super::{STREAM_ACTIVE_CHUNK_RADIUS, STREAM_CHUNK_SIZE_M};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StreamChunkCoord {
    pub x: i32,
    pub z: i32,
}

impl StreamChunkCoord {
    pub fn from_world(position: Vec3) -> Self {
        Self {
            x: (position.x / STREAM_CHUNK_SIZE_M).floor() as i32,
            z: (position.z / STREAM_CHUNK_SIZE_M).floor() as i32,
        }
    }

    pub fn is_inside_active_window(self, center: Self) -> bool {
        (self.x - center.x).abs() <= STREAM_ACTIVE_CHUNK_RADIUS
            && (self.z - center.z).abs() <= STREAM_ACTIVE_CHUNK_RADIUS
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StreamingLodStats {
    pub player_chunk: StreamChunkCoord,
    pub active_chunk_count: usize,
    pub active_island_count: usize,
    pub near_lod_islands: usize,
    pub mid_lod_islands: usize,
    pub far_lod_islands: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LodBand {
    Near,
    Mid,
    Far,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamActivation {
    Active,
    Inactive,
}

impl StreamActivation {
    pub fn is_active(self) -> bool {
        self == Self::Active
    }
}
