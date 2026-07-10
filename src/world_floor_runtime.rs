use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::player_runtime::Player;
use nau_engine::world::{
    WORLD_TERRAIN_GRID_SUBDIVISIONS, WORLD_TERRAIN_TILE_SIZE_M, world_terrain_visual_y_at,
};

pub(crate) const WORLD_FLOOR_TILE_SIZE_M: f32 = WORLD_TERRAIN_TILE_SIZE_M;
pub(crate) const WORLD_FLOOR_ACTIVE_RADIUS_TILES: i32 = 1;
pub(crate) const WORLD_FLOOR_GRID_SUBDIVISIONS: usize = WORLD_TERRAIN_GRID_SUBDIVISIONS;
const WORLD_FLOOR_MAX_TILE_SPAWNS_PER_FRAME: usize = 1;
const WORLD_FLOOR_MAX_TILE_DESPAWNS_PER_FRAME: usize = 1;
const WORLD_FLOOR_MAX_RESIDENT_TILES: usize = 25;
const WORLD_FLOOR_FEATURE_RIVER: u8 = 1 << 4;
const WORLD_FLOOR_FEATURE_HIGH_PEAK: u8 = 1 << 5;
const WORLD_FLOOR_GROUND_COVER_PATCHES: usize = 96;
const WORLD_FLOOR_GROUND_COVER_BLADES_PER_PATCH: usize = 4;

#[derive(Clone, Debug, Resource)]
pub(crate) struct WorldFloorMaterials {
    pub(crate) ocean: Handle<StandardMaterial>,
    pub(crate) lowland: Handle<StandardMaterial>,
    pub(crate) ridge: Handle<StandardMaterial>,
    pub(crate) mountain: Handle<StandardMaterial>,
    pub(crate) ground_cover: Handle<StandardMaterial>,
}

