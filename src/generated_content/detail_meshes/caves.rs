use super::{super::random_unit, shared::append_ellipsoid_lobe};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

pub(crate) const CAVE_MOUTH_ARCH_STONES: usize = 13;
pub(crate) const HANGING_ROOT_STRANDS: usize = 11;
pub(crate) const HANGING_ROOT_SEGMENTS: usize = 8;
pub(crate) const UNDERHANG_SHELF_SEGMENTS: usize = 28;

const CAVE_ARCH_LOBE_LATITUDE_SEGMENTS: usize = 4;
const CAVE_ARCH_LOBE_LONGITUDE_SEGMENTS: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IslandUnderRouteVisualKind {
    CaveMouthArch,
    UnderhangShelf,
    HangingRoots,
}

impl IslandUnderRouteVisualKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::CaveMouthArch => "under_route_cave_mouth",
            Self::UnderhangShelf => "under_route_hanging_shelf",
            Self::HangingRoots => "under_route_hanging_roots",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::CaveMouthArch => "under-route cave mouth arch",
            Self::UnderhangShelf => "under-route hanging shelf",
            Self::HangingRoots => "under-route hanging roots",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum IslandUnderRouteVisualMesh {
    CaveMouthArch {
        width: f32,
        height: f32,
        depth: f32,
    },
    UnderhangShelf {
        width: f32,
        depth: f32,
        thickness: f32,
    },
    HangingRoots {
        width: f32,
        height: f32,
        depth: f32,
    },
}

impl IslandUnderRouteVisualMesh {
    fn build(self, seed: u32) -> Mesh {
        match self {
            Self::CaveMouthArch {
                width,
                height,
                depth,
            } => cave_mouth_arch_mesh(width, height, depth, seed),
            Self::UnderhangShelf {
                width,
                depth,
                thickness,
            } => underhang_shelf_mesh(width, depth, thickness, seed),
            Self::HangingRoots {
                width,
                height,
                depth,
            } => hanging_root_curtain_mesh(width, height, depth, seed),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandUnderRouteVisualSpec {
    pub(crate) kind: IslandUnderRouteVisualKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) camera_half_extents: Vec3,
    mesh: IslandUnderRouteVisualMesh,
    seed: u32,
}

impl IslandUnderRouteVisualSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        self.mesh.build(self.seed)
    }
}

