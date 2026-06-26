use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::world::{SkyIsland, SkyRoute};

use crate::generated_content::island_visual_surface_position;
use crate::world_collision_runtime::{WorldCollisionProxy, WorldCollisionProxyKind};

use super::types::{AuthoredVisualScene, VisualAssetRegistry};

const AUTHORED_WORLD_FIXTURE_KINDS: &[VisualAssetKind] = &[
    VisualAssetKind::IslandTerrain,
    VisualAssetKind::IslandFoliage,
    VisualAssetKind::IslandRock,
    VisualAssetKind::IslandWater,
    VisualAssetKind::RouteMarker,
    VisualAssetKind::WeatherLayer,
    VisualAssetKind::DistantImpostor,
];

pub(crate) fn authored_world_fixture_scene_handles(
    registry: &VisualAssetRegistry,
) -> Vec<(VisualAssetKind, &'static str, Handle<Scene>)> {
    AUTHORED_WORLD_FIXTURE_KINDS
        .iter()
        .filter_map(|kind| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.kind == *kind)
                .and_then(|slot| {
                    slot.scene_handle
                        .clone()
                        .map(|scene_handle| (*kind, slot.spec.label, scene_handle))
                })
        })
        .collect()
}

pub(crate) fn authored_world_fixture_transform(
    kind: VisualAssetKind,
    route: &SkyRoute,
) -> Transform {
    let Some((island, normalized_offset, surface_offset_y, scale, yaw_radians)) =
        authored_world_fixture_layout(kind, route.islands())
    else {
        return Transform::from_xyz(-140.0, -80.0, 140.0);
    };
    let surface = island_visual_surface_position(island, normalized_offset);

    Transform {
        translation: surface + Vec3::Y * surface_offset_y,
        rotation: Quat::from_rotation_y(yaw_radians),
        scale: Vec3::splat(scale),
    }
}

pub(crate) fn authored_world_fixture_collision_proxy(
    kind: VisualAssetKind,
    transform: &Transform,
) -> Option<WorldCollisionProxy> {
    let (proxy_kind, local_center_y, base_half_extents): (WorldCollisionProxyKind, f32, Vec3) =
        match kind {
            VisualAssetKind::IslandTerrain => (
                WorldCollisionProxyKind::Landmark,
                0.55,
                Vec3::new(2.4, 0.55, 1.8),
            ),
            VisualAssetKind::IslandFoliage => (
                WorldCollisionProxyKind::Tree,
                1.35,
                Vec3::new(0.65, 1.35, 0.65),
            ),
            VisualAssetKind::IslandRock => (
                WorldCollisionProxyKind::Rock,
                0.75,
                Vec3::new(1.1, 0.75, 0.95),
            ),
            VisualAssetKind::RouteMarker => (
                WorldCollisionProxyKind::Landmark,
                0.95,
                Vec3::new(0.55, 0.95, 0.55),
            ),
            VisualAssetKind::IslandWater
            | VisualAssetKind::WeatherLayer
            | VisualAssetKind::DistantImpostor
            | VisualAssetKind::PlayerCharacter
            | VisualAssetKind::Glider => return None,
        };
    let scale = transform
        .scale
        .x
        .abs()
        .max(transform.scale.y.abs())
        .max(transform.scale.z.abs());
    let horizontal_half_extent = base_half_extents.x.max(base_half_extents.z) * scale;
    let half_extents = Vec3::new(
        horizontal_half_extent,
        base_half_extents.y * scale,
        horizontal_half_extent,
    );
    let center = transform.translation + Vec3::Y * local_center_y * scale;

    Some(WorldCollisionProxy::new(center, half_extents, proxy_kind))
}

fn authored_world_fixture_layout(
    kind: VisualAssetKind,
    islands: &[SkyIsland],
) -> Option<(SkyIsland, Vec2, f32, f32, f32)> {
    let (island_index, normalized_offset, surface_offset_y, scale, yaw_radians) = match kind {
        VisualAssetKind::IslandTerrain => (0, Vec2::new(0.34, -0.34), 0.08, 0.82, 0.35),
        VisualAssetKind::IslandFoliage => (0, Vec2::new(-0.42, -0.2), 0.02, 2.2, -0.2),
        VisualAssetKind::IslandRock => (1, Vec2::new(0.3, -0.26), 0.08, 1.8, 0.75),
        VisualAssetKind::IslandWater => (5, Vec2::new(-0.18, 0.24), 0.04, 1.35, -0.45),
        VisualAssetKind::RouteMarker => (3, Vec2::new(-0.08, 0.2), 1.58, 1.4, 0.2),
        VisualAssetKind::WeatherLayer => (4, Vec2::new(0.18, -0.18), 8.2, 4.5, -0.75),
        VisualAssetKind::DistantImpostor => (6, Vec2::new(0.0, 0.0), 4.0, 4.3, 0.55),
        VisualAssetKind::PlayerCharacter | VisualAssetKind::Glider => return None,
    };
    let island = islands
        .get(island_index.min(islands.len().saturating_sub(1)))
        .copied()?;

    Some((
        island,
        normalized_offset,
        surface_offset_y,
        scale,
        yaw_radians,
    ))
}

pub(crate) fn mark_authored_scene_ready(
    scene_ready: On<SceneInstanceReady>,
    authored_scenes: Query<&AuthoredVisualScene>,
    mut registry: ResMut<VisualAssetRegistry>,
) {
    let Ok(scene) = authored_scenes.get(scene_ready.entity) else {
        return;
    };

    registry.mark_scene_ready(scene.kind);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_world_fixture_collision_proxies_cover_solid_fixture_kinds() {
        let route = SkyRoute::default();
        let solid_fixtures = [
            (
                VisualAssetKind::IslandTerrain,
                WorldCollisionProxyKind::Landmark,
            ),
            (
                VisualAssetKind::IslandFoliage,
                WorldCollisionProxyKind::Tree,
            ),
            (VisualAssetKind::IslandRock, WorldCollisionProxyKind::Rock),
            (
                VisualAssetKind::RouteMarker,
                WorldCollisionProxyKind::Landmark,
            ),
        ];

        for (kind, expected_proxy_kind) in solid_fixtures {
            let transform = authored_world_fixture_transform(kind, &route);
            let proxy = authored_world_fixture_collision_proxy(kind, &transform)
                .expect("solid fixture should expose a collision proxy");

            assert_eq!(proxy.kind, expected_proxy_kind);
            assert!(proxy.center.y > transform.translation.y);
            assert!(proxy.half_extents.x >= 0.5);
            assert!(proxy.half_extents.y >= 0.4);
            assert_eq!(proxy.half_extents.x, proxy.half_extents.z);
        }
    }

    #[test]
    fn authored_world_fixture_collision_proxies_skip_non_solid_fixture_kinds() {
        let route = SkyRoute::default();
        for kind in [
            VisualAssetKind::IslandWater,
            VisualAssetKind::WeatherLayer,
            VisualAssetKind::DistantImpostor,
        ] {
            let transform = authored_world_fixture_transform(kind, &route);
            assert!(authored_world_fixture_collision_proxy(kind, &transform).is_none());
        }
    }
}