#[derive(Clone, Copy, Debug, Default, Resource)]
pub(crate) struct WorldFloorDiagnostics {
    pub(crate) visible_tile_count: usize,
    pub(crate) max_visible_tile_count: usize,
    pub(crate) resident_tile_count: usize,
    pub(crate) max_resident_tile_count: usize,
    pub(crate) initial_spawned_tile_count: usize,
    pub(crate) spawned_tiles_this_frame: usize,
    pub(crate) despawned_tiles_this_frame: usize,
    pub(crate) max_spawned_tiles_per_frame: usize,
    pub(crate) max_despawned_tiles_per_frame: usize,
    pub(crate) total_spawned_tiles: usize,
    pub(crate) total_despawned_tiles: usize,
    pub(crate) mesh_vertex_count: usize,
    pub(crate) mesh_triangle_count: usize,
    pub(crate) material_count: usize,
    pub(crate) biome_count: usize,
    pub(crate) terrain_feature_count: usize,
    pub(crate) color_band_count: usize,
    pub(crate) river_vertex_count: usize,
    pub(crate) min_height_y: f32,
    pub(crate) max_height_y: f32,
    pub(crate) relief_range_m: f32,
    pub(crate) active_radius_tiles: i32,
    pub(crate) tile_size_m: f32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct WorldFloorTileCoord {
    x: i32,
    z: i32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum WorldFloorBiome {
    Wetland,
    Lowland,
    Ridge,
    Mountain,
}

#[derive(Debug, Default, Resource)]
pub(crate) struct WorldFloorState {
    active_tiles: HashSet<WorldFloorTileCoord>,
    tiles: HashMap<WorldFloorTileCoord, WorldFloorTileInstance>,
}

#[derive(Debug)]
struct WorldFloorTileInstance {
    entities: [Entity; 2],
    meshes: [Handle<Mesh>; 2],
    stats: WorldFloorTileStats,
}

#[derive(Clone, Copy, Debug)]
struct WorldFloorTileStats {
    biome: WorldFloorBiome,
    biome_mask: u8,
    vertex_count: usize,
    triangle_count: usize,
    feature_mask: u8,
    color_band_count: usize,
    river_vertex_count: usize,
    min_height_y: f32,
    max_height_y: f32,
}

#[derive(Component)]
struct WorldFloorTile;

pub(crate) fn spawn_world_floor(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &WorldFloorMaterials,
    player_start: Vec3,
) {
    let mut state = WorldFloorState::default();
    let mut diagnostics = WorldFloorDiagnostics {
        active_radius_tiles: WORLD_FLOOR_ACTIVE_RADIUS_TILES,
        tile_size_m: WORLD_FLOOR_TILE_SIZE_M,
        ..default()
    };
    let desired_coords = desired_tile_coords(player_start);
    for coord in desired_coords {
        let instance = spawn_tile(commands, meshes, materials, coord);
        state.active_tiles.insert(coord);
        state.tiles.insert(coord, instance);
        diagnostics.initial_spawned_tile_count += 1;
        diagnostics.spawned_tiles_this_frame += 1;
        diagnostics.total_spawned_tiles += 1;
    }
    refresh_diagnostics(&state, &mut diagnostics);

    commands.insert_resource(materials.clone());
    commands.insert_resource(state);
    commands.insert_resource(diagnostics);
}

pub(crate) fn update_world_floor_streaming(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<WorldFloorMaterials>,
    mut state: ResMut<WorldFloorState>,
    mut diagnostics: ResMut<WorldFloorDiagnostics>,
    player: Query<&Transform, With<Player>>,
) {
    diagnostics.spawned_tiles_this_frame = 0;
    diagnostics.despawned_tiles_this_frame = 0;

    let player_position = player
        .single()
        .ok()
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::ZERO);
    let desired_coords = desired_tile_coords(player_position);
    let desired_set = desired_coords.iter().copied().collect::<HashSet<_>>();
    let center = tile_coord_for_position(player_position);

    let stale_coords = state
        .active_tiles
        .iter()
        .copied()
        .filter(|coord| !desired_set.contains(coord))
        .collect::<Vec<_>>();

    for coord in stale_coords
        .into_iter()
        .take(WORLD_FLOOR_MAX_TILE_DESPAWNS_PER_FRAME)
    {
        if state.active_tiles.remove(&coord) {
            if let Some(instance) = state.tiles.get(&coord) {
                set_tile_visibility(&mut commands, instance, false);
            }
            diagnostics.despawned_tiles_this_frame += 1;
            diagnostics.total_despawned_tiles += 1;
        }
    }

    if diagnostics.spawned_tiles_this_frame < WORLD_FLOOR_MAX_TILE_SPAWNS_PER_FRAME {
        let missing_coord = desired_coords
            .into_iter()
            .find(|coord| !state.active_tiles.contains(coord));
        if let Some(coord) = missing_coord {
            if let Some(instance) = state.tiles.get(&coord) {
                set_tile_visibility(&mut commands, instance, true);
            } else {
                let instance = spawn_tile(&mut commands, &mut meshes, &materials, coord);
                state.tiles.insert(coord, instance);
            }
            state.active_tiles.insert(coord);
            diagnostics.spawned_tiles_this_frame += 1;
            diagnostics.total_spawned_tiles += 1;
        }
    }

    evict_distant_pooled_tiles(&mut commands, &mut meshes, &mut state, center);
    diagnostics.max_spawned_tiles_per_frame = diagnostics
        .max_spawned_tiles_per_frame
        .max(diagnostics.spawned_tiles_this_frame);
    diagnostics.max_despawned_tiles_per_frame = diagnostics
        .max_despawned_tiles_per_frame
        .max(diagnostics.despawned_tiles_this_frame);
    refresh_diagnostics(&state, &mut diagnostics);
}

fn spawn_tile(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &WorldFloorMaterials,
    coord: WorldFloorTileCoord,
) -> WorldFloorTileInstance {
    let (terrain_mesh, mut stats) = world_floor_tile_mesh(coord);
    let (ground_cover_mesh, ground_cover_vertices, ground_cover_triangles) =
        world_floor_ground_cover_mesh(coord);
    stats.vertex_count += ground_cover_vertices;
    stats.triangle_count += ground_cover_triangles;
    let terrain_mesh = meshes.add(terrain_mesh);
    let ground_cover_mesh = meshes.add(ground_cover_mesh);
    let material = match stats.biome {
        WorldFloorBiome::Wetland => materials.ocean.clone(),
        WorldFloorBiome::Lowland => materials.lowland.clone(),
        WorldFloorBiome::Ridge => materials.ridge.clone(),
        WorldFloorBiome::Mountain => materials.mountain.clone(),
    };
    let terrain_entity = commands
        .spawn((
            Mesh3d(terrain_mesh.clone()),
            MeshMaterial3d(material),
            Transform::default(),
            NotShadowCaster,
            WorldFloorTile,
            Name::new(format!("world floor tile {},{}", coord.x, coord.z)),
        ))
        .id();
    let ground_cover_entity = commands
        .spawn((
            Mesh3d(ground_cover_mesh.clone()),
            MeshMaterial3d(materials.ground_cover.clone()),
            Transform::default(),
            NotShadowCaster,
            Name::new(format!("world floor ground cover {},{}", coord.x, coord.z)),
        ))
        .id();

    WorldFloorTileInstance {
        entities: [terrain_entity, ground_cover_entity],
        meshes: [terrain_mesh, ground_cover_mesh],
        stats,
    }
}

fn set_tile_visibility(commands: &mut Commands, instance: &WorldFloorTileInstance, visible: bool) {
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for entity in instance.entities {
        commands.entity(entity).insert(visibility);
    }
}

fn evict_distant_pooled_tiles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    state: &mut WorldFloorState,
    center: WorldFloorTileCoord,
) {
    while state.tiles.len() > WORLD_FLOOR_MAX_RESIDENT_TILES {
        let resident_coords = state.tiles.keys().copied().collect::<Vec<_>>();
        let Some(coord) = farthest_inactive_tile(&resident_coords, &state.active_tiles, center)
        else {
            break;
        };
        let Some(instance) = state.tiles.remove(&coord) else {
            continue;
        };
        for entity in instance.entities {
            commands.entity(entity).despawn();
        }
        for mesh in instance.meshes {
            meshes.remove(mesh.id());
        }
    }
}