pub(crate) fn island_under_route_visual_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandUnderRouteVisualSpec> {
    let Some(segment) = island.under_route_segment() else {
        return Vec::new();
    };

    let entry_yaw = yaw_between(segment.entry, segment.midpoint);
    let exit_yaw = yaw_between(segment.midpoint, segment.exit);
    let arch_width = segment.clearance_radius_m * 2.35;
    let arch_height = segment.clearance_radius_m * 1.65;
    let arch_depth = segment.clearance_radius_m * 0.55;
    let shelf_width = segment.clearance_radius_m * 4.4;
    let shelf_depth = segment.clearance_radius_m * 2.45;
    let shelf_thickness = (segment.clearance_radius_m * 0.32).max(4.0);
    let shelf_translation = segment.midpoint - Vec3::Y * (segment.clearance_radius_m * 0.88);
    let root_width = segment.clearance_radius_m * 2.65;
    let root_height = segment.clearance_radius_m * 1.05;
    let root_depth = segment.clearance_radius_m * 0.72;
    let root_translation = segment.midpoint + Vec3::Y * (segment.clearance_radius_m * 0.78);

    vec![
        IslandUnderRouteVisualSpec {
            kind: IslandUnderRouteVisualKind::CaveMouthArch,
            label: "underhang entry arch",
            translation: segment.entry,
            rotation_y: entry_yaw,
            camera_half_extents: Vec3::new(arch_width * 0.55, arch_height * 0.52, arch_depth),
            mesh: IslandUnderRouteVisualMesh::CaveMouthArch {
                width: arch_width,
                height: arch_height,
                depth: arch_depth,
            },
            seed: 41_000 + island_index as u32 * 211,
        },
        IslandUnderRouteVisualSpec {
            kind: IslandUnderRouteVisualKind::UnderhangShelf,
            label: "underside glide shelf",
            translation: shelf_translation,
            rotation_y: entry_yaw * 0.55 + exit_yaw * 0.45,
            camera_half_extents: Vec3::new(shelf_width * 0.50, shelf_thickness, shelf_depth * 0.50),
            mesh: IslandUnderRouteVisualMesh::UnderhangShelf {
                width: shelf_width,
                depth: shelf_depth,
                thickness: shelf_thickness,
            },
            seed: 42_000 + island_index as u32 * 223,
        },
        IslandUnderRouteVisualSpec {
            kind: IslandUnderRouteVisualKind::HangingRoots,
            label: "hanging root curtain",
            translation: root_translation,
            rotation_y: entry_yaw * 0.35 + exit_yaw * 0.65,
            camera_half_extents: Vec3::new(
                root_width * 0.50,
                root_height * 0.50,
                root_depth * 0.50,
            ),
            mesh: IslandUnderRouteVisualMesh::HangingRoots {
                width: root_width,
                height: root_height,
                depth: root_depth,
            },
            seed: 44_000 + island_index as u32 * 229,
        },
        IslandUnderRouteVisualSpec {
            kind: IslandUnderRouteVisualKind::CaveMouthArch,
            label: "updraft skylight exit arch",
            translation: segment.exit,
            rotation_y: exit_yaw,
            camera_half_extents: Vec3::new(arch_width * 0.48, arch_height * 0.48, arch_depth),
            mesh: IslandUnderRouteVisualMesh::CaveMouthArch {
                width: arch_width * 0.88,
                height: arch_height * 0.92,
                depth: arch_depth,
            },
            seed: 43_000 + island_index as u32 * 227,
        },
    ]
}

pub(crate) fn cave_mouth_arch_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for stone in 0..CAVE_MOUTH_ARCH_STONES {
        let t = stone as f32 / (CAVE_MOUTH_ARCH_STONES - 1) as f32;
        let angle = std::f32::consts::PI - t * std::f32::consts::PI;
        let shoulder = (1.0 - (t - 0.5).abs() * 2.0).max(0.0);
        let center = Vec3::new(
            angle.cos() * width * 0.48,
            -height * 0.28 + angle.sin().max(0.0) * height * 0.78,
            (random_unit(seed, stone as u32, 1_307) - 0.5) * depth * 0.22,
        );
        let radius = Vec3::new(
            width * (0.085 + random_unit(seed, stone as u32, 1_311) * 0.035),
            height * (0.10 + shoulder * 0.035),
            depth * (0.34 + random_unit(seed, stone as u32, 1_313) * 0.16),
        );

        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            radius,
            CAVE_ARCH_LOBE_LATITUDE_SEGMENTS,
            CAVE_ARCH_LOBE_LONGITUDE_SEGMENTS,
            seed.wrapping_add(stone as u32 * 101),
            0.32,
        );
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn hanging_root_curtain_mesh(width: f32, height: f32, depth: f32, seed: u32) -> Mesh {
    let vertices_per_strand = (HANGING_ROOT_SEGMENTS + 1) * 4;
    let indices_per_strand = HANGING_ROOT_SEGMENTS * 24;
    let mut positions = Vec::with_capacity(HANGING_ROOT_STRANDS * vertices_per_strand);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(HANGING_ROOT_STRANDS * indices_per_strand);

    for strand in 0..HANGING_ROOT_STRANDS {
        let strand_t = strand as f32 / (HANGING_ROOT_STRANDS - 1) as f32;
        let seed_offset = strand as u32 * 53;
        let base_x =
            (strand_t - 0.5) * width + (random_unit(seed, seed_offset, 1_601) - 0.5) * width * 0.08;
        let base_z = (random_unit(seed, seed_offset, 1_607) - 0.5) * depth;
        let strand_height = height * (0.72 + random_unit(seed, seed_offset, 1_613) * 0.28);
        let root_radius =
            (width * (0.0045 + random_unit(seed, seed_offset, 1_619) * 0.0025)).clamp(0.08, 0.20);
        let phase = random_unit(seed, seed_offset, 1_627) * std::f32::consts::TAU;
        let start = positions.len() as u32;

        for segment in 0..=HANGING_ROOT_SEGMENTS {
            let v = segment as f32 / HANGING_ROOT_SEGMENTS as f32;
            let sway = (v * std::f32::consts::PI).sin();
            let center = Vec3::new(
                base_x + (v * 4.7 + phase).sin() * width * 0.025 * sway,
                -strand_height * v,
                base_z + (v * 5.9 + phase * 0.7).cos() * depth * 0.055 * sway,
            );
            let taper = 1.0 - v * 0.58;
            let side_x = Vec3::X * root_radius * taper;
            let side_z = Vec3::Z * root_radius * taper;
            positions.extend([
                (center - side_x).to_array(),
                (center + side_x).to_array(),
                (center - side_z).to_array(),
                (center + side_z).to_array(),
            ]);
            normals.extend([
                Vec3::Z.to_array(),
                Vec3::Z.to_array(),
                Vec3::X.to_array(),
                Vec3::X.to_array(),
            ]);
            uvs.extend([[0.0, v], [1.0, v], [0.0, v], [1.0, v]]);
        }

        for segment in 0..HANGING_ROOT_SEGMENTS {
            let current = start + (segment * 4) as u32;
            let next = current + 4;
            append_double_sided_strip_indices(&mut indices, current, current + 1, next, next + 1);
            append_double_sided_strip_indices(
                &mut indices,
                current + 2,
                current + 3,
                next + 2,
                next + 3,
            );
        }
    }

    build_mesh(positions, normals, uvs, indices)
}

