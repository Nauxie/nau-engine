mod collision;
mod island;
mod island_art;
mod island_review;
mod objectives;
mod obstruction_spires;
mod route;
mod streaming;
mod surface;
mod terrain;
mod water_story;

#[cfg(test)]
mod tests;

use bevy::prelude::Vec3;

pub use collision::{
    TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND, TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND,
    TerrainCollisionTruthReport, WorldCollisionProxy, WorldCollisionProxyKind,
    WorldCollisionResolution, resolve_world_collisions, terrain_body_collision_proxies,
    terrain_collision_contact_probe_position, terrain_collision_truth_report,
    terrain_rim_collision_proxies,
};
pub use island::{
    ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT, IslandBiome, IslandFootprintProfile, IslandLandmarkRole,
    IslandPlateauRegion, IslandRouteRole, IslandScaleClass, IslandTerrainArchetype,
    IslandUnderRouteSegment, IslandVerticalProfile, IslandWaterFeature, IslandWorldTags, SkyIsland,
};
pub use island_art::{
    ISLAND_ART_DIRECTIONS, IslandArtDirection, IslandFloraIdentity, IslandFormationIdentity,
    IslandHeroLandmark, IslandPaletteFamily, IslandRuinIdentity, IslandSurfacePattern,
    IslandWaterStory, authored_island_art_direction, authored_island_art_direction_at,
    island_art_directions,
};
pub use island_review::{
    ISLAND_REVIEW_VIEWS_PER_ISLAND, IslandReviewPlan, IslandReviewPose, IslandReviewSpec,
    IslandReviewView,
};
pub use objectives::{RouteObjective, RouteObjectiveKind, is_recovery_branch_island};
pub use obstruction_spires::{
    ROUTE_OBSTRUCTION_SPIRES_PER_ISLAND, RouteObstructionSpire, route_obstruction_spire,
    route_obstruction_spires,
};
pub use route::{
    FirstExpeditionAltitudeBand, FirstExpeditionBeatKind, FirstExpeditionDetourKind,
    FirstExpeditionDetourRisk, FirstExpeditionNavigationContext, FirstExpeditionNavigationLandmark,
    FirstExpeditionNavigationLandmarkKind, FirstExpeditionOptionalDetour,
    FirstExpeditionRecoveryAffordance, FirstExpeditionRouteBeat, FirstExpeditionTraversalMode,
    IslandComposition, IslandCompositionFamily, IslandTraversalPurpose, IslandVisualMotif,
    PLAYTEST_RESET_ISLAND_NAME, SKY_ROUTE_ISLAND_COUNT, SkyRoute, authored_island_composition,
};
pub use streaming::{LodBand, StreamActivation, StreamChunkCoord, StreamingLodStats};
pub use surface::GroundSurface;
pub use terrain::{
    WORLD_TERRAIN_GRID_SPACING_M, WORLD_TERRAIN_GRID_SUBDIVISIONS, WORLD_TERRAIN_TILE_SIZE_M,
    world_terrain_floor_y_at, world_terrain_visual_y_at,
};
pub use water_story::{
    ROUTE_EDGE_WATERFALL_CHANNEL_OUTLET_OFFSET, RouteEdgeWaterfallPlacement,
    route_edge_waterfall_placement,
};

pub const PLAYER_STANDING_OFFSET: f32 = 0.24;
pub const START_FLOOR_Y: f32 = 28.0;
pub const START_POSITION: Vec3 = Vec3::new(0.0, START_FLOOR_Y, 0.0);
pub const RECOVERY_BRANCH_ISLANDS: [&str; 2] = ["sunlit terrace", "western refuge"];
pub const STREAM_CHUNK_SIZE_M: f32 = 160.0;
pub const STREAM_ACTIVE_CHUNK_RADIUS: i32 = 2;
pub const LOD_NEAR_DISTANCE_M: f32 = 190.0;
pub const LOD_MID_DISTANCE_M: f32 = 520.0;
pub const TERRAIN_MAX_RISE_M: f32 = 0.45;
pub const TERRAIN_MAX_DROP_M: f32 = 0.75;
pub const TERRAIN_VISUAL_FOOTING_OFFSET_M: f32 = 0.18;

const GROUND_CONTACT_EPSILON: f32 = 0.05;
const GROUND_CONTACT_VERTICAL_CAPTURE_M: f32 = 2.75;
const GROUND_CONTACT_HORIZONTAL_DAMPING: f32 = 0.58;
