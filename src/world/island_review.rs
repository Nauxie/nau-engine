use bevy::prelude::{Vec2, Vec3};

use super::{
    IslandArtDirection, IslandWaterStory, LodBand, PLAYER_STANDING_OFFSET, SkyIsland, SkyRoute,
    authored_island_art_direction, route_edge_waterfall_placement,
};
#[cfg(test)]
use super::{LOD_MID_DISTANCE_M, LOD_NEAR_DISTANCE_M};

pub const ISLAND_REVIEW_VIEWS_PER_ISLAND: usize = 3;
const NEAR_REVIEW_EXTENT_SCALE: f32 = 1.08;
const NEAR_REVIEW_BASE_MIN_DISTANCE_M: f32 = 30.0;
const NEAR_REVIEW_FEATURE_PADDING_M: f32 = 16.0;
const NEAR_REVIEW_PADDED_MIN_DISTANCE_CAP_M: f32 = 40.0;
const NEAR_REVIEW_MAX_DISTANCE_M: f32 = 135.0;
const NEAR_REVIEW_MIN_HEIGHT_M: f32 = 18.0;
const GREAT_PLATEAU_NEAR_REVIEW_MAX_DISTANCE_M: f32 = 180.0;
const MID_REVIEW_EDGE_DISTANCE_M: f32 = 245.0;
const TRAVERSAL_REVIEW_MAX_EDGE_DISTANCE_M: f32 = 170.0;
#[cfg(test)]
const NEAR_REVIEW_MAX_CAMERA_TARGET_DISTANCE_M: f32 = 190.0;
const REVIEW_CAMERA_AZIMUTH_STEPS: usize = 64;
const REVIEW_CAMERA_MIN_FOREIGN_CLEARANCE_M: f32 = 18.0;
const REVIEW_CAMERA_SIGHTLINE_STEPS: usize = 24;
const REVIEW_CAMERA_DISTANCE_SCALES: [f32; 3] = [1.0, 1.12, 1.28];
const REVIEW_CAMERA_HEIGHT_SCALES: [f32; 2] = [1.0, 1.18];
const REVIEW_CAPTURE_ASPECT_RATIO: f32 = 16.0 / 9.0;
const REVIEW_CAPTURE_VERTICAL_FOV_RADIANS: f32 = std::f32::consts::FRAC_PI_4;
const NEAR_REVIEW_AUTHORED_ANCHOR_FRAME_LIMIT: f32 = 0.88;
const WATERFALL_NEAR_REVIEW_FRAME_LIMIT: f32 = 0.70;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IslandReviewView {
    Near,
    Mid,
    Traversal,
}

impl IslandReviewView {
    pub const ALL: [Self; ISLAND_REVIEW_VIEWS_PER_ISLAND] =
        [Self::Near, Self::Mid, Self::Traversal];

