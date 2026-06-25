use crate::movement::smoothing_factor;
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component, Clone, Copy, Debug)]
pub struct FollowCamera {
    pub distance: f32,
    pub height: f32,
    pub look_height: f32,
    pub look_ahead: f32,
    pub position_smoothing: f32,
    pub rotation_smoothing: f32,
    pub direction_smoothing: f32,
    pub min_height: f32,
}

impl Default for FollowCamera {
    fn default() -> Self {
        Self {
            distance: 12.0,
            height: 5.0,
            look_height: 1.4,
            look_ahead: 0.5,
            position_smoothing: 10.0,
            rotation_smoothing: 24.0,
            direction_smoothing: 1.0,
            min_height: 1.6,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct FollowCameraState {
    pub direction: Vec3,
    initialized: bool,
}

impl Default for FollowCameraState {
    fn default() -> Self {
        Self {
            direction: Vec3::NEG_Z,
            initialized: false,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct CameraControlTuning {
    pub sensitivity_x: f32,
    pub sensitivity_y: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub invert_y: bool,
}

impl Default for CameraControlTuning {
    fn default() -> Self {
        Self {
            sensitivity_x: 0.0042,
            sensitivity_y: 0.0036,
            min_pitch: -35.0_f32.to_radians(),
            max_pitch: 35.0_f32.to_radians(),
            invert_y: false,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct CameraControlState {
    pub orbit: CameraOrbit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraInput {
    pub mouse_delta: Vec2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraOrbit {
    pub yaw: f32,
    pub pitch: f32,
}

impl CameraOrbit {
    pub fn yaw_degrees(self) -> f32 {
        self.yaw.to_degrees()
    }

    pub fn pitch_degrees(self) -> f32 {
        self.pitch.to_degrees()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CameraFrame {
    pub position: Vec3,
    pub rotation: Quat,
    pub look_target: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraObstruction {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl CameraObstruction {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            center,
            half_extents: half_extents.abs(),
        }
    }

    fn expanded(self, clearance: f32) -> Self {
        Self {
            center: self.center,
            half_extents: self.half_extents + Vec3::splat(clearance.max(0.0)),
        }
    }

    fn contains(self, point: Vec3) -> bool {
        let min = self.center - self.half_extents;
        let max = self.center + self.half_extents;

        point.x >= min.x
            && point.x <= max.x
            && point.y >= min.y
            && point.y <= max.y
            && point.z >= min.z
            && point.z <= max.z
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CameraObstructionResolution {
    pub frame: CameraFrame,
    pub adjusted_distance_m: f32,
    pub hit_count: usize,
}

pub fn step_camera(
    current_position: Vec3,
    current_rotation: Quat,
    player_position: Vec3,
    player_forward: Vec3,
    player_velocity: Vec3,
    follow: &FollowCamera,
    dt: f32,
) -> CameraFrame {
    step_camera_with_orbit(
        current_position,
        current_rotation,
        player_position,
        player_forward,
        player_velocity,
        follow,
        CameraOrbit::default(),
        dt,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn step_camera_with_orbit(
    current_position: Vec3,
    current_rotation: Quat,
    player_position: Vec3,
    player_forward: Vec3,
    player_velocity: Vec3,
    follow: &FollowCamera,
    orbit: CameraOrbit,
    dt: f32,
) -> CameraFrame {
    let direction = horizontal_follow_direction(player_velocity, player_forward);
    step_camera_with_direction(
        current_position,
        current_rotation,
        player_position,
        direction,
        follow,
        orbit,
        dt,
    )
}

pub fn update_follow_direction_state(
    state: &mut FollowCameraState,
    desired_direction: Vec3,
    follow: &FollowCamera,
    dt: f32,
) -> Vec3 {
    let fallback = if state.initialized {
        state.direction
    } else {
        Vec3::NEG_Z
    };
    let desired_direction = horizontal_or(desired_direction, fallback);
    if !state.initialized {
        state.direction = desired_direction;
        state.initialized = true;
        return state.direction;
    }

    state.direction = horizontal_or(
        state.direction.lerp(
            desired_direction,
            smoothing_factor(follow.direction_smoothing, dt),
        ),
        desired_direction,
    );
    state.direction
}

pub fn movement_stable_follow_direction(
    velocity: Vec3,
    player_forward: Vec3,
    current_follow_direction: Vec3,
) -> Vec3 {
    const MIN_FOLLOW_SPEED_SQUARED: f32 = 1.0;
    const MIN_FORWARD_FOLLOW_DOT: f32 = 0.99;

    let current_direction = horizontal_or(current_follow_direction, Vec3::NEG_Z);
    let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    if horizontal_velocity.length_squared() > MIN_FOLLOW_SPEED_SQUARED {
        let velocity_direction = horizontal_velocity.normalize();
        if velocity_direction.dot(current_direction) >= MIN_FORWARD_FOLLOW_DOT {
            return velocity_direction;
        }
        return current_direction;
    }

    let forward_direction = horizontal_or(player_forward, current_direction);
    if forward_direction.dot(current_direction) >= MIN_FORWARD_FOLLOW_DOT {
        forward_direction
    } else {
        current_direction
    }
}

pub fn movement_input_stable_follow_direction(
    velocity: Vec3,
    player_forward: Vec3,
    current_follow_direction: Vec3,
    movement_axis: Vec2,
) -> Vec3 {
    let current_direction = horizontal_or(current_follow_direction, Vec3::NEG_Z);
    let forward_only = movement_axis.y > 0.0 && movement_axis.x.abs() <= f32::EPSILON;
    if forward_only {
        movement_stable_follow_direction(velocity, player_forward, current_direction)
    } else {
        current_direction
    }
}

#[allow(clippy::too_many_arguments)]
pub fn step_camera_with_direction(
    current_position: Vec3,
    current_rotation: Quat,
    player_position: Vec3,
    follow_direction: Vec3,
    follow: &FollowCamera,
    orbit: CameraOrbit,
    dt: f32,
) -> CameraFrame {
    let direction = horizontal_or(follow_direction, Vec3::NEG_Z);
    let direction = yawed_horizontal_direction(direction, orbit.yaw);
    let look_target =
        player_position + Vec3::Y * follow.look_height + direction * follow.look_ahead;
    let base_horizontal_distance = follow.distance + follow.look_ahead;
    let base_vertical_offset = follow.height - follow.look_height;
    let boom_distance = Vec2::new(base_horizontal_distance, base_vertical_offset)
        .length()
        .max(0.001);
    let base_elevation = base_vertical_offset.atan2(base_horizontal_distance);
    let elevation = base_elevation - orbit.pitch;
    let horizontal_distance = elevation.cos().max(0.0) * boom_distance;
    let vertical_offset = elevation.sin() * boom_distance;
    let mut desired_position =
        look_target - direction * horizontal_distance + Vec3::Y * vertical_offset;
    desired_position.y = desired_position.y.max(follow.min_height);

    let mut position = current_position.lerp(
        desired_position,
        smoothing_factor(follow.position_smoothing, dt),
    );
    let lateral_axis = direction.cross(Vec3::Y).normalize_or_zero();
    if lateral_axis.length_squared() > 0.0001 {
        position += lateral_axis * (desired_position - position).dot(lateral_axis);
    }
    let target_rotation = Transform::from_translation(position)
        .looking_at(look_target, Vec3::Y)
        .rotation;
    let rotation = current_rotation.slerp(
        target_rotation,
        smoothing_factor(follow.rotation_smoothing, dt),
    );

    CameraFrame {
        position,
        rotation,
        look_target,
    }
}

pub fn apply_camera_input(
    orbit: CameraOrbit,
    input: CameraInput,
    tuning: &CameraControlTuning,
) -> CameraOrbit {
    let yaw = wrap_radians(orbit.yaw - input.mouse_delta.x * tuning.sensitivity_x);
    let y_sign = if tuning.invert_y { 1.0 } else { -1.0 };
    let pitch = (orbit.pitch + input.mouse_delta.y * tuning.sensitivity_y * y_sign)
        .clamp(tuning.min_pitch, tuning.max_pitch);

    CameraOrbit { yaw, pitch }
}

fn yawed_horizontal_direction(direction: Vec3, yaw: f32) -> Vec3 {
    let rotated = Quat::from_rotation_y(yaw) * direction;
    horizontal_or(rotated, direction)
}

pub fn horizontal_follow_direction(velocity: Vec3, player_forward: Vec3) -> Vec3 {
    let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    if horizontal_velocity.length_squared() > 1.0 {
        horizontal_velocity.normalize()
    } else {
        let horizontal_forward = Vec3::new(player_forward.x, 0.0, player_forward.z);
        if horizontal_forward.length_squared() > 0.0001 {
            horizontal_forward.normalize()
        } else {
            Vec3::Z
        }
    }
}

fn horizontal_or(value: Vec3, fallback: Vec3) -> Vec3 {
    let horizontal = Vec3::new(value.x, 0.0, value.z);
    if horizontal.length_squared() > 0.0001 {
        horizontal.normalize()
    } else {
        let fallback = Vec3::new(fallback.x, 0.0, fallback.z);
        if fallback.length_squared() > 0.0001 {
            fallback.normalize()
        } else {
            Vec3::NEG_Z
        }
    }
}

fn wrap_radians(value: f32) -> f32 {
    (value + PI).rem_euclid(PI * 2.0) - PI
}

pub fn camera_distance(camera_position: Vec3, target_position: Vec3) -> f32 {
    let distance = camera_position.distance(target_position);
    if distance.is_finite() { distance } else { 0.0 }
}

pub fn camera_surface_clearance(camera_position: Vec3, floor_y: f32) -> f32 {
    (camera_position.y - floor_y).max(0.0)
}

pub fn camera_target_angle_degrees(
    camera_position: Vec3,
    camera_rotation: Quat,
    target_position: Vec3,
) -> f32 {
    let to_target = target_position - camera_position;
    if to_target.length_squared() <= 0.0001 {
        return 0.0;
    }

    let forward = camera_rotation * Vec3::NEG_Z;
    let dot = forward
        .normalize_or_zero()
        .dot(to_target.normalize())
        .clamp(-1.0, 1.0);
    if dot.is_finite() {
        dot.acos().to_degrees()
    } else {
        0.0
    }
}

pub fn camera_orbit_alignment_degrees(
    camera_position: Vec3,
    look_target: Vec3,
    follow_direction: Vec3,
    orbit: CameraOrbit,
) -> f32 {
    let expected_direction = yawed_horizontal_direction(follow_direction, orbit.yaw);
    let actual_direction = horizontal_or(look_target - camera_position, expected_direction);
    let angle = actual_direction
        .angle_between(expected_direction)
        .to_degrees();

    if angle.is_finite() { angle } else { 0.0 }
}

pub fn camera_view_yaw_degrees(camera_rotation: Quat, reference_direction: Vec3) -> f32 {
    let reference_direction = horizontal_or(reference_direction, Vec3::NEG_Z);
    let view_direction = horizontal_or(camera_rotation * Vec3::NEG_Z, reference_direction);
    let cross_y = reference_direction.cross(view_direction).y;
    let dot = reference_direction.dot(view_direction).clamp(-1.0, 1.0);
    let yaw = cross_y.atan2(dot).to_degrees();

    if yaw.is_finite() { yaw } else { 0.0 }
}

pub fn lift_camera_above_floor(
    mut frame: CameraFrame,
    floor_y: f32,
    min_clearance: f32,
) -> CameraFrame {
    let min_y = floor_y + min_clearance.max(0.0);
    if frame.position.y < min_y {
        frame.position.y = min_y;
        frame.rotation = Transform::from_translation(frame.position)
            .looking_at(frame.look_target, Vec3::Y)
            .rotation;
    }

    frame
}

pub fn avoid_camera_obstructions(
    frame: CameraFrame,
    obstructions: impl IntoIterator<Item = CameraObstruction>,
    clearance: f32,
) -> CameraObstructionResolution {
    let segment = frame.position - frame.look_target;
    let segment_length = segment.length();
    if segment_length <= 0.001 || !segment_length.is_finite() {
        return CameraObstructionResolution {
            frame,
            adjusted_distance_m: 0.0,
            hit_count: 0,
        };
    }

    let direction = segment / segment_length;
    let mut nearest_hit_distance = segment_length;
    let mut hit_count = 0;

    for obstruction in obstructions {
        let obstruction = obstruction.expanded(clearance);
        if obstruction.contains(frame.look_target) {
            continue;
        }
        let Some(hit_distance) =
            segment_aabb_hit_distance(frame.look_target, direction, segment_length, obstruction)
        else {
            continue;
        };
        hit_count += 1;
        nearest_hit_distance = nearest_hit_distance.min(hit_distance);
    }

    if hit_count == 0 || nearest_hit_distance >= segment_length {
        return CameraObstructionResolution {
            frame,
            adjusted_distance_m: 0.0,
            hit_count,
        };
    }

    let min_target_distance = 2.4;
    let adjusted_distance = nearest_hit_distance.max(min_target_distance);
    let mut adjusted = frame;
    adjusted.position = frame.look_target + direction * adjusted_distance;
    adjusted.rotation = Transform::from_translation(adjusted.position)
        .looking_at(adjusted.look_target, Vec3::Y)
        .rotation;

    CameraObstructionResolution {
        frame: adjusted,
        adjusted_distance_m: frame.position.distance(adjusted.position),
        hit_count,
    }
}

fn segment_aabb_hit_distance(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    obstruction: CameraObstruction,
) -> Option<f32> {
    let min = obstruction.center - obstruction.half_extents;
    let max = obstruction.center + obstruction.half_extents;
    let mut t_min = 0.0;
    let mut t_max = max_distance;

    update_slab_interval(origin.x, direction.x, min.x, max.x, &mut t_min, &mut t_max)?;
    update_slab_interval(origin.y, direction.y, min.y, max.y, &mut t_min, &mut t_max)?;
    update_slab_interval(origin.z, direction.z, min.z, max.z, &mut t_min, &mut t_max)?;

    if t_min <= max_distance && t_max >= 0.0 {
        Some(t_min.max(0.0))
    } else {
        None
    }
}

fn update_slab_interval(
    origin: f32,
    direction: f32,
    min: f32,
    max: f32,
    t_min: &mut f32,
    t_max: &mut f32,
) -> Option<()> {
    if direction.abs() <= 0.0001 {
        return (origin >= min && origin <= max).then_some(());
    }

    let inverse_direction = direction.recip();
    let mut near = (min - origin) * inverse_direction;
    let mut far = (max - origin) * inverse_direction;
    if near > far {
        std::mem::swap(&mut near, &mut far);
    }

    *t_min = (*t_min).max(near);
    *t_max = (*t_max).min(far);
    (*t_min <= *t_max).then_some(())
}

pub fn camera_pitch_degrees(rotation: Quat) -> f32 {
    let forward = rotation * Vec3::NEG_Z;
    let y = forward.y.clamp(-1.0, 1.0);

    if y.is_finite() {
        y.asin().to_degrees()
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertical_launch_velocity_does_not_pull_camera_under_player() {
        let follow = FollowCamera::default();
        let frame = step_camera(
            Vec3::new(0.0, 6.0, -12.0),
            Quat::IDENTITY,
            Vec3::new(0.0, 20.0, 0.0),
            Vec3::Z,
            Vec3::new(0.0, 40.0, 0.0),
            &follow,
            1.0,
        );

        assert!(frame.position.y > 20.0);
        assert!(frame.position.z < 0.0);
    }

    #[test]
    fn horizontal_velocity_controls_follow_direction() {
        let direction = horizontal_follow_direction(Vec3::new(10.0, 40.0, 0.0), Vec3::Z);
        assert!(direction.x > 0.99);
        assert!(direction.y.abs() < 0.001);
    }

    #[test]
    fn lateral_velocity_does_not_drag_stable_follow_direction() {
        let direction =
            movement_stable_follow_direction(Vec3::new(20.0, 0.0, 0.0), Vec3::X, Vec3::NEG_Z);

        assert!(direction.distance(Vec3::NEG_Z) < 0.001);
    }

    #[test]
    fn mostly_forward_velocity_can_adjust_stable_follow_direction() {
        let velocity = Vec3::new(1.0, 0.0, -20.0);
        let direction = movement_stable_follow_direction(velocity, Vec3::NEG_Z, Vec3::NEG_Z);

        assert!(direction.distance(velocity.normalize()) < 0.001);
    }

    #[test]
    fn lateral_or_released_input_does_not_drag_camera_follow_direction() {
        let velocity = Vec3::new(1.0, 0.0, -20.0);
        let lateral_direction = movement_input_stable_follow_direction(
            velocity,
            Vec3::X,
            Vec3::NEG_Z,
            Vec2::new(1.0, 0.0),
        );
        let released_direction =
            movement_input_stable_follow_direction(velocity, Vec3::X, Vec3::NEG_Z, Vec2::ZERO);
        let forward_direction =
            movement_input_stable_follow_direction(velocity, Vec3::NEG_Z, Vec3::NEG_Z, Vec2::Y);

        assert!(lateral_direction.distance(Vec3::NEG_Z) < 0.001);
        assert!(released_direction.distance(Vec3::NEG_Z) < 0.001);
        assert!(forward_direction.distance(velocity.normalize()) < 0.001);
    }

    #[test]
    fn lateral_camera_tracking_translates_without_yawing_view() {
        let follow = FollowCamera::default();
        let player_position = Vec3::new(8.0, 30.0, 0.0);
        let frame = step_camera_with_direction(
            Vec3::new(0.0, 34.0, 12.0),
            Quat::IDENTITY,
            player_position,
            Vec3::NEG_Z,
            &follow,
            CameraOrbit::default(),
            1.0 / 60.0,
        );

        assert!(
            (frame.position.x - player_position.x).abs() < 0.001,
            "camera should translate laterally with the player instead of lagging into a yaw"
        );
        assert!(camera_view_yaw_degrees(frame.rotation, Vec3::NEG_Z).abs() < 0.001);
    }

    #[test]
    fn mouse_x_changes_camera_yaw_without_touching_pitch() {
        let tuning = CameraControlTuning::default();
        let orbit = apply_camera_input(
            CameraOrbit::default(),
            CameraInput {
                mouse_delta: Vec2::new(20.0, 0.0),
            },
            &tuning,
        );

        assert!(orbit.yaw < -0.08);
        assert_eq!(orbit.pitch, 0.0);
    }

    #[test]
    fn mouse_y_maps_to_pitch_and_clamps() {
        let tuning = CameraControlTuning::default();
        let up = apply_camera_input(
            CameraOrbit::default(),
            CameraInput {
                mouse_delta: Vec2::new(0.0, -20.0),
            },
            &tuning,
        );
        let clamped = apply_camera_input(
            CameraOrbit::default(),
            CameraInput {
                mouse_delta: Vec2::new(0.0, -1000.0),
            },
            &tuning,
        );

        assert!(up.pitch > 0.07);
        assert_eq!(clamped.pitch, tuning.max_pitch);
    }

    #[test]
    fn orbit_pitch_moves_view_pitch_in_expected_direction() {
        let follow = FollowCamera::default();
        let low = step_camera_with_orbit(
            Vec3::new(0.0, 6.0, -12.0),
            Quat::IDENTITY,
            Vec3::ZERO,
            Vec3::NEG_Z,
            Vec3::NEG_Z * 10.0,
            &follow,
            CameraOrbit {
                pitch: -0.25,
                yaw: 0.0,
            },
            1.0,
        );
        let high = step_camera_with_orbit(
            Vec3::new(0.0, 6.0, -12.0),
            Quat::IDENTITY,
            Vec3::ZERO,
            Vec3::NEG_Z,
            Vec3::NEG_Z * 10.0,
            &follow,
            CameraOrbit {
                pitch: 0.25,
                yaw: 0.0,
            },
            1.0,
        );

        assert!(camera_pitch_degrees(high.rotation) > camera_pitch_degrees(low.rotation));
    }

    #[test]
    fn orbit_pitch_keeps_player_focus_centered() {
        let follow = FollowCamera::default();
        let player_position = Vec3::ZERO;
        let frame = step_camera_with_orbit(
            Vec3::new(0.0, follow.height, follow.distance),
            Quat::IDENTITY,
            player_position,
            Vec3::NEG_Z,
            Vec3::ZERO,
            &follow,
            CameraOrbit {
                pitch: CameraControlTuning::default().max_pitch,
                yaw: 0.0,
            },
            1.0,
        );
        let player_focus = player_position + Vec3::Y * follow.look_height;

        assert!(camera_target_angle_degrees(frame.position, frame.rotation, player_focus) < 3.0);
    }

    #[test]
    fn follow_direction_smoothing_limits_turnaround_snap() {
        let follow = FollowCamera::default();
        let mut state = FollowCameraState {
            direction: Vec3::Z,
            initialized: true,
        };
        let follow_direction =
            update_follow_direction_state(&mut state, Vec3::NEG_Z, &follow, 1.0 / 60.0);
        let frame = step_camera_with_direction(
            Vec3::new(0.0, 6.0, 12.0),
            Quat::IDENTITY,
            Vec3::ZERO,
            follow_direction,
            &follow,
            CameraOrbit::default(),
            1.0 / 60.0,
        );

        assert!(
            frame.position.z > 8.0,
            "camera should not instantly orbit across the player on a velocity flip"
        );
    }

    #[test]
    fn persistent_yaw_offset_does_not_compound_into_spin() {
        let follow = FollowCamera::default();
        let orbit = CameraOrbit {
            yaw: 0.2,
            pitch: 0.0,
        };
        let player_position = Vec3::ZERO;
        let player_forward = Vec3::NEG_Z;
        let mut camera_position = Vec3::new(0.0, follow.height, follow.distance);
        let mut camera_rotation = Transform::from_translation(camera_position)
            .looking_at(player_position + Vec3::Y * follow.look_height, Vec3::Y)
            .rotation;
        let expected_direction = yawed_horizontal_direction(
            horizontal_follow_direction(Vec3::ZERO, player_forward),
            orbit.yaw,
        );

        for _ in 0..240 {
            let frame = step_camera_with_orbit(
                camera_position,
                camera_rotation,
                player_position,
                player_forward,
                Vec3::ZERO,
                &follow,
                orbit,
                1.0 / 60.0,
            );
            camera_position = frame.position;
            camera_rotation = frame.rotation;
        }

        let drift_degrees = camera_orbit_alignment_degrees(
            camera_position,
            player_position + Vec3::Y * follow.look_height,
            expected_direction,
            CameraOrbit::default(),
        );

        assert!(
            drift_degrees < 3.0,
            "persistent yaw drifted by {drift_degrees} degrees"
        );
    }

    #[test]
    fn camera_pitch_is_negative_when_looking_downward() {
        let rotation = Transform::from_xyz(0.0, 6.0, -12.0)
            .looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y)
            .rotation;

        assert!(camera_pitch_degrees(rotation) < -15.0);
    }

    #[test]
    fn camera_pitch_is_level_for_horizontal_forward() {
        assert!(camera_pitch_degrees(Quat::IDENTITY).abs() < 0.001);
    }

    #[test]
    fn camera_view_yaw_tracks_horizontal_rotation() {
        let yaw_radians = 0.35_f32;
        let yaw_degrees = camera_view_yaw_degrees(Quat::from_rotation_y(yaw_radians), Vec3::NEG_Z);

        assert!((yaw_degrees.abs() - yaw_radians.to_degrees()).abs() < 0.001);
    }

    #[test]
    fn camera_distance_matches_vector_length() {
        assert_eq!(camera_distance(Vec3::new(0.0, 3.0, 4.0), Vec3::ZERO), 5.0);
    }

    #[test]
    fn camera_surface_clearance_lifts_clipping_frame() {
        let frame = CameraFrame {
            position: Vec3::new(0.0, 3.0, 0.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 4.0, -4.0),
        };

        let lifted = lift_camera_above_floor(frame, 2.5, 2.0);

        assert_eq!(lifted.position.y, 4.5);
        assert_eq!(camera_surface_clearance(lifted.position, 2.5), 2.0);
    }

    #[test]
    fn camera_obstruction_moves_camera_in_front_of_blocker() {
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let resolved = avoid_camera_obstructions(
            frame,
            [CameraObstruction::new(
                Vec3::new(0.0, 2.0, 5.0),
                Vec3::new(1.0, 1.0, 1.0),
            )],
            0.5,
        );

        assert_eq!(resolved.hit_count, 1);
        assert!(resolved.adjusted_distance_m > 5.0);
        assert!(resolved.frame.position.z < 4.0);
        assert!(
            camera_target_angle_degrees(
                resolved.frame.position,
                resolved.frame.rotation,
                resolved.frame.look_target,
            ) < 0.001
        );
    }

    #[test]
    fn camera_obstruction_keeps_clear_view_when_blocker_is_off_segment() {
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 10.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let resolved = avoid_camera_obstructions(
            frame,
            [CameraObstruction::new(
                Vec3::new(5.0, 2.0, 5.0),
                Vec3::new(1.0, 1.0, 1.0),
            )],
            0.5,
        );

        assert_eq!(resolved.hit_count, 0);
        assert_eq!(resolved.adjusted_distance_m, 0.0);
        assert_eq!(resolved.frame.position, frame.position);
    }

    #[test]
    fn camera_obstruction_uses_nearest_blocker() {
        let frame = CameraFrame {
            position: Vec3::new(0.0, 2.0, 12.0),
            rotation: Quat::IDENTITY,
            look_target: Vec3::new(0.0, 2.0, 0.0),
        };

        let resolved = avoid_camera_obstructions(
            frame,
            [
                CameraObstruction::new(Vec3::new(0.0, 2.0, 8.0), Vec3::splat(1.0)),
                CameraObstruction::new(Vec3::new(0.0, 2.0, 4.0), Vec3::splat(1.0)),
            ],
            0.25,
        );

        assert_eq!(resolved.hit_count, 2);
        assert!(resolved.frame.position.z < 3.0);
        assert!(resolved.frame.position.z > 2.3);
    }
}