fn farthest_inactive_tile(
    resident_coords: &[WorldFloorTileCoord],
    active_tiles: &HashSet<WorldFloorTileCoord>,
    center: WorldFloorTileCoord,
) -> Option<WorldFloorTileCoord> {
    resident_coords
        .iter()
        .filter(|coord| !active_tiles.contains(coord))
        .max_by_key(|coord| {
            let dx = coord.x - center.x;
            let dz = coord.z - center.z;
            (dx * dx + dz * dz, coord.z, coord.x)
        })
        .copied()
}

fn world_floor_ground_cover_mesh(coord: WorldFloorTileCoord) -> (Mesh, usize, usize) {
    let max_blade_count =
        WORLD_FLOOR_GROUND_COVER_PATCHES * WORLD_FLOOR_GROUND_COVER_BLADES_PER_PATCH;
    let mut positions = Vec::with_capacity(max_blade_count * 8);
    let mut normals = Vec::with_capacity(max_blade_count * 8);
    let mut uvs = Vec::with_capacity(max_blade_count * 8);
    let mut indices = Vec::with_capacity(max_blade_count * 12);
    let half_size = WORLD_FLOOR_TILE_SIZE_M * 0.5;
    let center_x = coord.x as f32 * WORLD_FLOOR_TILE_SIZE_M;
    let center_z = coord.z as f32 * WORLD_FLOOR_TILE_SIZE_M;
    let seed_x = coord.x.wrapping_mul(197);
    let seed_z = coord.z.wrapping_mul(263);

    for patch in 0..WORLD_FLOOR_GROUND_COVER_PATCHES {
        let patch = patch as u32;
        let x = center_x - half_size
            + 12.0
            + hash01(seed_x, seed_z, patch.wrapping_mul(17) + 3) * (WORLD_FLOOR_TILE_SIZE_M - 24.0);
        let z = center_z - half_size
            + 12.0
            + hash01(seed_x, seed_z, patch.wrapping_mul(23) + 5) * (WORLD_FLOOR_TILE_SIZE_M - 24.0);
        if world_floor_biome_at(x, z) == WorldFloorBiome::Wetland {
            continue;
        }
        let patch_angle =
            hash01(seed_x, seed_z, patch.wrapping_mul(31) + 7) * std::f32::consts::TAU;

        for blade in 0..WORLD_FLOOR_GROUND_COVER_BLADES_PER_PATCH {
            let blade = blade as u32;
            let angle = patch_angle
                + blade as f32 * std::f32::consts::TAU
                    / WORLD_FLOOR_GROUND_COVER_BLADES_PER_PATCH as f32;
            let radius = 0.8 + hash01(seed_x, seed_z, patch * 41 + blade + 11) * 2.8;
            let blade_x = x + angle.cos() * radius;
            let blade_z = z + angle.sin() * radius;
            let origin = Vec3::new(
                blade_x,
                world_terrain_visual_y_at(Vec3::new(blade_x, 0.0, blade_z)) + 0.04,
                blade_z,
            );
            let width = 0.16 + hash01(seed_x, seed_z, patch * 43 + blade + 13) * 0.18;
            let height = 0.55 + hash01(seed_x, seed_z, patch * 47 + blade + 17) * 0.75;
            push_crossed_ground_blade(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                origin,
                angle,
                width,
                height,
            );
        }
    }

    let vertex_count = positions.len();
    let triangle_count = indices.len() / 3;
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    (mesh, vertex_count, triangle_count)
}

