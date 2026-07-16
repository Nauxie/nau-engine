use bevy::prelude::*;

use super::super::{
    CameraControlState, CameraControlTuning, CameraInput, CameraObstruction,
    CameraObstructionHandoffState, FollowCamera, camera_pitch_degrees, camera_view_yaw_degrees,
    resolve_camera_obstruction_handoff, step_camera_control, step_camera_with_direction_and_input,
};

const FRAME_RATES_HZ: [f32; 4] = [30.0, 60.0, 120.0, 144.0];
const STEP_DEGREES: f32 = 20.0;
const RESPONSE_WINDOW_SECS: f32 = 0.5;
const FIRST_FRAME_MIN_GAIN: f32 = 0.20;
const MAX_T50_SECS: f32 = 0.067;
const MAX_T90_SECS: f32 = 0.150;
const MAX_SETTLING_TIME_SECS: f32 = 0.250;
const SETTLED_ERROR_DEGREES: f32 = 0.5;
const MAX_OVERSHOOT_FRACTION: f32 = 0.05;
const REVERSAL_RESPONSE_DEGREES: f32 = 0.01;
const MAX_REVERSAL_RESPONSE_SECS: f32 = 0.0334;

#[derive(Clone, Copy, Debug)]
enum ResponseAxis {
    Yaw,
    Pitch,
}

#[derive(Clone, Copy)]
struct RenderedAngles {
    yaw_degrees: f32,
    pitch_degrees: f32,
}

struct ClearCameraHarness {
    dt: f32,
    follow: FollowCamera,
    tuning: CameraControlTuning,
    control: CameraControlState,
    obstruction: CameraObstructionHandoffState,
    player_position: Vec3,
    follow_direction: Vec3,
    camera_position: Vec3,
    camera_rotation: Quat,
}

impl ClearCameraHarness {
    fn new(frame_rate_hz: f32) -> Self {
        let follow = FollowCamera::default();
        let player_position = Vec3::Y * 20.0;
        let follow_direction = Vec3::NEG_Z;
        let camera_position =
            player_position - follow_direction * follow.distance + Vec3::Y * follow.height;
        let look_target =
            player_position + Vec3::Y * follow.look_height + follow_direction * follow.look_ahead;
        let camera_rotation = Transform::from_translation(camera_position)
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let mut harness = Self {
            dt: 1.0 / frame_rate_hz,
            follow,
            tuning: CameraControlTuning::default(),
            control: CameraControlState::default(),
            obstruction: CameraObstructionHandoffState::default(),
            player_position,
            follow_direction,
            camera_position,
            camera_rotation,
        };

        harness.step(CameraInput::default());
        harness
    }

    fn input_for_orbit_delta(&self, axis: ResponseAxis, delta_degrees: f32) -> CameraInput {
        let delta_radians = delta_degrees.to_radians();
        let mouse_delta = match axis {
            ResponseAxis::Yaw => Vec2::new(-delta_radians / self.tuning.sensitivity_x, 0.0),
            ResponseAxis::Pitch => {
                let y_sign = if self.tuning.invert_y { 1.0 } else { -1.0 };
                Vec2::new(0.0, delta_radians / (self.tuning.sensitivity_y * y_sign))
            }
        };

        CameraInput { mouse_delta }
    }

    fn rendered_angles(&self) -> RenderedAngles {
        RenderedAngles {
            yaw_degrees: camera_view_yaw_degrees(self.camera_rotation, self.follow_direction),
            pitch_degrees: camera_pitch_degrees(self.camera_rotation),
        }
    }

    fn step(&mut self, input: CameraInput) -> RenderedAngles {
        step_camera_control(&mut self.control, input, &self.tuning, self.dt);
        let frame = step_camera_with_direction_and_input(
            self.camera_position,
            self.camera_rotation,
            self.player_position,
            self.follow_direction,
            &self.follow,
            self.control.orbit,
            self.control.input_active,
            self.dt,
        );
        self.obstruction
            .set_intentional_camera_motion(self.control.input_active);
        let obstruction_step = resolve_camera_obstruction_handoff(
            frame,
            self.camera_position,
            self.camera_rotation,
            self.player_position,
            std::iter::empty::<CameraObstruction>(),
            0.0,
            self.dt,
            &mut self.obstruction,
            |frame| frame,
        );

        assert_eq!(
            obstruction_step.obstruction_hits, 0,
            "clear-camera response harness unexpectedly entered obstruction handoff"
        );
        self.camera_position = obstruction_step.frame.position;
        self.camera_rotation = obstruction_step.frame.rotation;
        self.rendered_angles()
    }
}

#[test]
fn clear_camera_static_yaw_step_meets_response_contract_across_frame_rates() {
    assert_static_response_contract(ResponseAxis::Yaw);
}

#[test]
fn clear_camera_static_pitch_step_meets_response_contract_across_frame_rates() {
    assert_static_response_contract(ResponseAxis::Pitch);
}

