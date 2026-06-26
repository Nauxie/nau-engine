use crate::authored_assets::VisualAssetDiagnostics;
use crate::camera_runtime::{CameraDiagnostics, CameraFollowFilter};
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::environment_visuals::{WeatherDrift, WindResponsiveVisual};
use crate::island_visuals::IslandStreamDiagnostics;
use crate::power_up_runtime::PowerUpCollectionState;
use crate::{Player, RouteObjectiveTracker};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use nau_engine::animation::AnimationState;
use nau_engine::environment::{LiftField, WindField};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::SkyRoute;

#[derive(SystemParam)]
pub(crate) struct EvalScene<'w, 's> {
    pub(crate) route: Res<'w, SkyRoute>,
    pub(crate) player: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Velocity,
            &'static FlightController,
            &'static AnimationState,
        ),
        With<Player>,
    >,
    pub(crate) camera: Query<'w, 's, &'static Transform, CameraFollowFilter>,
    pub(crate) camera_projection:
        Query<'w, 's, (&'static Camera, &'static GlobalTransform), CameraFollowFilter>,
    pub(crate) camera_diagnostics: Res<'w, CameraDiagnostics>,
    pub(crate) stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    pub(crate) content_diagnostics: Res<'w, IslandContentDiagnostics>,
    pub(crate) asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    pub(crate) route_objectives: Res<'w, RouteObjectiveTracker>,
    pub(crate) power_ups: Res<'w, PowerUpCollectionState>,
    pub(crate) wind_fields: Query<'w, 's, &'static WindField>,
    pub(crate) lift_fields: Query<'w, 's, &'static LiftField>,
    pub(crate) weather_clouds: Query<'w, 's, &'static Transform, With<WeatherDrift>>,
    pub(crate) wind_responsive_visuals:
        Query<'w, 's, (&'static WindResponsiveVisual, &'static Transform)>,
    pub(crate) all_entities: Query<'w, 's, Entity>,
}
