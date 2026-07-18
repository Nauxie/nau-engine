use crate::authored_assets::{
    AuthoredPlayerAnimation, AuthoredPlayerAttachmentMarker, AuthoredPlayerPoseNode,
    VisualAssetDiagnostics,
};
use crate::camera_runtime::{CameraDiagnostics, CameraFollowFilter};
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::environment_visuals::{
    CrosswindGuide, CrosswindRibbon, PlayerAirflowVisual, UpdraftGuide, UpdraftRibbon,
    WeatherDrift, WindResponsiveVisual,
};
use crate::island_visuals::IslandStreamDiagnostics;
use crate::player_runtime::{AuthoredGliderPose, PlayerDisplacementDiagnostics};
use crate::power_up_runtime::PowerUpCollectionState;
use crate::surface_material::SurfaceMaterial;
use crate::world_collision_runtime::WorldCollisionDiagnostics;
use crate::{Player, RouteObjectiveTracker, WindForceDiagnostics};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use nau_engine::animation::{AnimationState, CharacterPart};
use nau_engine::environment::{LiftField, WindField};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::SkyRoute;

pub(crate) type PlayerQueryItem = (
    &'static Transform,
    &'static Velocity,
    &'static FlightController,
    &'static AnimationState,
);
pub(crate) type GeneratedCharacterPartQueryItem = (
    &'static CharacterPart,
    &'static Transform,
    &'static Visibility,
);
pub(crate) type AuthoredPlayerPoseNodeQueryItem = (
    &'static AuthoredPlayerPoseNode,
    &'static Transform,
    &'static GlobalTransform,
    Option<&'static Visibility>,
    Option<&'static InheritedVisibility>,
);
pub(crate) type AuthoredPlayerAttachmentMarkerQueryItem = (
    &'static AuthoredPlayerAttachmentMarker,
    &'static GlobalTransform,
    Option<&'static Visibility>,
    Option<&'static InheritedVisibility>,
);
pub(crate) type AuthoredGliderQueryItem = (
    &'static AuthoredGliderPose,
    &'static Transform,
    Option<&'static Visibility>,
    Option<&'static InheritedVisibility>,
);

#[derive(SystemParam)]
pub(crate) struct EvalScene<'w, 's> {
    pub(crate) route: Res<'w, SkyRoute>,
    pub(crate) player: Query<'w, 's, PlayerQueryItem, With<Player>>,
    pub(crate) camera: Query<'w, 's, &'static Transform, CameraFollowFilter>,
    pub(crate) camera_projection:
        Query<'w, 's, (&'static Camera, &'static GlobalTransform), CameraFollowFilter>,
    pub(crate) camera_diagnostics: Res<'w, CameraDiagnostics>,
    pub(crate) stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    pub(crate) content_diagnostics: Res<'w, IslandContentDiagnostics>,
    pub(crate) asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    pub(crate) meshes: Res<'w, Assets<Mesh>>,
    pub(crate) materials: Res<'w, Assets<StandardMaterial>>,
    pub(crate) surface_materials: Res<'w, Assets<SurfaceMaterial>>,
    pub(crate) route_objectives: Res<'w, RouteObjectiveTracker>,
    pub(crate) power_ups: Res<'w, PowerUpCollectionState>,
    pub(crate) collision_diagnostics: Res<'w, WorldCollisionDiagnostics>,
    pub(crate) player_displacement_diagnostics: Res<'w, PlayerDisplacementDiagnostics>,
    pub(crate) wind_force_diagnostics: Res<'w, WindForceDiagnostics>,
    pub(crate) generated_character_parts: Query<'w, 's, GeneratedCharacterPartQueryItem>,
    pub(crate) authored_player_animations: Query<'w, 's, &'static AuthoredPlayerAnimation>,
    pub(crate) authored_player_pose_nodes: Query<'w, 's, AuthoredPlayerPoseNodeQueryItem>,
    pub(crate) authored_player_attachment_markers:
        Query<'w, 's, AuthoredPlayerAttachmentMarkerQueryItem>,
    pub(crate) authored_gliders: Query<'w, 's, AuthoredGliderQueryItem>,
    pub(crate) wind_fields: Query<'w, 's, &'static WindField>,
    pub(crate) lift_fields: Query<'w, 's, &'static LiftField>,
    pub(crate) weather_clouds: Query<'w, 's, &'static Transform, With<WeatherDrift>>,
    pub(crate) wind_responsive_visuals:
        Query<'w, 's, (&'static WindResponsiveVisual, &'static Transform)>,
    pub(crate) player_wind_shear_visuals: Query<
        'w,
        's,
        (
            &'static PlayerAirflowVisual,
            &'static Transform,
            &'static GlobalTransform,
            &'static Visibility,
        ),
    >,
    pub(crate) updraft_guides: Query<'w, 's, (Entity, &'static UpdraftGuide, &'static Transform)>,
    pub(crate) updraft_ribbons: Query<'w, 's, (Entity, &'static UpdraftRibbon, &'static Transform)>,
    pub(crate) crosswind_guides:
        Query<'w, 's, (Entity, &'static CrosswindGuide, &'static Transform)>,
    pub(crate) crosswind_ribbons:
        Query<'w, 's, (Entity, &'static CrosswindRibbon, &'static Transform)>,
    pub(crate) all_entities: Query<'w, 's, Entity>,
}
