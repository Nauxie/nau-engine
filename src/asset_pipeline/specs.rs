use super::{VisualAssetKind, VisualAssetLoadPolicy, VisualAssetResidency, VisualAssetSpec};

pub const PLAYER_ANIMATION_CLIP_NAMES: &[&str] = &[
    "Idle_Loop",
    "Jog_Fwd_Loop",
    "Launch_Start",
    "Fall_Loop",
    "Glide_Loop",
    "Bank_Left",
    "Bank_Right",
    "Dive_Loop",
    "Air_Brake",
    "Land",
];

pub const VISUAL_ASSET_SPECS: [VisualAssetSpec; 9] = [
    VisualAssetSpec {
        kind: VisualAssetKind::PlayerCharacter,
        label: "player character rig",
        gltf_scene_path: "models/player/player.gltf",
        animation_clip_names: PLAYER_ANIMATION_CLIP_NAMES,
        residency: VisualAssetResidency::Always,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::Glider,
        label: "player glider",
        gltf_scene_path: "models/player/glider.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::Always,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandTerrain,
        label: "island terrain kit",
        gltf_scene_path: "models/world/island_terrain.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::StreamWindow,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandFoliage,
        label: "island foliage kit",
        gltf_scene_path: "models/world/foliage.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::NearLod,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandRock,
        label: "island rock kit",
        gltf_scene_path: "models/world/rocks.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::StreamWindow,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandWater,
        label: "pond and water kit",
        gltf_scene_path: "models/world/water.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::NearLod,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::RouteMarker,
        label: "route marker kit",
        gltf_scene_path: "models/world/route_markers.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::Always,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::WeatherLayer,
        label: "weather cloud layer kit",
        gltf_scene_path: "models/world/weather_layers.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::Weather,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::DistantImpostor,
        label: "sky island distant impostor kit",
        gltf_scene_path: "models/world/island_impostors.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::FarLod,
    },
];

pub const VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SPECS.len();
pub const GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SPECS.len();
pub const STREAMING_VISUAL_ASSET_SLOT_COUNT: usize = 6;
pub const ALWAYS_VISUAL_ASSET_SLOT_COUNT: usize = 3;
pub const STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT: usize = 2;
pub const NEAR_LOD_VISUAL_ASSET_SLOT_COUNT: usize = 2;
pub const FAR_LOD_VISUAL_ASSET_SLOT_COUNT: usize = 1;
pub const WEATHER_VISUAL_ASSET_SLOT_COUNT: usize = 1;
pub const DECLARED_VISUAL_ANIMATION_CLIP_COUNT: usize = PLAYER_ANIMATION_CLIP_NAMES.len();
pub const MIN_READY_VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_LOADED_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT: usize = ALWAYS_VISUAL_ASSET_SLOT_COUNT;
pub const MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT: usize =
    STREAMING_VISUAL_ASSET_SLOT_COUNT;
pub const MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_READY_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MAX_MISSING_VISUAL_ASSET_SLOT_COUNT: usize =
    VISUAL_ASSET_SLOT_COUNT - MIN_READY_VISUAL_ASSET_SLOT_COUNT;
pub const MIN_READY_VISUAL_ANIMATION_CLIP_COUNT: usize = DECLARED_VISUAL_ANIMATION_CLIP_COUNT;
pub const MIN_VISUAL_ANIMATION_PLAYER_COUNT: usize = 1;
pub const MIN_VISUAL_ANIMATION_GRAPH_COUNT: usize = 1;
pub const MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT: usize = 0;
pub const DEFAULT_VISUAL_ASSET_LOAD_POLICY: VisualAssetLoadPolicy = VisualAssetLoadPolicy {
    max_admitted_scene_count: VISUAL_ASSET_SLOT_COUNT,
    max_streaming_admitted_scene_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
};
