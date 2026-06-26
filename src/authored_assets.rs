mod animation;
mod diagnostics;
mod fixtures;
mod registry;
mod types;

#[cfg(test)]
pub(crate) use animation::{
    AuthoredPlayerClip, authored_player_clip_for_state, resolve_named_animation_clip_handles,
};
pub(crate) use animation::{link_ready_authored_animations, update_authored_player_animation};
pub(crate) use diagnostics::update_visual_asset_diagnostics;
pub(crate) use fixtures::{
    authored_world_fixture_scene_handles, authored_world_fixture_transform,
    mark_authored_scene_ready,
};
pub(crate) use registry::prepare_visual_asset_registry;
pub(crate) use types::{
    AuthoredVisualScene, AuthoredVisualSceneRole, GeneratedPlayerPlaceholder,
    VisibleAuthoredWorldFixture, VisualAssetDiagnostics, VisualAssetRegistry,
};
