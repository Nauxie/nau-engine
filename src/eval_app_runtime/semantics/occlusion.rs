use bevy::prelude::*;
use nau_engine::world::SkyIsland;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SemanticMarkerOcclusion {
    pub(crate) island_name: &'static str,
    pub(crate) distance_m: f32,
}

pub(crate) fn marker_occlusion_between(
    camera_position: Vec3,
    marker_position: Vec3,
    islands: &[SkyIsland],
) -> Option<SemanticMarkerOcclusion> {
    let mut nearest = None;
    for island in islands {
        let Some(distance_m) =
            island_segment_occlusion_distance(camera_position, marker_position, *island)
        else {
            continue;
        };
        if nearest
            .as_ref()
            .is_none_or(|occlusion: &SemanticMarkerOcclusion| distance_m < occlusion.distance_m)
        {
            nearest = Some(SemanticMarkerOcclusion {
                island_name: island.name,
                distance_m,
            });
        }
    }
    nearest
}

fn island_segment_occlusion_distance(
    camera_position: Vec3,
    marker_position: Vec3,
    island: SkyIsland,
) -> Option<f32> {
    let segment = marker_position - camera_position;
    let length = segment.length();
    if length <= 0.01 {
        return None;
    }
    let direction = segment / length;
    let max_distance = length - 2.0;
    if max_distance <= 1.0 {
        return None;
    }
    let steps = ((length / 6.0).ceil() as usize).clamp(12, 96);

    for step in 1..steps {
        let distance_m = length * step as f32 / steps as f32;
        if distance_m >= max_distance {
            break;
        }
        let point = camera_position + direction * distance_m;
        if island_blocks_marker_ray(island, point) {
            return Some(distance_m);
        }
    }

    None
}

fn island_blocks_marker_ray(island: SkyIsland, point: Vec3) -> bool {
    if island.signed_visual_edge_distance(point) > 0.0 {
        return false;
    }

    let top_y = island.mesh_top_y_at(point) + 0.9;
    let bottom_y = island.center.y - island.thickness * 1.15;
    point.y >= bottom_y && point.y <= top_y
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generated_content::island_hero_landmark_spec;
    use nau_engine::world::SkyRoute;

    #[test]
    fn at_least_one_generated_copper_arcade_probe_is_unobstructed() {
        let route = SkyRoute::default();
        let camera = Vec3::new(42.012_337, 78.273_575, -294.080_38);
        let (island_index, island) = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .find(|(_, island)| island.name == "copper stair")
            .expect("copper stair should exist");
        let hero = island_hero_landmark_spec(island_index, island)
            .expect("copper stair should have a hero");
        let probes = hero.semantic_sample_positions().map(|sample| {
            (
                sample,
                marker_occlusion_between(camera, sample, route.islands()),
            )
        });

        assert!(
            probes.iter().any(|(_, occlusion)| occlusion.is_none()),
            "the visibly exposed Copper Arcade must retain an unobstructed mesh-derived probe; got {probes:?}"
        );
    }

    #[test]
    fn authored_visual_footprint_rejects_points_only_inside_the_coarse_ellipse() {
        let route = SkyRoute::default();
        let island = route
            .island_named("distant crown")
            .expect("distant crown should exist");
        let (angle, silhouette_scale) = (0..720)
            .map(|step| {
                let angle = step as f32 / 720.0 * std::f32::consts::TAU;
                (angle, island.visual_silhouette_scale(angle))
            })
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .expect("silhouette search should produce a sample");
        assert!(silhouette_scale < 1.0);
        let coarse_ellipse_radius = (silhouette_scale + 1.0) * 0.5;
        let point = Vec3::new(
            island.center.x + angle.cos() * island.half_extents.x * coarse_ellipse_radius,
            island.center.y,
            island.center.z + angle.sin() * island.half_extents.y * coarse_ellipse_radius,
        );

        assert!(coarse_ellipse_radius < 1.0);
        assert!(island.signed_visual_edge_distance(point) > 0.0);
        assert!(!island_blocks_marker_ray(island, point));
    }
}