#[allow(clippy::too_many_arguments)]
fn push_crossed_ground_blade(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    origin: Vec3,
    angle: f32,
    width: f32,
    height: f32,
) {
    for plane_angle in [angle, angle + std::f32::consts::FRAC_PI_2] {
        let right = Vec3::new(plane_angle.cos(), 0.0, plane_angle.sin()) * (width * 0.5);
        let lean = Vec3::new(angle.cos(), 0.0, angle.sin()) * (height * 0.16);
        let top = origin + Vec3::Y * height + lean;
        let normal = Vec3::new(right.z, width * 0.35, -right.x).normalize();
        let start = positions.len() as u32;
        positions.extend([
            (origin - right).to_array(),
            (origin + right).to_array(),
            (top - right * 0.22).to_array(),
            (top + right * 0.22).to_array(),
        ]);
        normals.extend([normal.to_array(); 4]);
        uvs.extend([[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]]);
        indices.extend([start, start + 2, start + 1, start + 1, start + 2, start + 3]);
    }
}

fn desired_tile_coords(player_position: Vec3) -> Vec<WorldFloorTileCoord> {
    let center = tile_coord_for_position(player_position);
    let diameter = WORLD_FLOOR_ACTIVE_RADIUS_TILES * 2 + 1;
    let mut coords = Vec::with_capacity((diameter * diameter) as usize);
    for z in
        (center.z - WORLD_FLOOR_ACTIVE_RADIUS_TILES)..=(center.z + WORLD_FLOOR_ACTIVE_RADIUS_TILES)
    {
        for x in (center.x - WORLD_FLOOR_ACTIVE_RADIUS_TILES)
            ..=(center.x + WORLD_FLOOR_ACTIVE_RADIUS_TILES)
        {
            coords.push(WorldFloorTileCoord { x, z });
        }
    }
    coords.sort_by_key(|coord| {
        let dx = coord.x - center.x;
        let dz = coord.z - center.z;
        (dx * dx + dz * dz, coord.z, coord.x)
    });
    coords
}

fn tile_coord_for_position(position: Vec3) -> WorldFloorTileCoord {
    WorldFloorTileCoord {
        x: (position.x / WORLD_FLOOR_TILE_SIZE_M).round() as i32,
        z: (position.z / WORLD_FLOOR_TILE_SIZE_M).round() as i32,
    }
}

