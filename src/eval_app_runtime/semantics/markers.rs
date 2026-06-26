use crate::eval_app_runtime::scene::EvalScene;
use crate::generated_content::island_visual_surface_position;
use bevy::prelude::*;
use nau_engine::environment::AERIAL_POWER_UP_ROUTE;
use nau_engine::world::RouteObjectiveKind;

#[derive(Clone, Copy, Debug)]
pub(super) struct SemanticRouteMarker {
    pub(super) kind: &'static str,
    pub(super) label: &'static str,
    pub(super) world_position: Vec3,
    pub(super) current_objective: bool,
}

pub(super) fn semantic_route_markers(scene: &EvalScene) -> Vec<SemanticRouteMarker> {
    let mut markers = Vec::new();
    let current_label =
        (!scene.route_objectives.complete).then_some(scene.route_objectives.current_label);

    for objective in scene
        .route
        .route_objectives(scene.route_objectives.target_island_name)
    {
        let kind = match objective.kind {
            RouteObjectiveKind::FlyThrough => "objective_updraft",
            RouteObjectiveKind::Land => "objective_landing",
        };
        markers.push(SemanticRouteMarker {
            kind,
            label: objective.label,
            world_position: objective.position,
            current_objective: current_label == Some(objective.label),
        });
    }

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        if island.is_target {
            let ring_size = 8.0;
            for offset in [
                Vec3::new(0.0, 0.05, ring_size * 0.5),
                Vec3::new(0.0, 0.05, -ring_size * 0.5),
                Vec3::new(ring_size * 0.5, 0.05, 0.0),
                Vec3::new(-ring_size * 0.5, 0.05, 0.0),
            ] {
                let surface_y =
                    island.mesh_top_y_at(island.center + Vec3::new(offset.x, 0.0, offset.z));
                markers.push(SemanticRouteMarker {
                    kind: "landing_marker",
                    label: island.name,
                    world_position: Vec3::new(
                        island.center.x + offset.x,
                        surface_y + offset.y,
                        island.center.z + offset.z,
                    ),
                    current_objective: current_label == Some(island.name),
                });
            }
        } else if island.name == "launch mesa" {
            markers.push(SemanticRouteMarker {
                kind: "launch_beacon",
                label: island.name,
                world_position: island_visual_surface_position(island, Vec2::new(-0.42, 0.38))
                    + Vec3::Y * 1.6,
                current_objective: false,
            });
        } else {
            let beacon_height = 3.8 + (island_index % 3) as f32 * 0.7;
            markers.push(SemanticRouteMarker {
                kind: "route_cairn",
                label: island.name,
                world_position: island_visual_surface_position(island, Vec2::new(-0.18, 0.22))
                    + Vec3::Y * (beacon_height * 0.5),
                current_objective: false,
            });
        }
    }

    for power_up in AERIAL_POWER_UP_ROUTE {
        if scene.power_ups.is_collected(power_up) {
            continue;
        }
        markers.push(SemanticRouteMarker {
            kind: "aerial_power_up",
            label: power_up.name,
            world_position: power_up.center,
            current_objective: false,
        });
    }

    markers
}
