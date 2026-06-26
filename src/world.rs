mod island;
mod objectives;
mod route;
mod streaming;
mod surface;

#[cfg(test)]
mod tests;

use bevy::prelude::Vec3;

pub use island::{IslandTerrainArchetype, SkyIsland};
pub use objectives::{RouteObjective, RouteObjectiveKind, is_recovery_branch_island};
pub use route::SkyRoute;
pub use streaming::{LodBand, StreamActivation, StreamChunkCoord, StreamingLodStats};
pub use surface::GroundSurface;

pub const PLAYER_STANDING_OFFSET: f32 = 0.24;
pub const START_FLOOR_Y: f32 = 28.0;
pub const START_POSITION: Vec3 = Vec3::new(0.0, START_FLOOR_Y, 0.0);
pub const RECOVERY_BRANCH_ISLANDS: [&str; 2] = ["sunlit terrace", "western refuge"];
pub const STREAM_CHUNK_SIZE_M: f32 = 160.0;
pub const STREAM_ACTIVE_CHUNK_RADIUS: i32 = 2;
pub const LOD_NEAR_DISTANCE_M: f32 = 220.0;
pub const LOD_MID_DISTANCE_M: f32 = 520.0;
pub const TERRAIN_MAX_RISE_M: f32 = 0.45;
pub const TERRAIN_MAX_DROP_M: f32 = 0.75;
pub const TERRAIN_VISUAL_FOOTING_OFFSET_M: f32 = 0.18;

const GROUND_CONTACT_EPSILON: f32 = 0.05;
const GROUND_CONTACT_HORIZONTAL_DAMPING: f32 = 0.58;
