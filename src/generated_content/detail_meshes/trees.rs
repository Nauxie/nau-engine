use super::{
    super::{TreeSpecies, random_unit},
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub(crate) const TREE_CANOPY_LATITUDE_SEGMENTS: usize = 6;
pub(crate) const TREE_CANOPY_LONGITUDE_SEGMENTS: usize = 12;
pub(crate) const TREE_CANOPY_CARD_COUNT: usize = 18;
pub(crate) const TREE_TRUNK_SEGMENTS: usize = 10;
pub(crate) const TREE_TRUNK_RING_COUNT: usize = 5;
pub(crate) const TREE_BRANCH_COUNT: usize = 4;
pub(crate) const TREE_BRANCH_SEGMENTS: usize = 8;
pub(crate) const TREE_ROOT_FLARE_COUNT: usize = 5;
pub(crate) const TREE_ROOT_FLARE_SEGMENTS: usize = 8;

pub(crate) fn tree_trunk_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    tree_trunk_mesh_for_species(
        TreeSpecies::from_mesh_seed(seed).unwrap_or(TreeSpecies::BroadCanopy),
        radius,
        height,
        seed,
    )
}

pub(crate) fn tree_trunk_mesh_for_species(
    species: TreeSpecies,
    radius: f32,
    height: f32,
    seed: u32,
) -> Mesh {
    let trunk_vertices = TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT + 2;
    let branch_vertices = TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2;
    let root_vertices = TREE_ROOT_FLARE_COUNT * TREE_ROOT_FLARE_SEGMENTS * 2;
    let mut positions = Vec::with_capacity(trunk_vertices + branch_vertices + root_vertices);
    let mut normals = Vec::with_capacity(trunk_vertices + branch_vertices + root_vertices);
    let mut uvs = Vec::with_capacity(trunk_vertices + branch_vertices + root_vertices);
    let mut indices = Vec::with_capacity(
        TREE_TRUNK_SEGMENTS * 6 * (TREE_TRUNK_RING_COUNT + 1)
            + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 6
            + TREE_ROOT_FLARE_COUNT * TREE_ROOT_FLARE_SEGMENTS * 6,
    );
    let profile = trunk_profile(species);
    let bend_direction = tree_form_direction(seed);
    let bend_tangent = Vec2::new(-bend_direction.y, bend_direction.x);
    let bend = bend_direction * radius * profile.bend_scale
        + bend_tangent * radius * (random_unit(seed, 7, 17) - 0.5) * profile.bend_irregularity;
    let height_factors = [-0.5, -0.24, 0.04, 0.30, 0.5];
    let bend_progress = [0.0_f32, 0.22, 0.48, 0.74, 1.0];

    for ring_index in 0..TREE_TRUNK_RING_COUNT {
        let height_factor = height_factors[ring_index];
        let ring_radius = radius * profile.ring_radius_factors[ring_index];
        let center_offset = bend * bend_progress[ring_index].powf(profile.bend_curve);
        for segment in 0..TREE_TRUNK_SEGMENTS {
            let phase = segment as f32 / TREE_TRUNK_SEGMENTS as f32 * std::f32::consts::TAU;
            let bark_noise = 1.0
                + (random_unit(seed, segment as u32, ring_index as u32) - 0.5)
                    * profile.bark_variation;
            let x = center_offset.x + phase.cos() * ring_radius * bark_noise;
            let z = center_offset.y + phase.sin() * ring_radius * bark_noise;
            positions.push([x, height * height_factor, z]);
            normals.push(
                Vec3::new(phase.cos(), 0.16, phase.sin())
                    .normalize()
                    .to_array(),
            );
            uvs.push([
                segment as f32 / TREE_TRUNK_SEGMENTS as f32,
                ring_index as f32 / 2.0,
            ]);
        }
    }

    for ring in 0..TREE_TRUNK_RING_COUNT - 1 {
        let start = (ring * TREE_TRUNK_SEGMENTS) as u32;
        let next = ((ring + 1) * TREE_TRUNK_SEGMENTS) as u32;
        for segment in 0..TREE_TRUNK_SEGMENTS {
            let a = start + segment as u32;
            let b = start + ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
            let c = next + segment as u32;
            let d = next + ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let bottom_center = positions.len() as u32;
    positions.push([0.0, -height * 0.5, 0.0]);
    normals.push(Vec3::NEG_Y.to_array());
    uvs.push([0.5, 0.5]);
    let top_center = positions.len() as u32;
    positions.push([bend.x, height * 0.5, bend.y]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 0.5]);

    let top_start = ((TREE_TRUNK_RING_COUNT - 1) * TREE_TRUNK_SEGMENTS) as u32;
    for segment in 0..TREE_TRUNK_SEGMENTS {
        let next = ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
        indices.extend([bottom_center, segment as u32, next]);
        indices.extend([top_center, top_start + next, top_start + segment as u32]);
    }

    for branch in 0..TREE_BRANCH_COUNT {
        let shape = branch_shape(species, branch, radius, height, seed, bend_direction);
        let trunk_progress = (shape.height_factor + 0.5).clamp(0.0, 1.0);
        let center_offset = bend * trunk_progress.powf(profile.bend_curve);
        let start = Vec3::new(
            center_offset.x,
            height * shape.height_factor,
            center_offset.y,
        );
        let end = start
            + Vec3::new(
                shape.phase.cos() * shape.reach,
                shape.lift,
                shape.phase.sin() * shape.reach,
            );
        append_tapered_limb(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            start,
            end,
            radius * shape.base_radius_factor,
            radius * shape.tip_radius_factor,
            TREE_BRANCH_SEGMENTS,
        );
    }

    let root_shape = root_shape(species);
    for root in 0..TREE_ROOT_FLARE_COUNT {
        let root_phase = root as f32 / TREE_ROOT_FLARE_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, root as u32, 211) * 0.28;
        let direction =
            Vec3::new(root_phase.cos(), -root_shape.slope, root_phase.sin()).normalize();
        let start = Vec3::new(
            root_phase.cos() * radius * root_shape.start_radius_factor,
            -height * 0.44,
            root_phase.sin() * radius * root_shape.start_radius_factor,
        );
        let reach = radius
            * (root_shape.reach_base
                + random_unit(seed, root as u32, 223) * root_shape.reach_variation);
        let end = start + direction * reach + Vec3::Y * (-height * 0.035);
        append_tapered_limb(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            start,
            end,
            radius * root_shape.base_radius_factor,
            radius * root_shape.tip_radius_factor,
            TREE_ROOT_FLARE_SEGMENTS,
        );
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn tree_canopy_mesh(radius: f32, seed: u32) -> Mesh {
    tree_canopy_mesh_for_species(
        TreeSpecies::from_mesh_seed(seed).unwrap_or(TreeSpecies::BroadCanopy),
        radius,
        seed,
    )
}

pub(crate) fn tree_canopy_mesh_for_species(species: TreeSpecies, radius: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let primary = primary_canopy_lobe(species, radius, seed);
    append_ellipsoid_lobe(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        primary.center,
        primary.radii,
        TREE_CANOPY_LATITUDE_SEGMENTS,
        TREE_CANOPY_LONGITUDE_SEGMENTS,
        seed,
        primary.noise_strength,
    );

    for lobe in 0..5 {
        let secondary = secondary_canopy_lobe(species, lobe, radius, seed);
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            secondary.center,
            secondary.radii,
            4,
            8,
            seed.wrapping_add(100 + lobe as u32 * 17),
            secondary.noise_strength,
        );
    }

    for card in 0..TREE_CANOPY_CARD_COUNT {
        let card_shape = canopy_card(species, card, radius, seed);
        append_double_sided_detail_card(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            card_shape.center,
            card_shape.tangent,
            card_shape.up,
            card_shape.half_width,
            card_shape.half_height,
        );
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

#[derive(Clone, Copy)]
struct TrunkProfile {
    ring_radius_factors: [f32; TREE_TRUNK_RING_COUNT],
    bend_scale: f32,
    bend_irregularity: f32,
    bend_curve: f32,
    bark_variation: f32,
}

#[derive(Clone, Copy)]
struct BranchShape {
    height_factor: f32,
    phase: f32,
    reach: f32,
    lift: f32,
    base_radius_factor: f32,
    tip_radius_factor: f32,
}

#[derive(Clone, Copy)]
struct RootShape {
    slope: f32,
    start_radius_factor: f32,
    reach_base: f32,
    reach_variation: f32,
    base_radius_factor: f32,
    tip_radius_factor: f32,
}

#[derive(Clone, Copy)]
struct CanopyLobe {
    center: Vec3,
    radii: Vec3,
    noise_strength: f32,
}

#[derive(Clone, Copy)]
struct CanopyCard {
    center: Vec3,
    tangent: Vec3,
    up: Vec3,
    half_width: f32,
    half_height: f32,
}

fn trunk_profile(species: TreeSpecies) -> TrunkProfile {
    match species {
        TreeSpecies::BroadCanopy => TrunkProfile {
            ring_radius_factors: [1.42, 1.06, 0.84, 0.66, 0.48],
            bend_scale: 0.65,
            bend_irregularity: 0.24,
            bend_curve: 1.0,
            bark_variation: 0.24,
        },
        TreeSpecies::WindBent => TrunkProfile {
            ring_radius_factors: [1.50, 1.12, 0.90, 0.70, 0.50],
            bend_scale: 2.70,
            bend_irregularity: 0.30,
            bend_curve: 0.82,
            bark_variation: 0.28,
        },
        TreeSpecies::Orchard => TrunkProfile {
            ring_radius_factors: [1.58, 1.20, 0.91, 0.68, 0.50],
            bend_scale: 0.42,
            bend_irregularity: 0.18,
            bend_curve: 1.18,
            bark_variation: 0.20,
        },
        TreeSpecies::Cypress => TrunkProfile {
            ring_radius_factors: [1.24, 1.02, 0.84, 0.68, 0.52],
            bend_scale: 0.14,
            bend_irregularity: 0.08,
            bend_curve: 1.35,
            bark_variation: 0.12,
        },
        TreeSpecies::Willow => TrunkProfile {
            ring_radius_factors: [1.52, 1.18, 0.90, 0.67, 0.47],
            bend_scale: 1.08,
            bend_irregularity: 0.32,
            bend_curve: 0.92,
            bark_variation: 0.26,
        },
        TreeSpecies::AlpinePine => TrunkProfile {
            ring_radius_factors: [1.38, 1.04, 0.78, 0.55, 0.34],
            bend_scale: 0.28,
            bend_irregularity: 0.12,
            bend_curve: 1.24,
            bark_variation: 0.18,
        },
    }
}

fn branch_shape(
    species: TreeSpecies,
    branch: usize,
    radius: f32,
    height: f32,
    seed: u32,
    bend_direction: Vec2,
) -> BranchShape {
    let sample = branch as u32;
    let radial_phase = branch as f32 / TREE_BRANCH_COUNT as f32 * std::f32::consts::TAU
        + random_unit(seed, sample, 89) * 0.55;
    let bend_phase = bend_direction.y.atan2(bend_direction.x);
    let reach_noise = random_unit(seed, sample, 97);
    let lift_noise = random_unit(seed, sample, 107);

    match species {
        TreeSpecies::BroadCanopy => BranchShape {
            height_factor: -0.03 + branch as f32 * 0.14,
            phase: radial_phase,
            reach: radius * (3.0 + reach_noise * 1.05),
            lift: height * (0.10 + lift_noise * 0.06),
            base_radius_factor: 0.34,
            tip_radius_factor: 0.12,
        },
        TreeSpecies::WindBent => BranchShape {
            height_factor: -0.12 + branch as f32 * 0.12,
            phase: bend_phase + (branch as f32 - 1.5) * 0.22 + (lift_noise - 0.5) * 0.16,
            reach: radius * (3.85 + reach_noise * 1.15),
            lift: height * (0.04 + lift_noise * 0.05),
            base_radius_factor: 0.32,
            tip_radius_factor: 0.10,
        },
        TreeSpecies::Orchard => BranchShape {
            height_factor: -0.11 + branch as f32 * 0.11,
            phase: (branch % 2) as f32 * std::f32::consts::PI
                + (branch / 2) as f32 * 0.42
                + (reach_noise - 0.5) * 0.18,
            reach: radius * (3.25 + reach_noise * 0.80),
            lift: height * (0.07 + lift_noise * 0.06),
            base_radius_factor: 0.38,
            tip_radius_factor: 0.14,
        },
        TreeSpecies::Cypress => BranchShape {
            height_factor: 0.02 + branch as f32 * 0.10,
            phase: radial_phase,
            reach: radius * (2.85 + reach_noise * 0.55),
            lift: height * (0.20 + lift_noise * 0.08),
            base_radius_factor: 0.25,
            tip_radius_factor: 0.08,
        },
        TreeSpecies::Willow => BranchShape {
            height_factor: -0.08 + branch as f32 * 0.13,
            phase: radial_phase,
            reach: radius * (3.45 + reach_noise * 0.90),
            lift: -height * (0.025 + lift_noise * 0.035),
            base_radius_factor: 0.29,
            tip_radius_factor: 0.08,
        },
        TreeSpecies::AlpinePine => BranchShape {
            height_factor: -0.14 + branch as f32 * 0.15,
            phase: radial_phase,
            reach: radius * (3.80 - branch as f32 * 0.22 + reach_noise * 0.42),
            lift: height * (0.045 + lift_noise * 0.045),
            base_radius_factor: 0.30,
            tip_radius_factor: 0.09,
        },
    }
}

fn root_shape(species: TreeSpecies) -> RootShape {
    match species {
        TreeSpecies::BroadCanopy => RootShape {
            slope: 0.18,
            start_radius_factor: 0.48,
            reach_base: 2.10,
            reach_variation: 0.70,
            base_radius_factor: 0.30,
            tip_radius_factor: 0.08,
        },
        TreeSpecies::WindBent => RootShape {
            slope: 0.14,
            start_radius_factor: 0.54,
            reach_base: 2.35,
            reach_variation: 0.72,
            base_radius_factor: 0.32,
            tip_radius_factor: 0.08,
        },
        TreeSpecies::Orchard => RootShape {
            slope: 0.20,
            start_radius_factor: 0.56,
            reach_base: 2.20,
            reach_variation: 0.60,
            base_radius_factor: 0.34,
            tip_radius_factor: 0.09,
        },
        TreeSpecies::Cypress => RootShape {
            slope: 0.22,
            start_radius_factor: 0.42,
            reach_base: 1.95,
            reach_variation: 0.52,
            base_radius_factor: 0.24,
            tip_radius_factor: 0.07,
        },
        TreeSpecies::Willow => RootShape {
            slope: 0.16,
            start_radius_factor: 0.52,
            reach_base: 2.30,
            reach_variation: 0.76,
            base_radius_factor: 0.31,
            tip_radius_factor: 0.08,
        },
        TreeSpecies::AlpinePine => RootShape {
            slope: 0.24,
            start_radius_factor: 0.46,
            reach_base: 2.15,
            reach_variation: 0.62,
            base_radius_factor: 0.28,
            tip_radius_factor: 0.07,
        },
    }
}

fn primary_canopy_lobe(species: TreeSpecies, radius: f32, seed: u32) -> CanopyLobe {
    let direction = tree_form_direction(seed);
    let wind = Vec3::new(direction.x, 0.0, direction.y);

    match species {
        TreeSpecies::BroadCanopy => CanopyLobe {
            center: Vec3::ZERO,
            radii: Vec3::new(radius * 1.08, radius * 0.82, radius),
            noise_strength: 0.22,
        },
        TreeSpecies::WindBent => CanopyLobe {
            center: wind * radius * 0.22 + Vec3::Y * radius * 0.02,
            radii: Vec3::new(
                radius * (0.78 + direction.x.abs() * 0.42),
                radius * 0.60,
                radius * (0.78 + direction.y.abs() * 0.42),
            ),
            noise_strength: 0.24,
        },
        TreeSpecies::Orchard => CanopyLobe {
            center: Vec3::Y * radius * 0.04,
            radii: Vec3::new(radius * 0.90, radius * 0.72, radius * 0.88),
            noise_strength: 0.18,
        },
        TreeSpecies::Cypress => CanopyLobe {
            center: Vec3::Y * radius * 0.18,
            radii: Vec3::new(radius * 0.48, radius * 1.32, radius * 0.46),
            noise_strength: 0.14,
        },
        TreeSpecies::Willow => CanopyLobe {
            center: Vec3::Y * radius * 0.18,
            radii: Vec3::new(radius * 1.15, radius * 0.54, radius * 1.08),
            noise_strength: 0.20,
        },
        TreeSpecies::AlpinePine => CanopyLobe {
            center: Vec3::Y * radius * 0.08,
            radii: Vec3::new(radius * 0.58, radius * 1.22, radius * 0.56),
            noise_strength: 0.16,
        },
    }
}

fn secondary_canopy_lobe(species: TreeSpecies, lobe: usize, radius: f32, seed: u32) -> CanopyLobe {
    let phase =
        lobe as f32 / 5.0 * std::f32::consts::TAU + random_unit(seed, lobe as u32, 71) * 0.45;
    let outward = Vec3::new(phase.cos(), 0.0, phase.sin());

    match species {
        TreeSpecies::BroadCanopy => CanopyLobe {
            center: Vec3::new(
                phase.cos() * radius * (0.30 + random_unit(seed, lobe as u32, 83) * 0.12),
                radius * (-0.02 + lobe as f32 * 0.035),
                phase.sin() * radius * (0.26 + random_unit(seed, lobe as u32, 97) * 0.10),
            ),
            radii: Vec3::new(radius * 0.58, radius * 0.50, radius * 0.54),
            noise_strength: 0.18,
        },
        TreeSpecies::WindBent => {
            let direction = tree_form_direction(seed);
            let wind = Vec3::new(direction.x, 0.0, direction.y);
            let cross = Vec3::new(-direction.y, 0.0, direction.x);
            let stream = lobe as f32 / 4.0;
            CanopyLobe {
                center: wind * radius * (-0.28 + stream * 0.86)
                    + cross * radius * (lobe as f32 - 2.0) * 0.10
                    + Vec3::Y * radius * (-0.12 + stream * 0.20),
                radii: Vec3::new(
                    radius * (0.44 + direction.x.abs() * 0.18),
                    radius * 0.38,
                    radius * (0.44 + direction.y.abs() * 0.18),
                ),
                noise_strength: 0.20,
            }
        }
        TreeSpecies::Orchard => CanopyLobe {
            center: outward * radius * (0.30 + random_unit(seed, lobe as u32, 83) * 0.08)
                + Vec3::Y * radius * (-0.05 + (lobe % 3) as f32 * 0.05),
            radii: Vec3::new(radius * 0.55, radius * 0.53, radius * 0.54),
            noise_strength: 0.15,
        },
        TreeSpecies::Cypress => CanopyLobe {
            center: outward * radius * 0.13 + Vec3::Y * radius * (-0.64 + lobe as f32 * 0.32),
            radii: Vec3::new(radius * 0.34, radius * 0.42, radius * 0.32),
            noise_strength: 0.12,
        },
        TreeSpecies::Willow => CanopyLobe {
            center: outward * radius * (0.42 + random_unit(seed, lobe as u32, 83) * 0.08)
                + Vec3::Y * radius * (-0.02 - (lobe % 2) as f32 * 0.10),
            radii: Vec3::new(radius * 0.62, radius * 0.36, radius * 0.58),
            noise_strength: 0.16,
        },
        TreeSpecies::AlpinePine => {
            let tier = lobe as f32;
            let spread = 0.28 + tier * 0.075;
            CanopyLobe {
                center: outward * radius * 0.08 + Vec3::Y * radius * (0.65 - tier * 0.28),
                radii: Vec3::new(radius * spread, radius * 0.30, radius * spread * 0.94),
                noise_strength: 0.13,
            }
        }
    }
}

fn canopy_card(species: TreeSpecies, card: usize, radius: f32, seed: u32) -> CanopyCard {
    let lower_skirt = card >= TREE_CANOPY_CARD_COUNT.saturating_sub(6);
    let phase = card as f32 / TREE_CANOPY_CARD_COUNT as f32 * std::f32::consts::TAU
        + random_unit(seed, card as u32, 151) * 0.24;
    let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
    let tangent = Vec3::new(-phase.sin(), 0.0, phase.cos()).normalize();

    match species {
        TreeSpecies::BroadCanopy => CanopyCard {
            center: Vec3::new(
                outward.x
                    * radius
                    * if lower_skirt {
                        0.74 + random_unit(seed, card as u32, 163) * 0.16
                    } else {
                        0.58 + random_unit(seed, card as u32, 163) * 0.22
                    },
                radius
                    * if lower_skirt {
                        -0.34 + random_unit(seed, card as u32, 167) * 0.16
                    } else {
                        -0.08 + random_unit(seed, card as u32, 167) * 0.34
                    },
                outward.z
                    * radius
                    * if lower_skirt {
                        0.70 + random_unit(seed, card as u32, 173) * 0.16
                    } else {
                        0.54 + random_unit(seed, card as u32, 173) * 0.20
                    },
            ),
            tangent,
            up: if lower_skirt {
                (Vec3::Y * 0.62 - outward * 0.26).normalize()
            } else {
                (Vec3::Y + outward * 0.16).normalize()
            },
            half_width: radius
                * if lower_skirt {
                    0.18 + random_unit(seed, card as u32, 179) * 0.07
                } else {
                    0.20 + random_unit(seed, card as u32, 179) * 0.08
                },
            half_height: radius
                * if lower_skirt {
                    0.36 + random_unit(seed, card as u32, 181) * 0.14
                } else {
                    0.28 + random_unit(seed, card as u32, 181) * 0.12
                },
        },
        TreeSpecies::WindBent => {
            let direction = tree_form_direction(seed);
            let wind = Vec3::new(direction.x, 0.0, direction.y);
            let cross = Vec3::new(-direction.y, 0.0, direction.x);
            let stream = card as f32 / (TREE_CANOPY_CARD_COUNT - 1) as f32;
            CanopyCard {
                center: wind * radius * (-0.42 + stream * 1.05)
                    + cross * radius * ((card % 6) as f32 - 2.5) * 0.12
                    + Vec3::Y * radius * (-0.18 + random_unit(seed, card as u32, 167) * 0.34),
                tangent: cross,
                up: (Vec3::Y * 0.72 - wind * 0.32).normalize(),
                half_width: radius * (0.18 + random_unit(seed, card as u32, 179) * 0.08),
                half_height: radius * (0.31 + random_unit(seed, card as u32, 181) * 0.13),
            }
        }
        TreeSpecies::Orchard => CanopyCard {
            center: outward
                * radius
                * if lower_skirt {
                    0.68 + random_unit(seed, card as u32, 163) * 0.12
                } else {
                    0.50 + random_unit(seed, card as u32, 163) * 0.18
                }
                + Vec3::Y
                    * radius
                    * if lower_skirt {
                        -0.25 + random_unit(seed, card as u32, 167) * 0.12
                    } else {
                        -0.05 + random_unit(seed, card as u32, 167) * 0.24
                    },
            tangent,
            up: (Vec3::Y + outward * 0.12).normalize(),
            half_width: radius * (0.17 + random_unit(seed, card as u32, 179) * 0.06),
            half_height: radius * (0.25 + random_unit(seed, card as u32, 181) * 0.10),
        },
        TreeSpecies::Cypress => {
            let level = (card % 9) as f32 / 8.0;
            CanopyCard {
                center: outward * radius * (0.24 + random_unit(seed, card as u32, 163) * 0.12)
                    + Vec3::Y * radius * (-0.82 + level * 1.64),
                tangent,
                up: (Vec3::Y + outward * 0.06).normalize(),
                half_width: radius * (0.12 + random_unit(seed, card as u32, 179) * 0.05),
                half_height: radius * (0.24 + random_unit(seed, card as u32, 181) * 0.10),
            }
        }
        TreeSpecies::Willow => CanopyCard {
            center: outward
                * radius
                * if lower_skirt {
                    0.78 + random_unit(seed, card as u32, 163) * 0.14
                } else {
                    0.62 + random_unit(seed, card as u32, 163) * 0.20
                }
                + Vec3::Y
                    * radius
                    * if lower_skirt {
                        -0.62 + random_unit(seed, card as u32, 167) * 0.14
                    } else {
                        -0.22 + random_unit(seed, card as u32, 167) * 0.28
                    },
            tangent,
            up: (Vec3::Y * 0.62 - outward * 0.42).normalize(),
            half_width: radius * (0.15 + random_unit(seed, card as u32, 179) * 0.07),
            half_height: radius * (0.50 + random_unit(seed, card as u32, 181) * 0.20),
        },
        TreeSpecies::AlpinePine => {
            let tier = (card / 3) as f32;
            let spread = 0.25 + tier * 0.09;
            CanopyCard {
                center: outward * radius * spread + Vec3::Y * radius * (0.68 - tier * 0.28),
                tangent,
                up: (Vec3::Y * 0.70 - outward * 0.32).normalize(),
                half_width: radius
                    * (0.13 + tier * 0.018 + random_unit(seed, card as u32, 179) * 0.03),
                half_height: radius
                    * (0.28 + tier * 0.03 + random_unit(seed, card as u32, 181) * 0.06),
            }
        }
    }
}

fn tree_form_direction(seed: u32) -> Vec2 {
    let phase = if TreeSpecies::from_mesh_seed(seed).is_some() {
        (((seed >> 16) & 0x1f) as f32 + 0.5) / 32.0 * std::f32::consts::TAU
    } else {
        random_unit(seed, 3, 11) * std::f32::consts::TAU
    };
    Vec2::new(phase.cos(), phase.sin())
}

#[allow(clippy::too_many_arguments)]
fn append_tapered_limb(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    start: Vec3,
    end: Vec3,
    base_radius: f32,
    tip_radius: f32,
    radial_segments: usize,
) {
    let axis = (end - start).normalize_or_zero();
    if axis.length_squared() <= 0.0001 {
        return;
    }
    let side_seed = if axis.dot(Vec3::Y).abs() > 0.92 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let side = axis.cross(side_seed).normalize();
    let bitangent = side.cross(axis).normalize();
    let first = positions.len() as u32;

    for (ring, (center, radius)) in [(start, base_radius), (end, tip_radius)]
        .into_iter()
        .enumerate()
    {
        for segment in 0..radial_segments {
            let phase = segment as f32 / radial_segments as f32 * std::f32::consts::TAU;
            let radial = side * phase.cos() + bitangent * phase.sin();
            positions.push((center + radial * radius).to_array());
            normals.push(radial.normalize().to_array());
            uvs.push([segment as f32 / radial_segments as f32, ring as f32]);
        }
    }

    for segment in 0..radial_segments {
        let a = first + segment as u32;
        let b = first + ((segment + 1) % radial_segments) as u32;
        let c = first + radial_segments as u32 + segment as u32;
        let d = first + radial_segments as u32 + ((segment + 1) % radial_segments) as u32;
        indices.extend([a, c, b, b, c, d]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[derive(Clone, Copy)]
    struct Bounds {
        min: Vec3,
        max: Vec3,
        center: Vec3,
    }

    impl Bounds {
        fn horizontal_span(self) -> f32 {
            (self.max.x - self.min.x).max(self.max.z - self.min.z)
        }

        fn vertical_span(self) -> f32 {
            self.max.y - self.min.y
        }

        fn horizontal_center_offset(self) -> f32 {
            Vec2::new(self.center.x, self.center.z).length()
        }
    }

    #[test]
    fn tagged_species_meshes_are_deterministic_and_match_compatibility_wrappers() {
        for species in TreeSpecies::ALL {
            let seed = species.mesh_seed(42);
            let trunk = tree_trunk_mesh_for_species(species, 0.3, 4.0, seed);
            let repeated_trunk = tree_trunk_mesh_for_species(species, 0.3, 4.0, seed);
            let wrapped_trunk = tree_trunk_mesh(0.3, 4.0, seed);
            let canopy = tree_canopy_mesh_for_species(species, 1.4, seed);
            let repeated_canopy = tree_canopy_mesh_for_species(species, 1.4, seed);
            let wrapped_canopy = tree_canopy_mesh(1.4, seed);

            assert_eq!(mesh_positions(&trunk), mesh_positions(&repeated_trunk));
            assert_eq!(mesh_positions(&trunk), mesh_positions(&wrapped_trunk));
            assert_eq!(mesh_positions(&canopy), mesh_positions(&repeated_canopy));
            assert_eq!(mesh_positions(&canopy), mesh_positions(&wrapped_canopy));
        }
    }

    #[test]
    fn every_species_preserves_tree_mesh_complexity_floors() {
        let expected_trunk_vertices = TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT
            + 2
            + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2
            + TREE_ROOT_FLARE_COUNT * TREE_ROOT_FLARE_SEGMENTS * 2;
        let branch_vertices_start = TREE_TRUNK_SEGMENTS * TREE_TRUNK_RING_COUNT + 2;
        let root_vertices_start =
            branch_vertices_start + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2;

        for species in TreeSpecies::ALL {
            let trunk = tree_trunk_mesh_for_species(species, 0.3, 4.0, species.mesh_seed(123));
            let positions = mesh_positions(&trunk);
            let bottom_radius = average_xz_radius(&positions[..TREE_TRUNK_SEGMENTS]);
            let branch_vertex_count = positions[branch_vertices_start..root_vertices_start].len();
            let root_vertex_count = positions[root_vertices_start..].len();
            let max_branch_reach = positions[branch_vertices_start..root_vertices_start]
                .iter()
                .map(|position| Vec2::new(position[0], position[2]).length())
                .fold(0.0, f32::max);
            let max_root_reach = positions[root_vertices_start..]
                .iter()
                .map(|position| Vec2::new(position[0], position[2]).length())
                .fold(0.0, f32::max);
            let canopy = tree_canopy_mesh_for_species(species, 1.4, species.mesh_seed(456));
            let canopy_bounds = mesh_bounds(&canopy);

            assert_eq!(trunk.count_vertices(), expected_trunk_vertices);
            assert!(branch_vertex_count >= 4 * TREE_BRANCH_SEGMENTS * 2);
            assert!(root_vertex_count >= 5 * TREE_ROOT_FLARE_SEGMENTS * 2);
            assert!(max_branch_reach > bottom_radius * 1.8);
            assert!(max_root_reach > bottom_radius * 1.35);
            assert!(canopy.count_vertices() >= 460);
            assert!(
                canopy_bounds.vertical_span() / canopy_bounds.horizontal_span() >= 0.45,
                "{} canopy should retain the visual export profile floor",
                species.visual_name()
            );
        }
    }

    #[test]
    fn species_meshes_have_distinct_silhouette_profiles() {
        let broad = tree_canopy_mesh_for_species(TreeSpecies::BroadCanopy, 1.0, 77);
        let wind = tree_canopy_mesh_for_species(TreeSpecies::WindBent, 1.0, 77);
        let orchard = tree_canopy_mesh_for_species(TreeSpecies::Orchard, 1.0, 77);
        let cypress = tree_canopy_mesh_for_species(TreeSpecies::Cypress, 1.0, 77);
        let willow = tree_canopy_mesh_for_species(TreeSpecies::Willow, 1.0, 77);
        let alpine = tree_canopy_mesh_for_species(TreeSpecies::AlpinePine, 1.0, 77);
        let broad_bounds = mesh_bounds(&broad);
        let wind_bounds = mesh_bounds(&wind);
        let orchard_bounds = mesh_bounds(&orchard);
        let cypress_bounds = mesh_bounds(&cypress);
        let willow_bounds = mesh_bounds(&willow);
        let alpine_bounds = mesh_bounds(&alpine);
        let wind_trunk = tree_trunk_mesh_for_species(TreeSpecies::WindBent, 0.3, 4.0, 77);
        let cypress_trunk = tree_trunk_mesh_for_species(TreeSpecies::Cypress, 0.3, 4.0, 77);

        assert!(broad_bounds.horizontal_span() > orchard_bounds.horizontal_span());
        assert!(broad_bounds.horizontal_span() > cypress_bounds.horizontal_span() * 1.45);
        assert!(cypress_bounds.vertical_span() > broad_bounds.vertical_span() * 1.35);
        assert!(willow_bounds.min.y < orchard_bounds.min.y - 0.12);
        assert!(
            alpine_bounds.vertical_span() / alpine_bounds.horizontal_span()
                > orchard_bounds.vertical_span() / orchard_bounds.horizontal_span() * 1.20
        );
        assert!(
            wind_bounds.horizontal_center_offset()
                > orchard_bounds.horizontal_center_offset() + 0.08
        );
        assert!(
            trunk_top_center_offset(&wind_trunk) > trunk_top_center_offset(&cypress_trunk) * 4.0
        );
    }

    fn mesh_positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("tree mesh should have positions")
        {
            VertexAttributeValues::Float32x3(values) => values,
            _ => panic!("tree mesh positions should be Float32x3"),
        }
    }

    fn mesh_bounds(mesh: &Mesh) -> Bounds {
        let positions = mesh_positions(mesh);
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let mut center = Vec3::ZERO;

        for position in positions {
            let position = Vec3::from_array(*position);
            min = min.min(position);
            max = max.max(position);
            center += position;
        }

        Bounds {
            min,
            max,
            center: center / positions.len() as f32,
        }
    }

    fn average_xz_radius(points: &[[f32; 3]]) -> f32 {
        let center = points
            .iter()
            .map(|position| Vec2::new(position[0], position[2]))
            .sum::<Vec2>()
            / points.len() as f32;

        points
            .iter()
            .map(|position| (Vec2::new(position[0], position[2]) - center).length())
            .sum::<f32>()
            / points.len() as f32
    }

    fn trunk_top_center_offset(mesh: &Mesh) -> f32 {
        let positions = mesh_positions(mesh);
        let top_start = TREE_TRUNK_SEGMENTS * (TREE_TRUNK_RING_COUNT - 1);
        let center = positions[top_start..top_start + TREE_TRUNK_SEGMENTS]
            .iter()
            .map(|position| Vec2::new(position[0], position[2]))
            .sum::<Vec2>()
            / TREE_TRUNK_SEGMENTS as f32;
        center.length()
    }
}