    pub fn label(self) -> &'static str {
        match self {
            Self::Near => "near",
            Self::Mid => "mid",
            Self::Traversal => "traversal",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IslandReviewPose {
    pub view: IslandReviewView,
    pub player_position: Vec3,
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub expected_lod: LodBand,
    pub approach_island_name: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IslandReviewSpec {
    pub island_index: usize,
    pub island_name: &'static str,
    pub island_slug: String,
    pub epithet: &'static str,
    pub environmental_story: &'static str,
    pub views: [IslandReviewPose; ISLAND_REVIEW_VIEWS_PER_ISLAND],
}

#[derive(Clone, Debug, PartialEq)]
pub struct IslandReviewPlan {
    pub islands: Vec<IslandReviewSpec>,
}

impl IslandReviewPlan {
    pub fn from_route(route: &SkyRoute) -> Self {
        let islands = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .map(|(island_index, island)| review_spec(route, island_index, island))
            .collect();
        Self { islands }
    }

    pub fn capture_count(&self) -> usize {
        self.islands.len() * ISLAND_REVIEW_VIEWS_PER_ISLAND
    }
}

fn review_spec(route: &SkyRoute, island_index: usize, island: SkyIsland) -> IslandReviewSpec {
    let profile = authored_island_art_direction(island.name)
        .unwrap_or_else(|| panic!("{} must have an authored art direction", island.name));
    let near_focus_height = (island.thickness * 0.16).clamp(2.2, 18.0);
    let focus =
        surface_position(island, Vec2::ZERO) + Vec3::Y * (island.thickness * 0.16).clamp(2.2, 18.0);
    let near_feature_points = authored_near_review_points(island, profile, near_focus_height);
    let near_frame_limit = near_review_frame_limit(island, profile);
    let near_focus = authored_near_focus(&near_feature_points);
    let heading = profile.review_heading_degrees as f32 * std::f32::consts::PI / 180.0;
    let authored_outward = Vec3::new(heading.sin(), 0.0, heading.cos()).normalize_or(Vec3::Z);
    let outward = route_outward_direction(route, island, authored_outward);
    let near_player = surface_position(island, Vec2::ZERO) + Vec3::Y * PLAYER_STANDING_OFFSET;
    let near_distance = island_review_camera_distance(
        island,
        NEAR_REVIEW_EXTENT_SCALE,
        near_review_min_distance(island),
        near_review_max_distance(island),
    );
    let near_height = island_review_camera_height(island, 0.32, NEAR_REVIEW_MIN_HEIGHT_M, 60.0);
    let near_camera = select_review_camera(
        route,
        island,
        ReviewCameraRequest {
            focus: near_focus,
            preferred_outward: outward,
            base_distance: near_distance,
            base_height: near_height,
            min_foreign_clearance_m: REVIEW_CAMERA_MIN_FOREIGN_CLEARANCE_M,
            required_frame_points: &near_feature_points,
            required_frame_limit: near_frame_limit,
            prioritize_clearance: true,
        },
    );

    let mid_player = mid_review_position(
        island,
        outward,
        focus.y + (island.thickness * 0.55).clamp(14.0, 42.0),
    );
    let mid_camera = select_review_camera(
        route,
        island,
        ReviewCameraRequest {
            focus,
            preferred_outward: outward,
            base_distance: island_review_camera_distance(island, 1.95, 38.0, 290.0),
            base_height: island_review_camera_height(island, 0.58, 14.0, 108.0),
            min_foreign_clearance_m: REVIEW_CAMERA_MIN_FOREIGN_CLEARANCE_M,
            required_frame_points: &[],
            required_frame_limit: f32::INFINITY,
            prioritize_clearance: false,
        },
    );

    let (approach_island_name, traversal_player) =
        traversal_player_position(route, island, focus, outward);
    let traversal_outward = (traversal_player - focus).with_y(0.0).normalize_or(outward);
    let traversal_camera_outward = (outward * 0.78 + traversal_outward * 0.22)
        .with_y(0.0)
        .normalize_or(outward);
    let traversal_side = Vec3::new(-traversal_camera_outward.z, 0.0, traversal_camera_outward.x);
    let traversal_distance = island_review_camera_distance(island, 2.15, 44.0, 320.0);
    let traversal_preferred = (traversal_camera_outward + traversal_side * 0.16)
        .with_y(0.0)
        .normalize_or(traversal_camera_outward);
    let traversal_camera = select_review_camera(
        route,
        island,
        ReviewCameraRequest {
            focus,
            preferred_outward: traversal_preferred,
            base_distance: traversal_distance * 1.0127,
            base_height: island_review_camera_height(island, 0.62, 16.0, 116.0),
            min_foreign_clearance_m: REVIEW_CAMERA_MIN_FOREIGN_CLEARANCE_M,
            required_frame_points: &[],
            required_frame_limit: f32::INFINITY,
            prioritize_clearance: false,
        },
    );
    let traversal_target = focus;

    IslandReviewSpec {
        island_index,
        island_name: island.name,
        island_slug: island.name.replace(' ', "_"),
        epithet: profile.epithet,
        environmental_story: profile.environmental_story,
        views: [
            IslandReviewPose {
                view: IslandReviewView::Near,
                player_position: near_player,
                camera_position: near_camera,
                camera_target: near_focus,
                expected_lod: island.lod_band(near_player),
                approach_island_name: None,
            },
            IslandReviewPose {
                view: IslandReviewView::Mid,
                player_position: mid_player,
                camera_position: mid_camera,
                camera_target: focus,
                expected_lod: island.lod_band(mid_player),
                approach_island_name: None,
            },
            IslandReviewPose {
                view: IslandReviewView::Traversal,
                player_position: traversal_player,
                camera_position: traversal_camera,
                camera_target: traversal_target,
                expected_lod: island.lod_band(traversal_player),
                approach_island_name,
            },
        ],
    }
}

fn authored_near_review_points(
    island: SkyIsland,
    profile: IslandArtDirection,
    focus_height: f32,
) -> Vec<Vec3> {
    let mut points = authored_near_feature_anchors(profile)
        .into_iter()
        .flatten()
        .map(|anchor| surface_position(island, anchor) + Vec3::Y * focus_height)
        .collect::<Vec<_>>();

    if profile.water_story == IslandWaterStory::WaterfallGarden && !island.is_great_plateau_anchor()
    {
        let waterfall = route_edge_waterfall_placement(island);
        points.extend([waterfall.ribbon_translation, waterfall.mist_translation]);
    }

    points
}

fn authored_near_focus(points: &[Vec3]) -> Vec3 {
    let mut points = points.iter().copied();
    let first = points
        .next()
        .expect("every authored island should retain a near-review point");
    let mut min = first;
    let mut max = first;

    for point in points {
        min = min.min(point);
        max = max.max(point);
    }

    (min + max) * 0.5
}

fn authored_near_feature_anchors(profile: IslandArtDirection) -> [Option<Vec2>; 4] {
    [
        Some(Vec2::from_array(profile.hero_anchor)),
        (profile.flora_count > 0).then(|| Vec2::from_array(profile.flora_anchor)),
        (profile.formation_count > 0).then(|| Vec2::from_array(profile.formation_anchor)),
        (profile.ruin_count > 0).then(|| Vec2::from_array(profile.ruin_anchor)),
    ]
}

fn near_review_frame_limit(island: SkyIsland, profile: IslandArtDirection) -> f32 {
    if profile.water_story == IslandWaterStory::WaterfallGarden && !island.is_great_plateau_anchor()
    {
        WATERFALL_NEAR_REVIEW_FRAME_LIMIT
    } else {
        NEAR_REVIEW_AUTHORED_ANCHOR_FRAME_LIMIT
    }
}

fn near_review_max_distance(island: SkyIsland) -> f32 {
    if island.is_great_plateau_anchor() {
        GREAT_PLATEAU_NEAR_REVIEW_MAX_DISTANCE_M
    } else {
        NEAR_REVIEW_MAX_DISTANCE_M
    }
}

fn near_review_min_distance(island: SkyIsland) -> f32 {
    (island.half_extents.max_element() + NEAR_REVIEW_FEATURE_PADDING_M).clamp(
        NEAR_REVIEW_BASE_MIN_DISTANCE_M,
        NEAR_REVIEW_PADDED_MIN_DISTANCE_CAP_M,
    )
}

fn route_outward_direction(route: &SkyRoute, island: SkyIsland, authored_outward: Vec3) -> Vec3 {
    let route_center = route
        .islands()
        .iter()
        .fold(Vec3::ZERO, |center, candidate| center + candidate.center)
        / route.islands().len().max(1) as f32;
    let route_outward = (island.center - route_center)
        .with_y(0.0)
        .normalize_or(authored_outward);

    (route_outward * 0.86 + authored_outward * 0.14)
        .with_y(0.0)
        .normalize_or(route_outward)
}

#[derive(Clone, Copy, Debug)]
struct ReviewCameraRequest<'a> {
    focus: Vec3,
    preferred_outward: Vec3,
    base_distance: f32,
    base_height: f32,
    min_foreign_clearance_m: f32,
    required_frame_points: &'a [Vec3],
    required_frame_limit: f32,
    prioritize_clearance: bool,
}

#[derive(Clone, Copy, Debug)]
struct ReviewCameraScore {
    sightline_obstruction_count: usize,
    sightline_penetration_m: f32,
    negative_min_clearance_m: f32,
    heading_deviation_radians: f32,
    distance_scale_index: usize,
    height_scale_index: usize,
    azimuth_index: usize,
    prioritize_clearance: bool,
}

impl ReviewCameraScore {
    fn is_better_than(self, other: Self) -> bool {
        let obstruction_order = self
            .sightline_obstruction_count
            .cmp(&other.sightline_obstruction_count)
            .then_with(|| {
                self.sightline_penetration_m
                    .total_cmp(&other.sightline_penetration_m)
            });
        let order = if self.prioritize_clearance {
            obstruction_order
                .then_with(|| self.distance_scale_index.cmp(&other.distance_scale_index))
                .then_with(|| self.height_scale_index.cmp(&other.height_scale_index))
                .then_with(|| {
                    self.negative_min_clearance_m
                        .total_cmp(&other.negative_min_clearance_m)
                })
                .then_with(|| {
                    self.heading_deviation_radians
                        .total_cmp(&other.heading_deviation_radians)
                })
        } else {
            obstruction_order
                .then_with(|| {
                    self.heading_deviation_radians
                        .total_cmp(&other.heading_deviation_radians)
                })
                .then_with(|| self.distance_scale_index.cmp(&other.distance_scale_index))
                .then_with(|| self.height_scale_index.cmp(&other.height_scale_index))
                .then_with(|| {
                    self.negative_min_clearance_m
                        .total_cmp(&other.negative_min_clearance_m)
                })
        };

        order
            .then_with(|| {
                self.heading_deviation_radians
                    .total_cmp(&other.heading_deviation_radians)
            })
            .then_with(|| self.azimuth_index.cmp(&other.azimuth_index))
            .is_lt()
    }
}

fn select_review_camera(
    route: &SkyRoute,
    target: SkyIsland,
    request: ReviewCameraRequest<'_>,
) -> Vec3 {
    let ReviewCameraRequest {
        focus,
        preferred_outward,
        base_distance,
        base_height,
        min_foreign_clearance_m,
        required_frame_points,
        required_frame_limit,
        prioritize_clearance,
    } = request;
    let preferred = preferred_outward.with_y(0.0).normalize_or(Vec3::Z);
    let preferred_angle = preferred.z.atan2(preferred.x);
    let mut best = None::<(ReviewCameraScore, Vec3)>;

    for (distance_scale_index, distance_scale) in
        REVIEW_CAMERA_DISTANCE_SCALES.into_iter().enumerate()
    {
        for (height_scale_index, height_scale) in
            REVIEW_CAMERA_HEIGHT_SCALES.into_iter().enumerate()
        {
            for azimuth_index in 0..REVIEW_CAMERA_AZIMUTH_STEPS {
                let offset_index = alternating_offset_index(azimuth_index);
                let angle = preferred_angle
                    + offset_index as f32 * std::f32::consts::TAU
                        / REVIEW_CAMERA_AZIMUTH_STEPS as f32;
                let outward = Vec3::new(angle.cos(), 0.0, angle.sin());
                let candidate = focus
                    + outward * base_distance * distance_scale
                    + Vec3::Y * base_height * height_scale;
                let min_clearance = route
                    .islands()
                    .iter()
                    .copied()
                    .filter(|island| island.name != target.name)
                    .map(|island| island.signed_visual_edge_distance(candidate))
                    .fold(f32::INFINITY, f32::min);
                if min_clearance < min_foreign_clearance_m {
                    continue;
                }
                if required_frame_points.iter().any(|point| {
                    normalized_review_screen_position(candidate, focus, *point).is_none_or(
                        |screen| {
                            screen.x.abs() > required_frame_limit
                                || screen.y.abs() > required_frame_limit
                        },
                    )
                }) {
                    continue;
                }

                let (sightline_obstruction_count, sightline_penetration_m) =
                    review_sightline_obstruction(route, target, candidate, focus);
                let score = ReviewCameraScore {
                    sightline_obstruction_count,
                    sightline_penetration_m,
                    negative_min_clearance_m: -min_clearance,
                    heading_deviation_radians: angular_distance(angle, preferred_angle),
                    distance_scale_index,
                    height_scale_index,
                    azimuth_index,
                    prioritize_clearance,
                };
                if best.is_none_or(|(best_score, _)| score.is_better_than(best_score)) {
                    best = Some((score, candidate));
                }
            }
        }
    }

    best.map(|(_, candidate)| candidate).unwrap_or_else(|| {
        let escape_distance = route
            .islands()
            .iter()
            .copied()
            .filter(|island| island.name != target.name)
            .map(|island| {
                target.center.distance(island.center)
                    + island.half_extents.max_element() * 1.34
                    + min_foreign_clearance_m
            })
            .fold(base_distance, f32::max);
        focus + preferred * escape_distance + Vec3::Y * base_height * 1.18
    })
}

fn normalized_review_screen_position(
    camera_position: Vec3,
    camera_target: Vec3,
    point: Vec3,
) -> Option<Vec2> {
    let forward = (camera_target - camera_position).normalize_or(Vec3::Z);
    let right = forward.cross(Vec3::Y).normalize_or(Vec3::X);
    let up = right.cross(forward).normalize_or(Vec3::Y);
    let relative = point - camera_position;
    let depth = relative.dot(forward);
    if depth <= 0.0 {
        return None;
    }
    let half_height = depth * (REVIEW_CAPTURE_VERTICAL_FOV_RADIANS * 0.5).tan();
    let half_width = half_height * REVIEW_CAPTURE_ASPECT_RATIO;
    Some(Vec2::new(
        relative.dot(right) / half_width,
        relative.dot(up) / half_height,
    ))
}

fn alternating_offset_index(index: usize) -> i32 {
    if index == 0 {
        0
    } else if index % 2 == 1 {
        (index as i32 + 1) / 2
    } else {
        -(index as i32 / 2)
    }
}

fn angular_distance(left: f32, right: f32) -> f32 {
    let delta = (left - right).rem_euclid(std::f32::consts::TAU);
    delta.min(std::f32::consts::TAU - delta)
}

fn review_sightline_obstruction(
    route: &SkyRoute,
    target: SkyIsland,
    camera: Vec3,
    focus: Vec3,
) -> (usize, f32) {
    let mut obstruction_count = 0;
    let mut penetration_m = 0.0;

    for step in 1..REVIEW_CAMERA_SIGHTLINE_STEPS {
        let t = step as f32 / REVIEW_CAMERA_SIGHTLINE_STEPS as f32;
        let sample = camera.lerp(focus, t);
        let deepest_penetration = route
            .islands()
            .iter()
            .copied()
            .filter(|island| island.name != target.name)
            .map(|island| -island.signed_visual_edge_distance(sample))
            .fold(0.0_f32, f32::max);
        if deepest_penetration > 0.0 {
            obstruction_count += 1;
            penetration_m += deepest_penetration;
        }
    }

    (obstruction_count, penetration_m)
}

fn island_review_camera_distance(
    island: SkyIsland,
    extent_scale: f32,
    min_distance: f32,
    max_distance: f32,
) -> f32 {
    (island.half_extents.max_element() * extent_scale).clamp(min_distance, max_distance)
}

fn island_review_camera_height(
    island: SkyIsland,
    extent_scale: f32,
    min_height: f32,
    max_height: f32,
) -> f32 {
    (island.half_extents.max_element() * extent_scale + island.thickness * 0.12)
        .clamp(min_height, max_height)
}

fn traversal_player_position(
    route: &SkyRoute,
    island: SkyIsland,
    focus: Vec3,
    fallback_outward: Vec3,
) -> (Option<&'static str>, Vec3) {
    let approach = route
        .island_composition(island.name)
        .into_iter()
        .flat_map(|composition| composition.neighbor_islands.iter().copied())
        .filter_map(|name| route.island_named(name))
        .min_by(|left, right| {
            left.horizontal_distance(island.center)
                .total_cmp(&right.horizontal_distance(island.center))
        });
    let Some(approach) = approach else {
        return (
            None,
            position_outside_visual_edge(
                island,
                fallback_outward,
                TRAVERSAL_REVIEW_MAX_EDGE_DISTANCE_M,
                focus.y + 12.0,
            ),
        );
    };

    let approach_surface =
        surface_position(approach, Vec2::ZERO) + Vec3::Y * PLAYER_STANDING_OFFSET;
    let edge_distance = island.visual_edge_distance(approach_surface);
    if edge_distance <= TRAVERSAL_REVIEW_MAX_EDGE_DISTANCE_M
        && island.stream_activation(approach_surface).is_active()
    {
        return (Some(approach.name), approach_surface);
    }

    let direction = (approach.center - island.center)
        .with_y(0.0)
        .normalize_or(fallback_outward);
    (
        Some(approach.name),
        position_outside_visual_edge(
            island,
            direction,
            TRAVERSAL_REVIEW_MAX_EDGE_DISTANCE_M,
            focus.y + 10.0,
        ),
    )
}

fn mid_review_position(island: SkyIsland, preferred_outward: Vec3, y: f32) -> Vec3 {
    let preferred = preferred_outward.with_y(0.0).normalize_or(Vec3::Z);
    for distance_step in 0..=5 {
        let edge_distance = MID_REVIEW_EDGE_DISTANCE_M - distance_step as f32 * 10.0;
        for angle_step in 0_i32..32 {
            let signed_step = if angle_step == 0 {
                0
            } else if angle_step % 2 == 0 {
                angle_step / 2
            } else {
                -(angle_step + 1) / 2
            };
            let angle = signed_step as f32 * std::f32::consts::TAU / 32.0;
            let outward = Vec3::new(
                preferred.x * angle.cos() - preferred.z * angle.sin(),
                0.0,
                preferred.x * angle.sin() + preferred.z * angle.cos(),
            );
            let candidate = position_outside_visual_edge(island, outward, edge_distance, y);
            if island.lod_band(candidate) == LodBand::Mid
                && island.stream_activation(candidate).is_active()
            {
                return candidate;
            }
        }
    }

    panic!(
        "{} has no stream-active mid-LOD review position",
        island.name
    );
}

fn position_outside_visual_edge(
    island: SkyIsland,
    outward: Vec3,
    edge_distance_m: f32,
    y: f32,
) -> Vec3 {
    let direction = outward.with_y(0.0).normalize_or(Vec3::Z);
    let angle = direction.z.atan2(direction.x);
    let edge = island.footprint_contour_point(angle, true);
    Vec3::new(edge.x, y, edge.y) + direction * edge_distance_m
}

fn surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let position = island.center
        + Vec3::new(
            normalized_offset.x * island.half_extents.x,
            0.0,
            normalized_offset.y * island.half_extents.y,
        );
    Vec3::new(
        position.x,
        island.terrain_surface_y_at(position),
        position.z,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn review_plan_covers_all_islands_and_three_distinct_views() {
        let route = SkyRoute::default();
        let plan = IslandReviewPlan::from_route(&route);
        assert_eq!(plan.islands.len(), route.islands().len());
        assert_eq!(
            plan.capture_count(),
            route.islands().len() * ISLAND_REVIEW_VIEWS_PER_ISLAND
        );

        let mut names = HashSet::new();
        let mut slugs = HashSet::new();
        for (island, review) in route.islands().iter().zip(&plan.islands) {
            assert_eq!(review.island_name, island.name);
            assert!(names.insert(review.island_name));
            assert!(slugs.insert(review.island_slug.as_str()));
            assert_eq!(review.views.map(|view| view.view), IslandReviewView::ALL);
            assert_ne!(
                review.views[0].camera_position,
                review.views[1].camera_position
            );
            assert_ne!(
                review.views[1].camera_position,
                review.views[2].camera_position
            );
        }
    }

    #[test]
    fn review_poses_are_finite_visible_and_match_expected_lod() {
        let route = SkyRoute::default();
        let plan = IslandReviewPlan::from_route(&route);

        for review in &plan.islands {
            let island = route
                .island_named(review.island_name)
                .expect("review target should exist");
            for pose in review.views {
                assert!(pose.player_position.is_finite());
                assert!(pose.camera_position.is_finite());
                assert!(pose.camera_target.is_finite());
                assert!(pose.camera_position.distance(pose.camera_target) > 8.0);
                assert_eq!(island.lod_band(pose.player_position), pose.expected_lod);
                assert!(
                    island.stream_activation(pose.player_position).is_active(),
                    "{} {} view should keep the target stream-active",
                    review.island_name,
                    pose.view.label()
                );
            }
            assert_eq!(review.views[0].expected_lod, LodBand::Near);
            assert_eq!(review.views[1].expected_lod, LodBand::Mid);
            let near = review.views[0];
            assert!(
                near.camera_position.distance(near.camera_target)
                    <= NEAR_REVIEW_MAX_CAMERA_TARGET_DISTANCE_M,
                "{} near view should frame a player-scale authored district",
                review.island_name
            );
            assert!(
                near.camera_position.distance(near.camera_target)
                    < review.views[1]
                        .camera_position
                        .distance(review.views[1].camera_target),
                "{} near view should be closer than its mid view",
                review.island_name
            );
            let profile = authored_island_art_direction(review.island_name)
                .expect("review target should have an art direction");
            let focus_height = (island.thickness * 0.16).clamp(2.2, 18.0);
            let expected_focus =
                authored_near_focus(&authored_near_review_points(island, profile, focus_height));
            assert!(
                near.camera_target.distance(expected_focus) < 0.001,
                "{} near view should target the center of its authored feature district",
                review.island_name
            );
            assert!(
                island.visual_edge_distance(review.views[1].player_position) > LOD_NEAR_DISTANCE_M
            );
            assert!(
                island.visual_edge_distance(review.views[1].player_position) <= LOD_MID_DISTANCE_M
            );
        }
    }

    #[test]
    fn near_reviews_keep_every_required_authored_district_anchor_in_frame() {
        let route = SkyRoute::default();
        let plan = IslandReviewPlan::from_route(&route);

        for review in &plan.islands {
            let island = route
                .island_named(review.island_name)
                .expect("review target should exist");
            let profile = authored_island_art_direction(review.island_name)
                .expect("review target should have an art direction");
            let near = review.views[0];
            let focus_height = (island.thickness * 0.16).clamp(2.2, 18.0);
            let frame_limit = near_review_frame_limit(island, profile);

            for point in authored_near_review_points(island, profile, focus_height) {
                let screen = normalized_review_screen_position(
                    near.camera_position,
                    near.camera_target,
                    point,
                )
                .expect("required near-review anchor should remain in front of the camera");
                assert!(
                    screen.x.abs() <= frame_limit && screen.y.abs() <= frame_limit,
                    "{} required near-review point {point:?} projects outside the frame at {screen:?}",
                    review.island_name
                );
            }
        }
    }

    #[test]
    fn cloudfall_near_review_frames_the_rendered_waterfall_evidence() {
        let route = SkyRoute::default();
        let plan = IslandReviewPlan::from_route(&route);
        let island = route
            .island_named("cloudfall meadow")
            .expect("cloudfall meadow should exist");
        let review = plan
            .islands
            .iter()
            .find(|review| review.island_name == island.name)
            .expect("cloudfall meadow should have a review");
        let near = review.views[0];
        let waterfall = route_edge_waterfall_placement(island);

        for point in [waterfall.ribbon_translation, waterfall.mist_translation] {
            let screen =
                normalized_review_screen_position(near.camera_position, near.camera_target, point)
                    .expect("waterfall evidence should remain in front of the near camera");
            assert!(
                screen.x.abs() <= WATERFALL_NEAR_REVIEW_FRAME_LIMIT
                    && screen.y.abs() <= WATERFALL_NEAR_REVIEW_FRAME_LIMIT,
                "cloudfall waterfall evidence {point:?} projects outside the frame at {screen:?}"
            );
        }
    }

    #[test]
    fn traversal_views_use_authored_neighbors() {
        let route = SkyRoute::default();
        let plan = IslandReviewPlan::from_route(&route);

        for review in &plan.islands {
            let approach = review.views[2]
                .approach_island_name
                .expect("every authored island should have an approach neighbor");
            let composition = route
                .island_composition(review.island_name)
                .expect("review target should have a composition");
            assert!(
                composition.neighbor_islands.contains(&approach),
                "{} traversal view should use an authored neighbor, got {approach}",
                review.island_name
            );
        }
    }

    #[test]
    fn review_cameras_clear_all_non_target_visual_footprints() {
        let route = SkyRoute::default();
        let plan = IslandReviewPlan::from_route(&route);

        for review in &plan.islands {
            for pose in review.views {
                for other in route
                    .islands()
                    .iter()
                    .copied()
                    .filter(|other| other.name != review.island_name)
                {
                    assert!(
                        other.signed_visual_edge_distance(pose.camera_position)
                            >= REVIEW_CAMERA_MIN_FOREIGN_CLEARANCE_M,
                        "{} {} camera overlaps {} by {:.2} m",
                        review.island_name,
                        pose.view.label(),
                        other.name,
                        other.signed_visual_edge_distance(pose.camera_position)
                    );
                }
            }
        }
    }

    #[test]
    fn camera_selection_is_deterministic() {
        let route = SkyRoute::default();
        assert_eq!(
            IslandReviewPlan::from_route(&route),
            IslandReviewPlan::from_route(&route)
        );
    }

    #[test]
    fn sapphire_basin_views_clear_broken_stair() {
        let route = SkyRoute::default();
        let plan = IslandReviewPlan::from_route(&route);
        let review = plan
            .islands
            .iter()
            .find(|review| review.island_name == "sapphire basin")
            .expect("sapphire basin review should exist");
        let broken_stair = route
            .island_named("broken stair")
            .expect("broken stair should exist");

        for pose in review.views {
            assert!(
                broken_stair.signed_visual_edge_distance(pose.camera_position)
                    >= REVIEW_CAMERA_MIN_FOREIGN_CLEARANCE_M,
                "{} camera should clear broken stair",
                pose.view.label()
            );
        }
    }
}
