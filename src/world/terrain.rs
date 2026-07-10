use bevy::prelude::Vec3;

use super::{PLAYER_STANDING_OFFSET, TERRAIN_VISUAL_FOOTING_OFFSET_M};

pub const WORLD_TERRAIN_TILE_SIZE_M: f32 = 720.0;
pub const WORLD_TERRAIN_GRID_SUBDIVISIONS: usize = 36;
pub const WORLD_TERRAIN_GRID_SPACING_M: f32 =
    WORLD_TERRAIN_TILE_SIZE_M / WORLD_TERRAIN_GRID_SUBDIVISIONS as f32;

pub fn world_terrain_floor_y_at(position: Vec3) -> f32 {
    let grid_x = position.x / WORLD_TERRAIN_GRID_SPACING_M;
    let grid_z = position.z / WORLD_TERRAIN_GRID_SPACING_M;
    let cell_x = grid_x.floor() as i32;
    let cell_z = grid_z.floor() as i32;
    let x = grid_x - cell_x as f32;
    let z = grid_z - cell_z as f32;

    let h00 = world_terrain_node_floor_y(cell_x, cell_z);
    let h10 = world_terrain_node_floor_y(cell_x + 1, cell_z);
    let h01 = world_terrain_node_floor_y(cell_x, cell_z + 1);
    let h11 = world_terrain_node_floor_y(cell_x + 1, cell_z + 1);

    if x + z <= 1.0 {
        h00 + (h10 - h00) * x + (h01 - h00) * z
    } else {
        h10 * (1.0 - z) + h01 * (1.0 - x) + h11 * (x + z - 1.0)
    }
}

pub fn world_terrain_visual_y_at(position: Vec3) -> f32 {
    world_terrain_floor_y_at(position) - TERRAIN_VISUAL_FOOTING_OFFSET_M
}

fn world_terrain_node_floor_y(grid_x: i32, grid_z: i32) -> f32 {
    let x = grid_x as f32 * WORLD_TERRAIN_GRID_SPACING_M;
    let z = grid_z as f32 * WORLD_TERRAIN_GRID_SPACING_M;
    let broad = (x * 0.0031 + z * 0.0017).sin() * 4.2 + (x * -0.0019 + z * 0.0037).cos() * 3.4;
    let rolling = (x * 0.0091).sin() * (z * 0.0073).cos() * 1.8;
    let ridge = (1.0 - (x * 0.0048 + z * 0.0062).sin().abs()).powf(2.6) * 6.5;
    let highland = (1.0 - (x * -0.0027 + z * 0.0041).cos().abs()).powf(3.0) * 4.8;
    let river = river_channel_factor(x, z);
    let river_cut = (1.0 - river).powf(2.0) * 2.8;

    PLAYER_STANDING_OFFSET + broad + rolling + ridge + highland - river_cut
}

fn river_channel_factor(x: f32, z: f32) -> f32 {
    let braided = (x * 0.006 + (z * 0.0035).sin() * 1.7 + z * 0.0011)
        .sin()
        .abs();
    (braided / 0.24).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_floor_interpolates_the_render_triangle_lattice() {
        let origin = Vec3::new(0.0, 100.0, 0.0);
        let east = Vec3::new(WORLD_TERRAIN_GRID_SPACING_M, -100.0, 0.0);
        let south = Vec3::new(0.0, 0.0, WORLD_TERRAIN_GRID_SPACING_M);
        let midpoint = Vec3::new(
            WORLD_TERRAIN_GRID_SPACING_M * 0.25,
            0.0,
            WORLD_TERRAIN_GRID_SPACING_M * 0.25,
        );
        let expected = world_terrain_floor_y_at(origin) * 0.5
            + world_terrain_floor_y_at(east) * 0.25
            + world_terrain_floor_y_at(south) * 0.25;

        assert!((world_terrain_floor_y_at(midpoint) - expected).abs() < 0.0001);
    }

    #[test]
    fn terrain_floor_is_continuous_across_tile_boundaries() {
        let boundary = WORLD_TERRAIN_TILE_SIZE_M * 0.5;
        for z in [-311.0, -20.0, 0.0, 187.0, 354.0] {
            let left = world_terrain_floor_y_at(Vec3::new(boundary - 0.001, 0.0, z));
            let right = world_terrain_floor_y_at(Vec3::new(boundary + 0.001, 0.0, z));
            assert!(
                (left - right).abs() < 0.01,
                "terrain seam at z={z} differs by {} m",
                (left - right).abs()
            );
        }
    }

    #[test]
    fn terrain_relief_stays_near_the_original_playable_ground() {
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for z in -180..=180 {
            for x in -180..=180 {
                let position = Vec3::new(x as f32 * 20.0, 0.0, z as f32 * 20.0);
                let y = world_terrain_floor_y_at(position);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }

        assert!(min_y > -12.0, "terrain dropped too far below play: {min_y}");
        assert!(max_y < 24.0, "terrain rose too close to islands: {max_y}");
        assert!(
            max_y - min_y >= 14.0,
            "terrain needs readable relief: {}",
            max_y - min_y
        );
    }
}