#[test]
fn clear_camera_yaw_reversal_responds_without_wrong_way_motion_across_frame_rates() {
    for frame_rate_hz in FRAME_RATES_HZ {
        let mut harness = ClearCameraHarness::new(frame_rate_hz);
        let neutral_yaw = harness.rendered_angles().yaw_degrees;
        let positive_input = harness.input_for_orbit_delta(ResponseAxis::Yaw, STEP_DEGREES);
        harness.step(positive_input);
        for _ in 1..(frame_rate_hz as usize) {
            harness.step(CameraInput::default());
        }

        let reversal_start_yaw = harness.rendered_angles().yaw_degrees;
        let positive_response = signed_angle_delta_degrees(neutral_yaw, reversal_start_yaw);
        assert!(
            (positive_response - STEP_DEGREES).abs() <= SETTLED_ERROR_DEGREES,
            "{frame_rate_hz} Hz reversal setup did not settle at +20 degrees: {positive_response:.3}"
        );

        let reversal_input = harness.input_for_orbit_delta(ResponseAxis::Yaw, -2.0 * STEP_DEGREES);
        let frame_count = (RESPONSE_WINDOW_SECS * frame_rate_hz).ceil() as usize;
        let mut displacements = Vec::with_capacity(frame_count);
        for frame_index in 1..=frame_count {
            let input = if frame_index == 1 {
                reversal_input
            } else {
                CameraInput::default()
            };
            let rendered = harness.step(input);
            displacements.push((
                frame_index as f32 / frame_rate_hz,
                signed_angle_delta_degrees(reversal_start_yaw, rendered.yaw_degrees),
            ));
        }

        let correct_direction_time = displacements
            .iter()
            .find_map(|(time, displacement)| {
                (*displacement <= -REVERSAL_RESPONSE_DEGREES).then_some(*time)
            })
            .unwrap_or(f32::INFINITY);
        assert!(
            correct_direction_time <= MAX_REVERSAL_RESPONSE_SECS,
            "{frame_rate_hz} Hz reversal first moved toward -20 degrees at {:.2} ms",
            correct_direction_time * 1_000.0
        );

        for (frame_index, frames) in displacements.windows(2).enumerate().skip(1) {
            let frame_delta = frames[1].1 - frames[0].1;
            assert!(
                frame_delta <= REVERSAL_RESPONSE_DEGREES,
                "{frame_rate_hz} Hz reversal moved the wrong way after frame {} by {frame_delta:.3} degrees",
                frame_index + 2
            );
        }
    }
}

fn assert_static_response_contract(axis: ResponseAxis) {
    for frame_rate_hz in FRAME_RATES_HZ {
        let mut harness = ClearCameraHarness::new(frame_rate_hz);
        let baseline = harness.rendered_angles();
        let step_input = harness.input_for_orbit_delta(axis, STEP_DEGREES);
        let frame_count = (RESPONSE_WINDOW_SECS * frame_rate_hz).ceil() as usize;
        let mut samples = Vec::with_capacity(frame_count);

        for frame_index in 1..=frame_count {
            let input = if frame_index == 1 {
                step_input
            } else {
                CameraInput::default()
            };
            let rendered = harness.step(input);
            samples.push((
                frame_index as f32 / frame_rate_hz,
                rendered_progress_degrees(axis, baseline, rendered),
            ));
        }

        let first_frame_gain = samples[0].1 / STEP_DEGREES;
        let t50 = first_crossing_time(&samples, STEP_DEGREES * 0.5);
        let t90 = first_crossing_time(&samples, STEP_DEGREES * 0.9);
        let settling_time = settling_time(&samples, STEP_DEGREES, SETTLED_ERROR_DEGREES);
        let max_progress = samples
            .iter()
            .map(|(_, progress)| *progress)
            .fold(f32::NEG_INFINITY, f32::max);
        let overshoot_fraction = (max_progress - STEP_DEGREES).max(0.0) / STEP_DEGREES;

        assert!(
            first_frame_gain + 0.0001 >= FIRST_FRAME_MIN_GAIN,
            "{axis:?} at {frame_rate_hz} Hz first-frame rendered gain was {:.1}%",
            first_frame_gain * 100.0
        );
        assert!(
            t50 <= MAX_T50_SECS,
            "{axis:?} at {frame_rate_hz} Hz t50 was {:.2} ms",
            t50 * 1_000.0
        );
        assert!(
            t90 <= MAX_T90_SECS,
            "{axis:?} at {frame_rate_hz} Hz t90 was {:.2} ms",
            t90 * 1_000.0
        );
        assert!(
            settling_time <= MAX_SETTLING_TIME_SECS,
            "{axis:?} at {frame_rate_hz} Hz settled after {:.2} ms",
            settling_time * 1_000.0
        );
        assert!(
            overshoot_fraction <= MAX_OVERSHOOT_FRACTION + 0.0001,
            "{axis:?} at {frame_rate_hz} Hz overshot by {:.1}%",
            overshoot_fraction * 100.0
        );
    }
}

fn rendered_progress_degrees(
    axis: ResponseAxis,
    baseline: RenderedAngles,
    rendered: RenderedAngles,
) -> f32 {
    match axis {
        ResponseAxis::Yaw => signed_angle_delta_degrees(baseline.yaw_degrees, rendered.yaw_degrees),
        ResponseAxis::Pitch => rendered.pitch_degrees - baseline.pitch_degrees,
    }
}

fn signed_angle_delta_degrees(from_degrees: f32, to_degrees: f32) -> f32 {
    let delta = (to_degrees - from_degrees).to_radians();
    delta.sin().atan2(delta.cos()).to_degrees()
}

fn first_crossing_time(samples: &[(f32, f32)], threshold_degrees: f32) -> f32 {
    samples
        .iter()
        .find_map(|(time, progress)| (*progress >= threshold_degrees).then_some(*time))
        .unwrap_or(f32::INFINITY)
}

fn settling_time(samples: &[(f32, f32)], target_degrees: f32, tolerance_degrees: f32) -> f32 {
    samples
        .iter()
        .enumerate()
        .find_map(|(index, (time, _))| {
            samples[index..]
                .iter()
                .all(|(_, progress)| (progress - target_degrees).abs() <= tolerance_degrees)
                .then_some(*time)
        })
        .unwrap_or(f32::INFINITY)
}
