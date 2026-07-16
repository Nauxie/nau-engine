use bevy::prelude::{Vec2, Vec3};

use super::{SkyIsland, SkyRoute};

pub const ROUTE_OBSTRUCTION_SPIRES_PER_ISLAND: usize = 1;
const OBSTRUCTION_SPIRE_PROXY_RADIUS_SCALE: f32 = 1.55;
const OBSTRUCTION_SPIRE_PROXY_VERTICAL_PADDING_M: f32 = 0.001;
const OBSTRUCTION_SPIRE_VISIBLE_TIP_RADIUS_SCALE: f32 = 0.24;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteObstructionSpire {
    pub island_index: usize,
    pub island_name: &'static str,
    pub normalized_offset: Vec2,
    pub base_position: Vec3,
    pub center: Vec3,
    pub half_extents: Vec3,
    pub radius_m: f32,
    pub height_m: f32,
    pub seed: u32,
}

pub fn route_obstruction_spires(route: &SkyRoute) -> Vec<RouteObstructionSpire> {
    route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .map(|(index, island)| route_obstruction_spire(index, island))
        .collect()
}

pub fn route_obstruction_spire(island_index: usize, island: SkyIsland) -> RouteObstructionSpire {
    let normalized_offset = obstruction_spire_offset(island_index, island);
    let base_position = island_surface_position(island, normalized_offset);
    let height_m = 4.8 + (island_index % 4) as f32 * 0.75 + island.thickness * 0.035;
    let radius_m = 0.92 + (island_index % 3) as f32 * 0.12;
    let proxy_height_m = height_m + radius_m * OBSTRUCTION_SPIRE_VISIBLE_TIP_RADIUS_SCALE;
    let center = base_position + Vec3::Y * (proxy_height_m * 0.5);
    let half_extents = Vec3::new(
        radius_m * OBSTRUCTION_SPIRE_PROXY_RADIUS_SCALE,
        proxy_height_m * 0.5 + OBSTRUCTION_SPIRE_PROXY_VERTICAL_PADDING_M,
        radius_m * OBSTRUCTION_SPIRE_PROXY_RADIUS_SCALE,
    );

    RouteObstructionSpire {
        island_index,
        island_name: island.name,
        normalized_offset,
        base_position,
        center,
        half_extents,
        radius_m,
        height_m,
        seed: 18_000 + island_index as u32 * 181,
    }
}

fn obstruction_spire_offset(island_index: usize, island: SkyIsland) -> Vec2 {
    if island.name == "launch mesa" {
        return Vec2::new(0.0, (9.5 / island.half_extents.y).clamp(0.18, 0.34));
    }

    let angle = island_index as f32 * 2.399_963 + 0.72;
    let radius = 0.48 + (island_index % 5) as f32 * 0.055;
    Vec2::new(angle.cos(), angle.sin()) * radius
}

fn island_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let normalized_offset = playable_normalized_offset(island, normalized_offset);
    let x = island.center.x + island.half_extents.x * normalized_offset.x;
    let z = island.center.z + island.half_extents.y * normalized_offset.y;

    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
}

fn playable_normalized_offset(island: SkyIsland, normalized_offset: Vec2) -> Vec2 {
    let radius = normalized_offset.length();
    if radius <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let angle = normalized_offset.y.atan2(normalized_offset.x);
    let max_radius = island.playable_silhouette_scale(angle) * 0.82;
    normalized_offset / radius * radius.min(max_radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_obstruction_spires_anchor_one_blocker_to_each_island_surface() {
        let route = SkyRoute::default();
        let spires = route_obstruction_spires(&route);

        assert_eq!(spires.len(), route.islands().len());
        for spire in &spires {
            let island = route
                .islands()
                .iter()
                .copied()
                .find(|island| island.name == spire.island_name)
                .expect("spire should reference a route island");
            assert_eq!(
                spire.base_position.y,
                island.mesh_top_y_at(spire.base_position)
            );
            assert!(island.contains_horizontal(spire.base_position));
            assert!(spire.height_m >= 4.8);
            assert!(spire.radius_m >= 0.9);
            assert_eq!(
                spire.center,
                spire.base_position
                    + Vec3::Y
                        * ((spire.height_m
                            + spire.radius_m * OBSTRUCTION_SPIRE_VISIBLE_TIP_RADIUS_SCALE)
                            * 0.5)
            );
            assert!(
                spire.half_extents.x / spire.radius_m >= 1.485,
                "proxy must contain the visible rib reach and width"
            );
            assert!(
                spire.center.y + spire.half_extents.y
                    >= spire.base_position.y
                        + spire.height_m
                        + spire.radius_m * OBSTRUCTION_SPIRE_VISIBLE_TIP_RADIUS_SCALE
            );
        }
    }

    #[test]
    fn launch_mesa_spire_sits_in_the_camera_obstruction_lane() {
        let route = SkyRoute::default();
        let launch = route_obstruction_spire(0, route.islands()[0]);

        assert_eq!(launch.island_name, "launch mesa");
        assert!(launch.center.z > 8.0);
        assert!(launch.center.z < 11.0);
        assert!(launch.center.y > route.islands()[0].floor_y());
    }
}