fn refresh_diagnostics(state: &WorldFloorState, diagnostics: &mut WorldFloorDiagnostics) {
    diagnostics.visible_tile_count = state.active_tiles.len();
    diagnostics.max_visible_tile_count = diagnostics
        .max_visible_tile_count
        .max(diagnostics.visible_tile_count);
    diagnostics.resident_tile_count = state.tiles.len();
    diagnostics.max_resident_tile_count = diagnostics
        .max_resident_tile_count
        .max(diagnostics.resident_tile_count);
    diagnostics.mesh_vertex_count = state
        .active_tiles
        .iter()
        .filter_map(|coord| state.tiles.get(coord))
        .map(|instance| instance.stats.vertex_count)
        .sum();
    diagnostics.mesh_triangle_count = state
        .active_tiles
        .iter()
        .filter_map(|coord| state.tiles.get(coord))
        .map(|instance| instance.stats.triangle_count)
        .sum();

    let mut biome_mask = 0u8;
    let mut feature_mask = 0u8;
    let mut color_band_count = 0usize;
    let mut river_vertex_count = 0usize;
    let mut min_height_y = f32::INFINITY;
    let mut max_height_y = f32::NEG_INFINITY;
    for coord in &state.active_tiles {
        let Some(instance) = state.tiles.get(coord) else {
            continue;
        };
        biome_mask |= instance.stats.biome_mask;
        feature_mask |= instance.stats.feature_mask;
        color_band_count += instance.stats.color_band_count;
        river_vertex_count += instance.stats.river_vertex_count;
        min_height_y = min_height_y.min(instance.stats.min_height_y);
        max_height_y = max_height_y.max(instance.stats.max_height_y);
    }

    diagnostics.material_count = if state.active_tiles.is_empty() { 0 } else { 2 };
    diagnostics.biome_count = biome_mask.count_ones() as usize;
    diagnostics.terrain_feature_count = feature_mask.count_ones() as usize;
    diagnostics.color_band_count = color_band_count;
    diagnostics.river_vertex_count = river_vertex_count;
    diagnostics.min_height_y = if min_height_y.is_finite() {
        min_height_y
    } else {
        0.0
    };
    diagnostics.max_height_y = if max_height_y.is_finite() {
        max_height_y
    } else {
        0.0
    };
    diagnostics.relief_range_m = if min_height_y.is_finite() && max_height_y.is_finite() {
        max_height_y - min_height_y
    } else {
        0.0
    };
}

fn biome_index(biome: WorldFloorBiome) -> u8 {
    match biome {
        WorldFloorBiome::Wetland => 0,
        WorldFloorBiome::Lowland => 1,
        WorldFloorBiome::Ridge => 2,
        WorldFloorBiome::Mountain => 3,
    }
}