pub(crate) fn underhang_shelf_mesh(width: f32, depth: f32, thickness: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity(UNDERHANG_SHELF_SEGMENTS * 2);
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(UNDERHANG_SHELF_SEGMENTS * 12);

    for layer in 0..2 {
        let y = if layer == 0 {
            thickness * 0.5
        } else {
            -thickness * 0.5
        };
        let normal = if layer == 0 { Vec3::Y } else { Vec3::NEG_Y };
        for segment in 0..UNDERHANG_SHELF_SEGMENTS {
            let angle = segment as f32 / UNDERHANG_SHELF_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_noise =
                1.0 + (random_unit(seed, segment as u32 + layer as u32 * 29, 1_401) - 0.5) * 0.18;
            let fracture = (angle * 5.0 + seed as f32 * 0.013).sin() * 0.05;
            let x = angle.cos() * width * 0.5 * (edge_noise + fracture);
            let z = angle.sin() * depth * 0.5 * (edge_noise - fracture * 0.6);
            positions.push([x, y + fracture * thickness * 0.25, z]);
            normals.push(normal.to_array());
            uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
        }
    }

    let top = |segment: usize| -> u32 { (segment % UNDERHANG_SHELF_SEGMENTS) as u32 };
    let bottom = |segment: usize| -> u32 {
        (UNDERHANG_SHELF_SEGMENTS + segment % UNDERHANG_SHELF_SEGMENTS) as u32
    };

    for segment in 1..UNDERHANG_SHELF_SEGMENTS - 1 {
        indices.extend([top(0), top(segment), top(segment + 1)]);
        indices.extend([bottom(0), bottom(segment + 1), bottom(segment)]);
    }
    for segment in 0..UNDERHANG_SHELF_SEGMENTS {
        indices.extend([
            top(segment),
            top(segment + 1),
            bottom(segment),
            top(segment + 1),
            bottom(segment + 1),
            bottom(segment),
        ]);
    }

    build_mesh(positions, normals, uvs, indices)
}

fn append_double_sided_strip_indices(indices: &mut Vec<u32>, a0: u32, a1: u32, b0: u32, b1: u32) {
    indices.extend([a0, b0, a1, a1, b0, b1, a1, b0, a0, b1, b0, a1]);
}

fn yaw_between(from: Vec3, to: Vec3) -> f32 {
    let delta = to - from;
    delta.x.atan2(delta.z)
}

fn build_mesh(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}
