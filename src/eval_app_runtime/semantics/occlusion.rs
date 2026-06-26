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
    let dx = (point.x - island.center.x) / island.half_extents.x.max(0.001);
    let dz = (point.z - island.center.z) / island.half_extents.y.max(0.001);
    if dx * dx + dz * dz > 1.10 {
        return false;
    }

    let top_y = island.mesh_top_y_at(point) + 0.9;
    let bottom_y = island.center.y - island.thickness * 1.15;
    point.y >= bottom_y && point.y <= top_y
}