fn world_floor_tile_mesh(coord: WorldFloorTileCoord) -> (Mesh, WorldFloorTileStats) {
    let dominant_biome = tile_biome(coord);
    let stride = WORLD_FLOOR_GRID_SUBDIVISIONS + 1;
    let vertex_count = stride * stride;
    let triangle_count = WORLD_FLOOR_GRID_SUBDIVISIONS * WORLD_FLOOR_GRID_SUBDIVISIONS * 2;
    let half_size = WORLD_FLOOR_TILE_SIZE_M * 0.5;
    let center_x = coord.x as f32 * WORLD_FLOOR_TILE_SIZE_M;
    let center_z = coord.z as f32 * WORLD_FLOOR_TILE_SIZE_M;

    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(triangle_count * 3);
    let mut min_height_y = f32::INFINITY;
    let mut max_height_y = f32::NEG_INFINITY;
    let mut biome_mask = 0u8;
    let mut feature_mask = 0u8;
    let mut color_bands = HashSet::new();
    let mut river_vertex_count = 0usize;

    for row in 0..=WORLD_FLOOR_GRID_SUBDIVISIONS {
        let z_t = row as f32 / WORLD_FLOOR_GRID_SUBDIVISIONS as f32;
        let z = center_z - half_size + z_t * WORLD_FLOOR_TILE_SIZE_M;
        for col in 0..=WORLD_FLOOR_GRID_SUBDIVISIONS {
            let x_t = col as f32 / WORLD_FLOOR_GRID_SUBDIVISIONS as f32;
            let x = center_x - half_size + x_t * WORLD_FLOOR_TILE_SIZE_M;
            let biome = world_floor_biome_at(x, z);
            let river = river_channel_factor(x, z);
            let y = world_terrain_visual_y_at(Vec3::new(x, 0.0, z));
            let color = world_floor_vertex_color(biome, y, river, x, z);

            biome_mask |= 1 << biome_index(biome);
            feature_mask |= world_floor_feature_mask(biome, river, y, x, z);
            if biome != WorldFloorBiome::Wetland && river < 0.42 {
                river_vertex_count += 1;
            }
            color_bands.insert(quantized_color_band(color));
            min_height_y = min_height_y.min(y);
            max_height_y = max_height_y.max(y);
            positions.push([x, y, z]);
            uvs.push([x / 96.0, z / 96.0]);
            colors.push(color);
        }
    }

    for row in 0..WORLD_FLOOR_GRID_SUBDIVISIONS {
        for col in 0..WORLD_FLOOR_GRID_SUBDIVISIONS {
            let a = (row * stride + col) as u32;
            let b = a + 1;
            let c = ((row + 1) * stride + col) as u32;
            let d = c + 1;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let normals = smooth_normals_from_triangles(&positions, &indices);
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    (
        mesh,
        WorldFloorTileStats {
            biome: dominant_biome,
            biome_mask,
            vertex_count,
            triangle_count,
            feature_mask,
            color_band_count: color_bands.len(),
            river_vertex_count,
            min_height_y,
            max_height_y,
        },
    )
}

fn tile_biome(coord: WorldFloorTileCoord) -> WorldFloorBiome {
    world_floor_biome_at(
        coord.x as f32 * WORLD_FLOOR_TILE_SIZE_M,
        coord.z as f32 * WORLD_FLOOR_TILE_SIZE_M,
    )
}

fn world_floor_biome_at(x: f32, z: f32) -> WorldFloorBiome {
    let x = x / 300.0;
    let z = z / 300.0;
    let continental = (x * 0.67 + z * 0.29).sin() * 0.48
        + (x * -0.21 + z * 0.73).cos() * 0.34
        + (hash01(x.floor() as i32, z.floor() as i32, 11) - 0.5) * 0.42;
    let ridge_signal =
        ((x * 0.91 - z * 0.37).sin() * 0.55 + (x * 0.23 + z * 1.17).cos() * 0.45).abs();

    if continental < -0.34 {
        WorldFloorBiome::Wetland
    } else if continental > 0.34 && ridge_signal > 0.58 {
        WorldFloorBiome::Mountain
    } else if ridge_signal > 0.48 {
        WorldFloorBiome::Ridge
    } else {
        WorldFloorBiome::Lowland
    }
}

fn peak_cluster_factor(x: f32, z: f32) -> f32 {
    (1.0 - ((x * -0.007 + z * 0.013).cos()).abs()).powf(3.2)
}

fn river_channel_factor(x: f32, z: f32) -> f32 {
    let braided = (x * 0.006 + (z * 0.0035).sin() * 1.7 + z * 0.0011)
        .sin()
        .abs();
    (braided / 0.24).clamp(0.0, 1.0)
}

fn world_floor_feature_mask(biome: WorldFloorBiome, river: f32, y: f32, x: f32, z: f32) -> u8 {
    let mut mask = 1 << biome_index(biome);
    if biome != WorldFloorBiome::Wetland && river < 0.42 {
        mask |= WORLD_FLOOR_FEATURE_RIVER;
    }
    if y > 8.0 && peak_cluster_factor(x, z) > 0.20 {
        mask |= WORLD_FLOOR_FEATURE_HIGH_PEAK;
    }
    mask
}

fn world_floor_vertex_color(
    biome: WorldFloorBiome,
    y: f32,
    river: f32,
    x: f32,
    z: f32,
) -> [f32; 4] {
    let relief = ((y + 10.0) / 34.0).clamp(0.0, 1.0);
    let river_tint = (1.0 - river).powf(2.0);
    let terrain_grain = terrain_grain_factor(x, z);
    let warm_band = ((x * 0.0042 - z * 0.0031).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let (mut r, mut g, mut b) = match biome {
        WorldFloorBiome::Wetland => (
            0.25 + terrain_grain * 0.05,
            0.42 + warm_band * 0.08,
            0.31 + relief * 0.04,
        ),
        WorldFloorBiome::Lowland => (
            0.54 + warm_band * 0.12 + relief * 0.08,
            0.66 + terrain_grain * 0.10,
            0.34 + warm_band * 0.08 - relief * 0.04,
        ),
        WorldFloorBiome::Ridge => (
            0.50 + relief * 0.16 + terrain_grain * 0.05,
            0.50 + relief * 0.07,
            0.42 + warm_band * 0.08,
        ),
        WorldFloorBiome::Mountain => (
            0.54 + relief * 0.24,
            0.57 + relief * 0.20 + terrain_grain * 0.04,
            0.58 + relief * 0.18,
        ),
    };

    if biome != WorldFloorBiome::Wetland {
        r = r * (1.0 - river_tint * 0.35) + 0.20 * river_tint;
        g = g * (1.0 - river_tint * 0.20) + 0.38 * river_tint;
        b = b * (1.0 - river_tint * 0.08) + 0.62 * river_tint;
    }

    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0]
}

fn terrain_grain_factor(x: f32, z: f32) -> f32 {
    ((x * 0.031).sin() * 0.45 + (z * 0.027).cos() * 0.35 + (x * 0.014 + z * 0.021).sin() * 0.20)
        * 0.5
        + 0.5
}

fn quantized_color_band(color: [f32; 4]) -> u16 {
    let r = (color[0].clamp(0.0, 1.0) * 7.0).round() as u16;
    let g = (color[1].clamp(0.0, 1.0) * 7.0).round() as u16;
    let b = (color[2].clamp(0.0, 1.0) * 7.0).round() as u16;
    (r << 6) | (g << 3) | b
}

fn smooth_normals_from_triangles(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    for triangle in indices.chunks_exact(3) {
        let a = vec3_from_array(positions[triangle[0] as usize]);
        let b = vec3_from_array(positions[triangle[1] as usize]);
        let c = vec3_from_array(positions[triangle[2] as usize]);
        let normal = (b - a).cross(c - a);
        for index in triangle {
            normals[*index as usize] += normal;
        }
    }

    normals
        .into_iter()
        .map(|normal| {
            if normal.length_squared() > 0.0001 {
                normal.normalize().to_array()
            } else {
                Vec3::Y.to_array()
            }
        })
        .collect()
}

fn vec3_from_array(position: [f32; 3]) -> Vec3 {
    Vec3::new(position[0], position[1], position[2])
}

fn hash01(x: i32, z: i32, salt: u32) -> f32 {
    let mut value = x as u32;
    value ^= (z as u32)
        .wrapping_add(0x9e37_79b9)
        .wrapping_add(value << 6)
        .wrapping_add(value >> 2);
    value ^= salt.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    value as f32 / u32::MAX as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    fn mesh_positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(positions)) => positions,
            _ => panic!("world floor mesh must contain Float32x3 positions"),
        }
    }

    #[test]
    fn world_floor_window_is_budgeted_and_centered() {
        let coords = desired_tile_coords(Vec3::new(18.0, 90.0, -24.0));

        assert_eq!(coords.len(), 9);
        assert_eq!(coords[0], WorldFloorTileCoord { x: 0, z: 0 });
        assert_eq!(WORLD_FLOOR_MAX_RESIDENT_TILES, 25);
        assert!(WORLD_FLOOR_MAX_RESIDENT_TILES >= coords.len());
    }

    #[test]
    fn pooled_world_floor_evicts_only_the_farthest_inactive_tile() {
        let center = WorldFloorTileCoord { x: 0, z: 0 };
        let active_tiles = desired_tile_coords(Vec3::ZERO)
            .into_iter()
            .collect::<HashSet<_>>();
        let mut resident_coords = active_tiles.iter().copied().collect::<Vec<_>>();
        resident_coords.extend([
            WorldFloorTileCoord { x: 2, z: 0 },
            WorldFloorTileCoord { x: -3, z: 1 },
            WorldFloorTileCoord { x: 1, z: 2 },
        ]);

        assert_eq!(
            farthest_inactive_tile(&resident_coords, &active_tiles, center),
            Some(WorldFloorTileCoord { x: -3, z: 1 })
        );
    }

    #[test]
    fn world_floor_tile_mesh_has_relief_and_bounded_cost() {
        let (_mesh, stats) = world_floor_tile_mesh(WorldFloorTileCoord { x: 1, z: -1 });

        assert_eq!(
            stats.vertex_count,
            (WORLD_FLOOR_GRID_SUBDIVISIONS + 1) * (WORLD_FLOOR_GRID_SUBDIVISIONS + 1)
        );
        assert_eq!(
            stats.triangle_count,
            WORLD_FLOOR_GRID_SUBDIVISIONS * WORLD_FLOOR_GRID_SUBDIVISIONS * 2
        );
        assert!(
            stats.max_height_y - stats.min_height_y > 3.0,
            "floor tiles should have visible relief"
        );
        assert!(
            stats.biome_mask.count_ones() >= 2,
            "large floor tiles should contain multiple biome regions"
        );
        assert!(stats.min_height_y > -12.0);
        assert!(stats.max_height_y < 24.0);
    }

    #[test]
    fn active_world_floor_tile_has_authored_landform_variety() {
        let (_mesh, stats) = world_floor_tile_mesh(WorldFloorTileCoord { x: 0, z: 0 });

        assert!(
            stats.max_height_y - stats.min_height_y >= 10.0,
            "visible floor tile should have readable lowland/ridge/mountain relief"
        );
        assert!(
            stats.feature_mask.count_ones() >= 5,
            "visible floor tile should combine biomes with river or high-peak features"
        );
        assert!(
            stats.color_band_count >= 12,
            "visible floor tile should have varied material/color bands"
        );
        assert!(
            stats.river_vertex_count >= 2,
            "visible floor tile should include a readable river/lowland cut"
        );
    }

    #[test]
    fn world_floor_biomes_cover_multiple_material_regions() {
        let mut biomes = HashSet::new();
        for z in -5..=5 {
            for x in -5..=5 {
                biomes.insert(tile_biome(WorldFloorTileCoord { x, z }));
            }
        }

        assert!(biomes.contains(&WorldFloorBiome::Wetland));
        assert!(biomes.contains(&WorldFloorBiome::Lowland));
        assert!(biomes.contains(&WorldFloorBiome::Ridge));
        assert!(biomes.contains(&WorldFloorBiome::Mountain));
    }

    #[test]
    fn world_floor_mesh_vertices_match_gameplay_terrain() {
        let (mesh, _stats) = world_floor_tile_mesh(WorldFloorTileCoord { x: -2, z: 1 });

        for [x, y, z] in mesh_positions(&mesh) {
            let expected = world_terrain_visual_y_at(Vec3::new(*x, 0.0, *z));
            assert!(
                (*y - expected).abs() < 0.0001,
                "rendered terrain at ({x}, {z}) differed from gameplay terrain by {} m",
                (*y - expected).abs()
            );
        }
    }

    #[test]
    fn adjacent_world_floor_tile_edges_match_exactly() {
        let (west, _stats) = world_floor_tile_mesh(WorldFloorTileCoord { x: 0, z: 0 });
        let (east, _stats) = world_floor_tile_mesh(WorldFloorTileCoord { x: 1, z: 0 });
        let west_positions = mesh_positions(&west);
        let east_positions = mesh_positions(&east);
        let stride = WORLD_FLOOR_GRID_SUBDIVISIONS + 1;

        for row in 0..stride {
            assert_eq!(
                west_positions[row * stride + WORLD_FLOOR_GRID_SUBDIVISIONS],
                east_positions[row * stride],
                "adjacent floor tiles must share the same edge vertices"
            );
        }
    }

    #[test]
    fn world_floor_ground_cover_is_bounded_and_tracks_the_surface() {
        let (mesh, vertex_count, triangle_count) =
            world_floor_ground_cover_mesh(WorldFloorTileCoord { x: 2, z: -1 });
        let positions = mesh_positions(&mesh);

        assert_eq!(positions.len(), vertex_count);
        assert!(vertex_count > 0);
        assert!(
            vertex_count
                <= WORLD_FLOOR_GROUND_COVER_PATCHES * WORLD_FLOOR_GROUND_COVER_BLADES_PER_PATCH * 8
        );
        assert_eq!(triangle_count * 3, mesh.indices().unwrap().len());

        for quad in positions.chunks_exact(4) {
            for [x, y, z] in &quad[..2] {
                let expected = world_terrain_visual_y_at(Vec3::new(*x, 0.0, *z)) + 0.04;
                assert!(
                    (*y - expected).abs() < 0.08,
                    "ground cover base at ({x}, {z}) missed terrain by {} m",
                    (*y - expected).abs()
                );
            }
        }
    }
}
