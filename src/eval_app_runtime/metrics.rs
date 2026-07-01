use super::scene::EvalScene;
use crate::authored_assets::{
    AuthoredAnimationDiagnostics, AuthoredPlayerAnimation, AuthoredPlayerAttachmentMarker,
    AuthoredPlayerPoseNode, authored_player_clip_for_pose_intent_with_input,
};
use crate::camera_runtime::CAMERA_PLAYER_FOCUS_HEIGHT;
use crate::environment_visuals::{
    CrosswindGuide, CrosswindRibbon, ObservedWindVisualMotionMetrics, UpdraftGuide, UpdraftRibbon,
    observe_crosswind_guide_frame_motion, observe_crosswind_ribbon_frame_motion,
    observe_updraft_guide_frame_motion, observe_updraft_ribbon_frame_motion,
    observed_wind_visual_velocity, wind_guide_visual_metrics, wind_responsive_visual_metrics,
    wind_visual_quality_visible,
};
use crate::eval_runtime::{EvalMovementBasis, EvalRun};
use crate::player_runtime::AuthoredGliderPose;
use crate::{grounded_visual_foot_gap_m, movement_facing};
use bevy::prelude::*;
use nau_engine::animation::{
    CharacterPart, CharacterPartRole, MIN_KEY_POSE_READABILITY_SCORE, PlayerPoseContext,
    PlayerPoseIntent, PoseReadabilityMetrics, PoseReadabilityPartTransforms, ScarfSegment, Side,
    body_local_pose_velocity, key_pose_readability_score, pose_readability_metrics,
    pose_readability_metrics_from_part_transforms,
};
use nau_engine::camera::{
    CameraControlState, camera_distance, camera_pitch_degrees, camera_surface_clearance,
    camera_target_angle_degrees, camera_view_yaw_degrees,
};
use nau_engine::diagnostics::frame_ms;
use nau_engine::environment::{
    AERIAL_POWER_UP_ROUTE, active_lift_fields_at, readable_lift_fields_at, visible_fields_at,
    wind_flow_metrics_at,
};
use nau_engine::eval::{
    EvalMovementMetrics, EvalObjectiveProgress, EvalPoseReadabilityMetrics,
    EvalPoseTemporalMetrics, EvalSample, scripted_input,
};
use nau_engine::movement::{
    FlightInput, FlightMode, body_roll_degrees, body_yaw_error_degrees,
    desired_heading_alignment_speed, desired_planar_movement_direction,
    desired_planar_travel_heading_error_degrees, lateral_response_speed,
};
use std::collections::HashMap;

pub(super) const EVAL_FRAME_TIME_WARMUP_FRAMES: u32 = 5;
const BODY_TRAVEL_HEADING_MIN_PLANAR_SPEED_MPS: f32 = 6.0;
const KEY_POSE_TRANSITION_READABILITY_FLOOR: f32 = 0.65;
const KEY_POSE_AIR_BRAKE_RELEASE_TRANSITION_READABILITY_FLOOR: f32 = 0.30;
const KEY_POSE_LANDING_FLIP_TRANSITION_READABILITY_FLOOR: f32 = 0.28;
const KEY_POSE_LANDING_RELEASE_TRANSITION_READABILITY_FLOOR: f32 = 0.35;
const KEY_POSE_READABILITY_EPSILON: f32 = 0.01;
const KEY_POSE_TRANSITION_GRACE_FRAMES: u32 = 5;
const KEY_POSE_EXTENDED_TRANSITION_GRACE_FRAMES: u32 = 8;
const KEY_POSE_LANDING_TRANSITION_GRACE_FRAMES: u32 = 12;
const KEY_POSE_TRANSITION_MAX_ROTATION_DELTA_DEGREES: f32 = 60.0;
const KEY_POSE_TRANSITION_MAX_TRANSLATION_DELTA_M: f32 = 0.15;
const KEY_POSE_LANDING_TRANSITION_MAX_ROTATION_DELTA_DEGREES: f32 = 60.0;
const KEY_POSE_LANDING_TRANSITION_MAX_TRANSLATION_DELTA_M: f32 = 0.25;

#[derive(Resource, Default)]
pub(crate) struct VisiblePoseTemporalState {
    previous_frame: Option<u32>,
    previous_parts: Option<VisiblePosePartSet>,
    previous_key_intent: Option<PlayerPoseIntent>,
    transition_from_key_intent: Option<PlayerPoseIntent>,
    key_intent_age_frames: u32,
    visible_pose_part_count: u32,
    pending_pose_temporal_samples: u32,
    pending_max_pose_part_rotation_delta_degrees: f32,
    pending_max_pose_part_translation_delta_m: f32,
    pending_pose_clearance_samples: u32,
    pending_min_pose_limb_clearance_m: f32,
    pending_max_pose_limb_penetration_m: f32,
    pending_pose_attachment_samples: u32,
    pending_max_pose_joint_gap_m: f32,
}

impl VisiblePoseTemporalState {
    fn transition_from_key_intent(&self) -> Option<PlayerPoseIntent> {
        self.transition_from_key_intent
    }

    fn key_intent_age_frames(&self) -> u32 {
        self.key_intent_age_frames
    }

    fn observe_frame(
        &mut self,
        frame: u32,
        intent: PlayerPoseIntent,
        current: VisiblePosePartSet,
        attachments: VisiblePoseAttachmentSet,
    ) {
        self.visible_pose_part_count = current.part_count();
        let current_parts = current.complete();

        if key_pose_intent(intent)
            && let Some(current_parts) = current_parts
            && let Some(clearance_m) = current.min_limb_clearance_m()
        {
            self.pending_min_pose_limb_clearance_m = if self.pending_pose_clearance_samples == 0 {
                clearance_m
            } else {
                self.pending_min_pose_limb_clearance_m.min(clearance_m)
            };
            self.pending_max_pose_limb_penetration_m = self
                .pending_max_pose_limb_penetration_m
                .max((-clearance_m).max(0.0));
            self.pending_pose_clearance_samples += 1;

            if let Some(joint_gap_m) =
                current_parts.max_joint_gap_m(current, current.head, attachments)
            {
                self.pending_max_pose_joint_gap_m =
                    self.pending_max_pose_joint_gap_m.max(joint_gap_m);
                self.pending_pose_attachment_samples += 1;
            }
        }

        if key_pose_intent(intent)
            && let (Some(previous_frame), Some(previous), Some(_current_parts)) =
                (self.previous_frame, self.previous_parts, current_parts)
            && previous_frame < frame
        {
            self.pending_pose_temporal_samples += 1;
            self.pending_max_pose_part_rotation_delta_degrees = self
                .pending_max_pose_part_rotation_delta_degrees
                .max(current.max_rotation_delta_degrees(previous));
            self.pending_max_pose_part_translation_delta_m = self
                .pending_max_pose_part_translation_delta_m
                .max(current.max_translation_delta_m(previous));
        }

        self.previous_frame = Some(frame);
        self.previous_parts = Some(current);
        if key_pose_intent(intent) {
            if self.previous_key_intent == Some(intent) {
                self.key_intent_age_frames = self.key_intent_age_frames.saturating_add(1);
            } else {
                self.transition_from_key_intent = self.previous_key_intent;
                self.previous_key_intent = Some(intent);
                self.key_intent_age_frames = 1;
            }
        } else {
            self.previous_key_intent = None;
            self.transition_from_key_intent = None;
            self.key_intent_age_frames = 0;
        }
    }

    fn take_sample_metrics(&mut self) -> EvalPoseTemporalMetrics {
        let metrics = EvalPoseTemporalMetrics {
            visible_pose_part_count: self.visible_pose_part_count,
            max_pose_part_rotation_delta_degrees: if self.pending_pose_temporal_samples > 0 {
                self.pending_max_pose_part_rotation_delta_degrees
            } else {
                f32::NAN
            },
            max_pose_part_translation_delta_m: if self.pending_pose_temporal_samples > 0 {
                self.pending_max_pose_part_translation_delta_m
            } else {
                f32::NAN
            },
            min_pose_limb_clearance_m: if self.pending_pose_clearance_samples > 0 {
                self.pending_min_pose_limb_clearance_m
            } else {
                f32::NAN
            },
            max_pose_limb_penetration_m: if self.pending_pose_clearance_samples > 0 {
                self.pending_max_pose_limb_penetration_m
            } else {
                f32::NAN
            },
            max_pose_joint_gap_m: if self.pending_pose_attachment_samples > 0 {
                self.pending_max_pose_joint_gap_m
            } else {
                f32::NAN
            },
            pose_joint_gap_samples: self.pending_pose_attachment_samples,
        };
        self.pending_pose_temporal_samples = 0;
        self.pending_max_pose_part_rotation_delta_degrees = 0.0;
        self.pending_max_pose_part_translation_delta_m = 0.0;
        self.pending_pose_clearance_samples = 0;
        self.pending_min_pose_limb_clearance_m = 0.0;
        self.pending_max_pose_limb_penetration_m = 0.0;
        self.pending_pose_attachment_samples = 0;
        self.pending_max_pose_joint_gap_m = 0.0;
        metrics
    }
}

#[derive(Resource, Default)]
pub(crate) struct ObservedWindVisualMotionState {
    previous: HashMap<Entity, WindVisualFrameSnapshot>,
    pending: ObservedWindVisualMotionMetrics,
}

impl ObservedWindVisualMotionState {
    fn observe_frame<'a>(
        &mut self,
        frame: u32,
        elapsed_secs: f32,
        updraft_guides: impl Iterator<Item = (Entity, &'a UpdraftGuide, &'a Transform)>,
        updraft_ribbons: impl Iterator<Item = (Entity, &'a UpdraftRibbon, &'a Transform)>,
        crosswind_guides: impl Iterator<Item = (Entity, &'a CrosswindGuide, &'a Transform)>,
        crosswind_ribbons: impl Iterator<Item = (Entity, &'a CrosswindRibbon, &'a Transform)>,
    ) {
        let previous = std::mem::take(&mut self.previous);
        let mut current = HashMap::with_capacity(previous.len());

        for (entity, guide, transform) in updraft_guides {
            let quality_visible = wind_visual_quality_visible(transform.scale);
            if let Some(snapshot) = previous
                .get(&entity)
                .filter(|snapshot| snapshot.frame < frame)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
            {
                let dt_secs = elapsed_secs - snapshot.elapsed_secs;
                observe_updraft_guide_frame_motion(
                    &mut self.pending,
                    guide,
                    &snapshot.transform,
                    transform,
                    snapshot.elapsed_secs,
                    dt_secs,
                    snapshot.velocity,
                );
            }
            let velocity = previous
                .get(&entity)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
                .and_then(|snapshot| {
                    observed_wind_visual_velocity(
                        snapshot.transform.translation,
                        transform.translation,
                        elapsed_secs - snapshot.elapsed_secs,
                    )
                });
            current.insert(
                entity,
                WindVisualFrameSnapshot::new(
                    frame,
                    elapsed_secs,
                    transform,
                    velocity,
                    quality_visible,
                ),
            );
        }

        for (entity, ribbon, transform) in updraft_ribbons {
            let quality_visible = wind_visual_quality_visible(transform.scale);
            if let Some(snapshot) = previous
                .get(&entity)
                .filter(|snapshot| snapshot.frame < frame)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
            {
                let dt_secs = elapsed_secs - snapshot.elapsed_secs;
                observe_updraft_ribbon_frame_motion(
                    &mut self.pending,
                    ribbon,
                    &snapshot.transform,
                    transform,
                    snapshot.elapsed_secs,
                    dt_secs,
                    snapshot.velocity,
                );
            }
            let velocity = previous
                .get(&entity)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
                .and_then(|snapshot| {
                    observed_wind_visual_velocity(
                        snapshot.transform.translation,
                        transform.translation,
                        elapsed_secs - snapshot.elapsed_secs,
                    )
                });
            current.insert(
                entity,
                WindVisualFrameSnapshot::new(
                    frame,
                    elapsed_secs,
                    transform,
                    velocity,
                    quality_visible,
                ),
            );
        }

        for (entity, guide, transform) in crosswind_guides {
            let quality_visible = wind_visual_quality_visible(transform.scale);
            if let Some(snapshot) = previous
                .get(&entity)
                .filter(|snapshot| snapshot.frame < frame)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
            {
                let dt_secs = elapsed_secs - snapshot.elapsed_secs;
                observe_crosswind_guide_frame_motion(
                    &mut self.pending,
                    guide,
                    &snapshot.transform,
                    transform,
                    snapshot.elapsed_secs,
                    dt_secs,
                    snapshot.velocity,
                );
            }
            let velocity = previous
                .get(&entity)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
                .and_then(|snapshot| {
                    observed_wind_visual_velocity(
                        snapshot.transform.translation,
                        transform.translation,
                        elapsed_secs - snapshot.elapsed_secs,
                    )
                });
            current.insert(
                entity,
                WindVisualFrameSnapshot::new(
                    frame,
                    elapsed_secs,
                    transform,
                    velocity,
                    quality_visible,
                ),
            );
        }

        for (entity, ribbon, transform) in crosswind_ribbons {
            let quality_visible = wind_visual_quality_visible(transform.scale);
            if let Some(snapshot) = previous
                .get(&entity)
                .filter(|snapshot| snapshot.frame < frame)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
            {
                let dt_secs = elapsed_secs - snapshot.elapsed_secs;
                observe_crosswind_ribbon_frame_motion(
                    &mut self.pending,
                    ribbon,
                    &snapshot.transform,
                    transform,
                    snapshot.elapsed_secs,
                    dt_secs,
                    snapshot.velocity,
                );
            }
            let velocity = previous
                .get(&entity)
                .filter(|snapshot| snapshot.quality_visible && quality_visible)
                .and_then(|snapshot| {
                    observed_wind_visual_velocity(
                        snapshot.transform.translation,
                        transform.translation,
                        elapsed_secs - snapshot.elapsed_secs,
                    )
                });
            current.insert(
                entity,
                WindVisualFrameSnapshot::new(
                    frame,
                    elapsed_secs,
                    transform,
                    velocity,
                    quality_visible,
                ),
            );
        }

        self.previous = current;
    }

    fn take_sample_metrics(&mut self) -> ObservedWindVisualMotionMetrics {
        let metrics = self.pending;
        self.pending = ObservedWindVisualMotionMetrics::default();
        metrics
    }
}

#[derive(Clone, Debug)]
struct WindVisualFrameSnapshot {
    frame: u32,
    elapsed_secs: f32,
    transform: Transform,
    velocity: Option<Vec3>,
    quality_visible: bool,
}

impl WindVisualFrameSnapshot {
    fn new(
        frame: u32,
        elapsed_secs: f32,
        transform: &Transform,
        velocity: Option<Vec3>,
        quality_visible: bool,
    ) -> Self {
        Self {
            frame,
            elapsed_secs,
            transform: *transform,
            velocity,
            quality_visible,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct VisiblePosePartTransform {
    translation: Vec3,
    global_translation: Vec3,
    base_delta: Vec3,
    rotation: Quat,
}

impl VisiblePosePartTransform {
    fn from_part(part: &CharacterPart, transform: &Transform, global_translation: Vec3) -> Self {
        Self {
            translation: transform.translation,
            global_translation,
            base_delta: transform.translation - part.base_translation,
            rotation: transform.rotation,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct VisiblePosePartSet {
    hips: Option<VisiblePosePartTransform>,
    torso: Option<VisiblePosePartTransform>,
    head: Option<VisiblePosePartTransform>,
    left_arm: Option<VisiblePosePartTransform>,
    right_arm: Option<VisiblePosePartTransform>,
    left_forearm: Option<VisiblePosePartTransform>,
    right_forearm: Option<VisiblePosePartTransform>,
    left_hand: Option<VisiblePosePartTransform>,
    right_hand: Option<VisiblePosePartTransform>,
    left_leg: Option<VisiblePosePartTransform>,
    right_leg: Option<VisiblePosePartTransform>,
    left_lower_leg: Option<VisiblePosePartTransform>,
    right_lower_leg: Option<VisiblePosePartTransform>,
    left_foot: Option<VisiblePosePartTransform>,
    right_foot: Option<VisiblePosePartTransform>,
    scarf_anchor: Option<VisiblePosePartTransform>,
    scarf_tail: Option<VisiblePosePartTransform>,
}

impl VisiblePosePartSet {
    fn part_count(self) -> u32 {
        self.articulated_parts()
            .into_iter()
            .filter(Option::is_some)
            .count() as u32
    }

    fn complete(self) -> Option<VisiblePosePartTransforms> {
        Some(VisiblePosePartTransforms {
            hips: self.hips,
            torso: self.torso?,
            left_arm: self.left_arm?,
            right_arm: self.right_arm?,
            left_leg: self.left_leg?,
            right_leg: self.right_leg?,
        })
    }

    fn readability_metrics(self, context: PlayerPoseContext) -> Option<PoseReadabilityMetrics> {
        self.complete()
            .map(|parts| parts.readability_metrics(context, self.scarf_anchor, self.scarf_tail))
    }

    fn torso_offset_m(self) -> f32 {
        self.torso
            .map(|torso| torso.base_delta.length())
            .unwrap_or(f32::NAN)
    }

    fn torso_local_bend_degrees(self) -> f32 {
        self.torso
            .map(|torso| torso.rotation.angle_between(Quat::IDENTITY).to_degrees())
            .unwrap_or(f32::NAN)
    }

    fn articulated_parts(self) -> [Option<VisiblePosePartTransform>; 15] {
        [
            self.hips,
            self.torso,
            self.head,
            self.left_arm,
            self.right_arm,
            self.left_forearm,
            self.right_forearm,
            self.left_hand,
            self.right_hand,
            self.left_leg,
            self.right_leg,
            self.left_lower_leg,
            self.right_lower_leg,
            self.left_foot,
            self.right_foot,
        ]
    }

    fn max_rotation_delta_degrees(self, previous: Self) -> f32 {
        self.articulated_parts()
            .into_iter()
            .zip(previous.articulated_parts())
            .filter_map(|(current, previous)| Some((current?, previous?)))
            .map(|(current, previous)| {
                current
                    .rotation
                    .angle_between(previous.rotation)
                    .to_degrees()
            })
            .fold(0.0, f32::max)
    }

    fn max_translation_delta_m(self, previous: Self) -> f32 {
        self.articulated_parts()
            .into_iter()
            .zip(previous.articulated_parts())
            .filter_map(|(current, previous)| Some((current?, previous?)))
            .map(|(current, previous)| current.translation.distance(previous.translation))
            .fold(0.0, f32::max)
    }

    fn min_limb_clearance_m(self) -> Option<f32> {
        const TORSO_RADIUS_M: f32 = 0.26;
        const ARM_RADIUS_M: f32 = 0.10;
        const FOREARM_RADIUS_M: f32 = 0.055;
        const HAND_RADIUS_M: f32 = 0.047;
        const LEG_RADIUS_M: f32 = 0.11;
        const LOWER_LEG_RADIUS_M: f32 = 0.067;
        const FOOT_RADIUS_M: f32 = 0.059;

        let parts = [
            self.torso.map(|part| (part, TORSO_RADIUS_M)),
            self.left_arm.map(|part| (part, ARM_RADIUS_M)),
            self.right_arm.map(|part| (part, ARM_RADIUS_M)),
            self.left_forearm.map(|part| (part, FOREARM_RADIUS_M)),
            self.right_forearm.map(|part| (part, FOREARM_RADIUS_M)),
            self.left_hand.map(|part| (part, HAND_RADIUS_M)),
            self.right_hand.map(|part| (part, HAND_RADIUS_M)),
            self.left_leg.map(|part| (part, LEG_RADIUS_M)),
            self.right_leg.map(|part| (part, LEG_RADIUS_M)),
            self.left_lower_leg.map(|part| (part, LOWER_LEG_RADIUS_M)),
            self.right_lower_leg.map(|part| (part, LOWER_LEG_RADIUS_M)),
            self.left_foot.map(|part| (part, FOOT_RADIUS_M)),
            self.right_foot.map(|part| (part, FOOT_RADIUS_M)),
        ];
        let present = parts.into_iter().flatten().collect::<Vec<_>>();
        let mut min_clearance = f32::INFINITY;
        for index in 0..present.len() {
            for other_index in (index + 1)..present.len() {
                let (a, a_radius_m) = present[index];
                let (b, b_radius_m) = present[other_index];
                min_clearance = min_clearance.min(limb_clearance(a, b, a_radius_m, b_radius_m));
            }
        }
        min_clearance.is_finite().then_some(min_clearance)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct VisiblePoseAttachmentSet {
    neck: Option<Vec3>,
    left_shoulder: Option<Vec3>,
    right_shoulder: Option<Vec3>,
    left_elbow: Option<Vec3>,
    right_elbow: Option<Vec3>,
    left_wrist: Option<Vec3>,
    right_wrist: Option<Vec3>,
    left_hip: Option<Vec3>,
    right_hip: Option<Vec3>,
    left_knee: Option<Vec3>,
    right_knee: Option<Vec3>,
    left_ankle: Option<Vec3>,
    right_ankle: Option<Vec3>,
}

#[derive(Clone, Copy, Debug)]
struct VisiblePosePartTransforms {
    hips: Option<VisiblePosePartTransform>,
    torso: VisiblePosePartTransform,
    left_arm: VisiblePosePartTransform,
    right_arm: VisiblePosePartTransform,
    left_leg: VisiblePosePartTransform,
    right_leg: VisiblePosePartTransform,
}

impl VisiblePosePartTransforms {
    fn readability_metrics(
        self,
        context: PlayerPoseContext,
        scarf_anchor: Option<VisiblePosePartTransform>,
        scarf_tail: Option<VisiblePosePartTransform>,
    ) -> PoseReadabilityMetrics {
        let mut metrics = pose_readability_metrics_from_part_transforms(
            context,
            PoseReadabilityPartTransforms {
                torso_rotation: self.hips.map_or(self.torso.rotation, |hips| {
                    hips.rotation * self.torso.rotation
                }),
                left_arm_rotation: self.left_arm.rotation,
                right_arm_rotation: self.right_arm.rotation,
                left_leg_rotation: self.left_leg.rotation,
                right_leg_rotation: self.right_leg.rotation,
                left_leg_translation: self.left_leg.base_delta,
                right_leg_translation: self.right_leg.base_delta,
            },
        );
        if let Some(scarf_tail) = scarf_tail {
            metrics.scarf_stream_m = scarf_tail.base_delta.z.max(0.0);
            metrics.scarf_lateral_sway_m = scarf_tail.base_delta.x.abs();
            metrics.scarf_tail_flex_degrees = scarf_anchor.map_or_else(
                || {
                    scarf_tail
                        .rotation
                        .angle_between(Quat::IDENTITY)
                        .to_degrees()
                },
                |scarf_anchor| {
                    scarf_tail
                        .rotation
                        .angle_between(scarf_anchor.rotation)
                        .to_degrees()
                },
            );
        }
        metrics
    }

    fn max_joint_gap_m(
        self,
        parts: VisiblePosePartSet,
        head: Option<VisiblePosePartTransform>,
        attachments: VisiblePoseAttachmentSet,
    ) -> Option<f32> {
        let head = head?;
        Some(
            [
                self.left_arm
                    .global_translation
                    .distance(attachments.left_shoulder?),
                self.right_arm
                    .global_translation
                    .distance(attachments.right_shoulder?),
                parts
                    .left_forearm
                    .zip(attachments.left_elbow)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                parts
                    .right_forearm
                    .zip(attachments.right_elbow)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                parts
                    .left_hand
                    .zip(attachments.left_wrist)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                parts
                    .right_hand
                    .zip(attachments.right_wrist)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                self.left_leg
                    .global_translation
                    .distance(attachments.left_hip?),
                self.right_leg
                    .global_translation
                    .distance(attachments.right_hip?),
                parts
                    .left_lower_leg
                    .zip(attachments.left_knee)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                parts
                    .right_lower_leg
                    .zip(attachments.right_knee)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                parts
                    .left_foot
                    .zip(attachments.left_ankle)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                parts
                    .right_foot
                    .zip(attachments.right_ankle)
                    .map_or(0.0, |(part, marker)| {
                        part.global_translation.distance(marker)
                    }),
                head.global_translation.distance(attachments.neck?),
            ]
            .into_iter()
            .fold(0.0, f32::max),
        )
    }
}

fn limb_clearance(
    a: VisiblePosePartTransform,
    b: VisiblePosePartTransform,
    a_radius_m: f32,
    b_radius_m: f32,
) -> f32 {
    a.global_translation.distance(b.global_translation) - a_radius_m - b_radius_m
}

pub(crate) fn collect_eval_frame_time(time: Res<Time>, mut run: ResMut<EvalRun>) {
    if !run.finalized && run.frame >= EVAL_FRAME_TIME_WARMUP_FRAMES {
        run.accumulator
            .observe_frame_time_ms(frame_ms(time.delta_secs()));
    }
}

pub(crate) fn collect_eval_metrics(
    mut run: ResMut<EvalRun>,
    camera_control: Res<CameraControlState>,
    movement_basis: Res<EvalMovementBasis>,
    mut pose_temporal_state: ResMut<VisiblePoseTemporalState>,
    mut observed_wind_visual_motion_state: ResMut<ObservedWindVisualMotionState>,
    authored_animation_diagnostics: Option<Res<AuthoredAnimationDiagnostics>>,
    scene: EvalScene,
) {
    if run.finalized {
        return;
    }

    let Ok((transform, velocity, controller, animation)) = scene.player.single() else {
        return;
    };
    let (
        camera_distance_m,
        camera_surface_clearance_m,
        camera_player_angle_degrees,
        camera_pitch_degrees,
        camera_view_yaw,
        camera_world_yaw,
    ) = scene
        .camera
        .single()
        .map(|camera_transform| {
            let camera_floor_y = scene.route.ground_at(camera_transform.translation).floor_y;
            let player_focus = transform.translation + Vec3::Y * CAMERA_PLAYER_FOCUS_HEIGHT;
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_surface_clearance(camera_transform.translation, camera_floor_y),
                camera_target_angle_degrees(
                    camera_transform.translation,
                    camera_transform.rotation,
                    player_focus,
                ),
                camera_pitch_degrees(camera_transform.rotation),
                camera_view_yaw_degrees(
                    camera_transform.rotation,
                    scene.camera_diagnostics.follow_direction,
                ),
                camera_view_yaw_degrees(camera_transform.rotation, Vec3::NEG_Z),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields =
        visible_fields_at(transform.translation, scene.wind_fields.iter().copied());
    let elapsed_secs = run.frame as f32 * run.scenario.fixed_dt;
    let wind_flow = wind_flow_metrics_at(
        transform.translation,
        elapsed_secs,
        scene.wind_fields.iter().copied(),
    );
    let active_lift_fields =
        active_lift_fields_at(transform.translation, scene.lift_fields.iter().copied());
    let readable_lift_fields = readable_lift_fields_at(
        transform.translation,
        scene.lift_fields.iter().copied(),
        scene.wind_fields.iter().copied(),
    );
    let player_ground = scene.route.ground_at(transform.translation);
    let visual_foot_gap_m = grounded_visual_foot_gap_m(
        transform.translation.y,
        player_ground.floor_y,
        controller.mode,
    );
    let scenario_target = run.scenario.target_island_name;
    let target_distance_m = scene
        .route
        .target_distance_to(transform.translation, scenario_target);
    let on_landing_target = scene.route.on_landing_target_named(
        transform.translation,
        controller.mode,
        scenario_target,
    );
    let objective = EvalObjectiveProgress::new(
        scene.route_objectives.completed_count,
        scene.route_objectives.total_count,
        scene.route_objectives.current_label,
        scene.route_objectives.current_distance_m,
        scene.route_objectives.complete,
    );
    let streaming_lod = scene.route.streaming_lod_stats(transform.translation);
    let lod_visuals = scene.stream_diagnostics.counts;
    let asset_metrics = scene.asset_diagnostics.metrics;
    let authored_animation_metrics =
        authored_animation_sample_metrics(authored_animation_diagnostics.as_deref());
    let authored_glider_metrics = visible_authored_glider_metrics(scene.authored_gliders.iter());
    let content_metrics = *scene.content_diagnostics;
    let (environment_motion_visuals, max_environment_motion_offset_m) =
        wind_responsive_visual_metrics(scene.wind_responsive_visuals.iter());
    observed_wind_visual_motion_state.observe_frame(
        run.frame,
        elapsed_secs,
        scene.updraft_guides.iter(),
        scene.updraft_ribbons.iter(),
        scene.crosswind_guides.iter(),
        scene.crosswind_ribbons.iter(),
    );
    let wind_guide_metrics = wind_guide_visual_metrics(
        elapsed_secs,
        scene
            .updraft_guides
            .iter()
            .map(|(_, guide, transform)| (guide, transform)),
        scene
            .updraft_ribbons
            .iter()
            .map(|(_, ribbon, transform)| (ribbon, transform)),
        scene
            .crosswind_guides
            .iter()
            .map(|(_, guide, transform)| (guide, transform)),
        scene
            .crosswind_ribbons
            .iter()
            .map(|(_, ribbon, transform)| (ribbon, transform)),
    );
    let movement_input = scripted_input(run.scenario, run.frame);
    let pose_intent_label = animation.pose_intent.label();
    let pose_context = PlayerPoseContext::new(
        controller.mode,
        body_local_pose_velocity(velocity.0, transform.rotation),
        animation.last_input,
        animation.height_above_ground_m,
    )
    .with_landing_recovery(
        controller.landing_recovery_timer,
        controller.landing_impact_speed_mps,
    )
    .with_resolved_intent(animation.pose_intent);
    let fallback_pose_readability = pose_readability_metrics(pose_context, animation.phase);
    let generated_pose_parts =
        visible_generated_pose_part_set(scene.generated_character_parts.iter());
    let visible_pose_parts = if generated_pose_parts.part_count() > 0 {
        generated_pose_parts
    } else {
        visible_authored_pose_part_set(scene.authored_player_pose_nodes.iter())
    };
    let visible_pose_attachments =
        visible_authored_pose_attachment_set(scene.authored_player_attachment_markers.iter());
    pose_temporal_state.observe_frame(
        run.frame,
        pose_context.intent(),
        visible_pose_parts,
        visible_pose_attachments,
    );
    let transition_from_key_intent = pose_temporal_state.transition_from_key_intent();
    let key_intent_age_frames = pose_temporal_state.key_intent_age_frames();

    if !run.scenario.should_sample(run.frame) {
        return;
    }

    let pose_temporal = pose_temporal_state.take_sample_metrics();
    let observed_wind_visual_motion = observed_wind_visual_motion_state.take_sample_metrics();
    let pose_readability = visible_pose_parts
        .readability_metrics(pose_context)
        .unwrap_or_else(|| {
            let authored_metrics = visible_authored_pose_readability_metrics(
                scene.authored_player_pose_nodes.iter(),
                pose_context,
            )
            .unwrap_or_else(|| {
                missing_visible_authored_pose_metrics(
                    fallback_pose_readability,
                    pose_context.intent(),
                )
            });
            authored_pose_readability_metrics(
                scene.authored_player_animations.iter(),
                authored_metrics,
                pose_context.intent(),
                velocity.0.length(),
                pose_context.input,
            )
        });
    let transition_pose_readability = transition_aware_pose_readability(
        pose_readability,
        pose_context.intent(),
        transition_from_key_intent,
        key_intent_age_frames,
        &pose_temporal,
    );
    let pose_readability = transition_pose_readability.metrics;
    let movement_axis = movement_input.planar_axis();
    let movement_facing = if movement_basis.frame == run.frame {
        movement_basis
            .facing
            .unwrap_or_else(|| movement_facing(scene.camera.single().ok(), transform))
    } else {
        movement_facing(scene.camera.single().ok(), transform)
    };
    let desired_movement_direction =
        desired_planar_movement_direction(movement_input, movement_facing);
    let desired_body_yaw_error_degrees = desired_movement_direction
        .map(|direction| body_yaw_error_degrees(transform.rotation, direction))
        .unwrap_or(f32::NAN);
    let desired_heading_alignment_mps = desired_movement_direction
        .map(|direction| desired_heading_alignment_speed(velocity.0, direction))
        .unwrap_or(f32::NAN);
    let desired_travel_heading_error_degrees = desired_movement_direction
        .map(|direction| {
            desired_planar_travel_heading_error_degrees(
                velocity.0,
                direction,
                BODY_TRAVEL_HEADING_MIN_PLANAR_SPEED_MPS,
            )
        })
        .unwrap_or(f32::NAN);
    let lateral_axis_active = movement_input.has_lateral_axis();
    let lateral_input_active = lateral_axis_active && controller.mode != FlightMode::Grounded;
    let body_travel_heading_error_degrees = body_travel_heading_error_degrees(
        transform.rotation,
        velocity.0,
        controller.mode,
        lateral_input_active,
    );
    let lateral_response_mps = if lateral_axis_active {
        lateral_response_speed(velocity.0, movement_input, movement_facing)
    } else {
        0.0
    };
    let sample = EvalSample::new(
        run.frame,
        run.scenario.fixed_dt,
        transform.translation,
        velocity.0,
        controller.mode,
        pose_intent_label,
        camera_distance_m,
        camera_surface_clearance_m,
        camera_player_angle_degrees,
        camera_pitch_degrees,
        camera_control.orbit.yaw_degrees(),
        camera_control.orbit.pitch_degrees(),
        scene.camera_diagnostics.step_distance_m,
        scene.camera_diagnostics.rotation_delta_degrees,
        scene.camera_diagnostics.orbit_alignment_degrees,
        camera_view_yaw,
        scene.camera_diagnostics.obstruction_adjustment_m,
        scene.camera_diagnostics.obstruction_hits,
        visible_wind_fields,
        scene.wind_fields.iter().count(),
        wind_flow.active_fields,
        wind_flow.max_speed_mps,
        wind_flow.max_variation,
        wind_flow.max_direction_change_degrees,
        active_lift_fields,
        readable_lift_fields,
        scene.lift_fields.iter().count(),
        target_distance_m,
        on_landing_target,
        objective,
        scene.route.islands().len(),
        streaming_lod.active_chunk_count,
        streaming_lod.active_island_count,
        streaming_lod.near_lod_islands,
        streaming_lod.mid_lod_islands,
        streaming_lod.far_lod_islands,
        lod_visuals.visible_terrain_count,
        lod_visuals.hidden_terrain_count,
        lod_visuals.visible_impostor_count,
        lod_visuals.hidden_impostor_count,
        lod_visuals.visible_detail_count,
        lod_visuals.hidden_detail_count,
        lod_visuals.visible_beacon_count,
        scene.weather_clouds.iter().count(),
        environment_motion_visuals,
        max_environment_motion_offset_m,
        lod_visuals.resident_count(),
        scene.stream_diagnostics.visibility_changes_this_frame,
        scene.stream_diagnostics.max_visibility_changes_per_frame,
        scene.stream_diagnostics.total_visibility_changes,
        lod_visuals.catalog_count(),
        lod_visuals.hidden_count(),
        lod_visuals.resident_fraction(),
        scene.stream_diagnostics.spawned_visuals_this_frame,
        scene.stream_diagnostics.despawned_visuals_this_frame,
        scene.stream_diagnostics.max_spawned_visuals_per_frame,
        scene.stream_diagnostics.max_despawned_visuals_per_frame,
        scene.stream_diagnostics.total_spawned_visuals,
        scene.stream_diagnostics.total_despawned_visuals,
        scene.all_entities.iter().count(),
        asset_metrics.slot_count,
        asset_metrics.gltf_scene_slot_count,
        asset_metrics.ready_slot_count,
        asset_metrics.placeholder_slot_count,
        asset_metrics.streaming_slot_count,
        asset_metrics.missing_slot_count,
        asset_metrics.queued_scene_count,
        asset_metrics.loading_scene_count,
        asset_metrics.loaded_scene_count,
        asset_metrics.dependency_loaded_scene_count,
        asset_metrics.preload_ready_scene_count,
        asset_metrics.failed_scene_count,
        asset_metrics.spawned_scene_count,
        asset_metrics.ready_scene_count,
        asset_metrics.always_slot_count,
        asset_metrics.stream_window_slot_count,
        asset_metrics.near_lod_slot_count,
        asset_metrics.far_lod_slot_count,
        asset_metrics.weather_slot_count,
        asset_metrics.always_preload_ready_slot_count,
        asset_metrics.streaming_preload_ready_slot_count,
        asset_metrics.declared_animation_clip_count,
        asset_metrics.ready_animation_clip_count,
        asset_metrics.animation_player_count,
        asset_metrics.animation_graph_count,
        AERIAL_POWER_UP_ROUTE.len(),
        scene.power_ups.visible_count(),
        scene.power_ups.collected_count(),
        scene.power_ups.active_effects(),
        scene.power_ups.total_activations(),
    )
    .with_visible_authored_world_fixture_count(scene.asset_diagnostics.visible_world_fixture_count)
    .with_deferred_visual_asset_scene_count(asset_metrics.deferred_scene_count)
    .with_authored_animation_metrics(
        authored_animation_metrics.current_clip_label,
        authored_animation_metrics.desired_clip_label,
        authored_animation_metrics.player_count,
        authored_animation_metrics.transition_duration_ms,
    )
    .with_authored_animation_transition_metrics(
        authored_animation_metrics.transition_from_clip_label,
        authored_animation_metrics.transition_to_clip_label,
        authored_animation_metrics.transition_active,
        authored_animation_metrics.transition_elapsed_ms,
        authored_animation_metrics.transition_progress,
        authored_animation_metrics.transition_class_label,
    )
    .with_authored_glider_metrics(
        authored_glider_metrics.max_response_degrees,
        authored_glider_metrics.max_motion_m,
    )
    .with_camera_follow_metrics(scene.camera_diagnostics.follow_direction_error_degrees)
    .with_camera_world_yaw_metrics(camera_world_yaw)
    .with_visual_foot_gap(visual_foot_gap_m)
    .with_wind_guide_visual_metrics(
        wind_guide_metrics.updraft_guide_count,
        wind_guide_metrics.updraft_ribbon_count,
        wind_guide_metrics.crosswind_guide_count,
        wind_guide_metrics.crosswind_ribbon_count,
        wind_guide_metrics.max_updraft_visual_motion_m,
        wind_guide_metrics.max_updraft_visual_rise_m,
        wind_guide_metrics.max_updraft_visual_swirl_displacement_m,
        wind_guide_metrics.max_crosswind_visual_motion_m,
        wind_guide_metrics.max_crosswind_guide_flow_displacement_m,
        wind_guide_metrics.max_crosswind_ribbon_flow_displacement_m,
    )
    .with_wind_guide_depth_metrics(
        wind_guide_metrics.max_updraft_visual_depth_span_m,
        wind_guide_metrics.max_updraft_visual_scale_pulse,
        wind_guide_metrics.max_crosswind_visual_lane_depth_span_m,
        wind_guide_metrics.max_crosswind_visual_scale_pulse,
    )
    .with_wind_guide_flow_coherence_metrics(
        wind_guide_metrics.updraft_flow_coherent_visual_count,
        wind_guide_metrics.crosswind_flow_coherent_visual_count,
        wind_guide_metrics.max_updraft_visual_flow_alignment,
        wind_guide_metrics.max_crosswind_visual_flow_alignment,
    )
    .with_crosswind_ribbon_flow_coherence_metrics(
        wind_guide_metrics.crosswind_ribbon_flow_coherent_sample_count,
        wind_guide_metrics.max_crosswind_ribbon_visual_flow_alignment,
    )
    .with_observed_wind_visual_motion_metrics(
        observed_wind_visual_motion.observed_updraft_flow_coherent_visual_count,
        observed_wind_visual_motion.observed_crosswind_flow_coherent_visual_count,
        observed_wind_visual_motion.observed_crosswind_ribbon_flow_coherent_sample_count,
        observed_wind_visual_motion.max_observed_updraft_visual_frame_motion_m,
        observed_wind_visual_motion.max_observed_updraft_visual_frame_rise_m,
        observed_wind_visual_motion.max_observed_updraft_visual_frame_swirl_displacement_m,
        observed_wind_visual_motion.max_observed_crosswind_visual_frame_motion_m,
        observed_wind_visual_motion.max_observed_crosswind_guide_frame_flow_displacement_m,
        observed_wind_visual_motion.max_observed_crosswind_ribbon_frame_flow_displacement_m,
        observed_wind_visual_motion.max_observed_updraft_visual_flow_alignment,
        observed_wind_visual_motion.max_observed_crosswind_visual_flow_alignment,
        observed_wind_visual_motion.max_observed_crosswind_ribbon_visual_flow_alignment,
    )
    .with_observed_wind_visual_quality_metrics(
        observed_wind_visual_motion.max_observed_updraft_visual_speed_mps,
        observed_wind_visual_motion.max_observed_crosswind_visual_speed_mps,
        observed_wind_visual_motion.max_observed_wind_visual_acceleration_mps2,
        observed_wind_visual_motion.observed_wind_visual_jump_count,
    )
    .with_wind_field_visual_coverage_metrics(
        wind_guide_metrics.updraft_field_count,
        wind_guide_metrics.updraft_fields_with_guides_count,
        wind_guide_metrics.updraft_fields_with_ribbons_count,
        wind_guide_metrics.updraft_fields_with_guides_and_ribbons_count,
        wind_guide_metrics.updraft_flow_coherent_field_count,
        wind_guide_metrics.crosswind_field_count,
        wind_guide_metrics.crosswind_fields_with_guides_count,
        wind_guide_metrics.crosswind_fields_with_ribbons_count,
        wind_guide_metrics.crosswind_fields_with_guides_and_ribbons_count,
        wind_guide_metrics.crosswind_flow_coherent_field_count,
    )
    .with_wind_force_metrics(
        scene.wind_force_diagnostics.active_fields,
        scene.wind_force_diagnostics.crosswind_fields,
        scene.wind_force_diagnostics.updraft_swirl_fields,
        scene.wind_force_diagnostics.applied_delta_mps,
        scene.wind_force_diagnostics.crosswind_delta_mps,
        scene.wind_force_diagnostics.updraft_swirl_delta_mps,
        scene.wind_force_diagnostics.max_flow_speed_mps,
        scene.wind_force_diagnostics.max_variation,
        scene.wind_force_diagnostics.max_flow_alignment,
        scene.wind_force_diagnostics.max_crosswind_flow_alignment,
        scene
            .wind_force_diagnostics
            .max_updraft_swirl_flow_alignment,
        scene.wind_force_diagnostics.max_flow_aligned_delta_mps,
        scene
            .wind_force_diagnostics
            .max_crosswind_flow_aligned_delta_mps,
        scene
            .wind_force_diagnostics
            .max_updraft_swirl_flow_aligned_delta_mps,
    )
    .with_crosswind_force_delta(scene.wind_force_diagnostics.crosswind_delta)
    .with_wind_lateral_load(scene.wind_force_diagnostics.wind_lateral_load)
    .with_world_collision_metrics(
        scene.collision_diagnostics.proxy_count,
        scene.collision_diagnostics.resolved_count,
        scene.collision_diagnostics.max_push_m,
    )
    .with_terrain_rim_collision_metrics(
        scene.collision_diagnostics.terrain_rim_proxy_count,
        scene.collision_diagnostics.terrain_rim_resolved_count,
        scene.collision_diagnostics.max_terrain_rim_push_m,
    )
    .with_terrain_body_collision_metrics(
        scene.collision_diagnostics.terrain_body_proxy_count,
        scene.collision_diagnostics.terrain_body_resolved_count,
        scene.collision_diagnostics.max_terrain_body_push_m,
    )
    .with_world_collision_kind_metrics(
        scene.collision_diagnostics.solid_proxy_count,
        scene.collision_diagnostics.tree_proxy_count,
        scene.collision_diagnostics.rock_proxy_count,
        scene.collision_diagnostics.landmark_proxy_count,
    )
    .with_pose_readability_metrics(EvalPoseReadabilityMetrics {
        torso_pitch_degrees: pose_readability.torso_pitch_degrees,
        arm_spread_degrees: pose_readability.arm_spread_degrees,
        leg_tuck_degrees: pose_readability.leg_tuck_degrees,
        lateral_lean_degrees: pose_readability.lateral_lean_degrees,
        signed_lateral_lean_degrees: pose_readability.signed_lateral_lean_degrees,
        grounded_stride_foot_travel_m: pose_readability.grounded_stride_foot_travel_m,
        grounded_stride_leg_opposition_degrees: pose_readability
            .grounded_stride_leg_opposition_degrees,
        landing_crouch_m: pose_readability.landing_crouch_m,
        landing_foot_forward_m: pose_readability.landing_foot_forward_m,
        landing_foot_split_m: pose_readability.landing_foot_split_m,
        landing_recovery_flip_degrees: pose_readability.landing_recovery_flip_degrees,
        wing_airflow_strength: pose_readability.wing_airflow_strength,
        key_pose_readability_score: pose_readability.key_pose_readability_score,
    })
    .with_pose_torso_backward_bend(pose_readability.torso_backward_bend_degrees)
    .with_pose_torso_local_bend(visible_pose_parts.torso_local_bend_degrees())
    .with_pose_torso_offset(visible_pose_parts.torso_offset_m())
    .with_scarf_pose_metrics(
        pose_readability.scarf_stream_m,
        pose_readability.scarf_lateral_sway_m,
        pose_readability.scarf_tail_flex_degrees,
    )
    .with_key_pose_transition_grace(transition_pose_readability.used_transition_grace)
    .with_pose_temporal_metrics(pose_temporal)
    .with_content_metrics(
        content_metrics.island_terrain_surface_count,
        content_metrics.min_island_terrain_mesh_vertices,
        content_metrics.min_island_terrain_color_bands,
        content_metrics.min_island_terrain_relief_range_m(),
        content_metrics.island_terrain_archetype_count(),
        content_metrics.min_island_cliff_color_bands,
        content_metrics.procedural_island_body_count,
        content_metrics.primitive_island_body_count,
        content_metrics.min_island_body_silhouette_segments,
        content_metrics.average_island_body_silhouette_segments(),
        content_metrics.min_island_body_mesh_vertices,
        content_metrics.max_island_body_mesh_vertices,
    )
    .with_island_impostor_metrics(
        content_metrics.min_island_impostor_mesh_vertices,
        content_metrics.min_island_impostor_color_bands,
    )
    .with_terrain_material_metrics(
        content_metrics.min_island_terrain_material_weight_bands,
        content_metrics.min_island_terrain_material_channels,
        content_metrics.min_island_terrain_material_regions,
        content_metrics.min_island_terrain_texture_detail_bands,
    )
    .with_generated_visual_shape_metrics(
        content_metrics.generated_ground_cover_patch_count,
        content_metrics.min_ground_cover_blade_count,
        content_metrics.min_ground_cover_mesh_vertices,
        content_metrics.generated_tree_trunk_count,
        content_metrics.generated_tree_canopy_count,
        content_metrics.min_tree_trunk_mesh_vertices,
        content_metrics.min_tree_canopy_mesh_vertices,
        content_metrics.detail_biome_palette_count(),
        content_metrics.generated_rock_count,
        content_metrics.min_rock_mesh_vertices,
        content_metrics.generated_landmark_count,
        content_metrics.generated_route_cairn_count,
        content_metrics.generated_launch_beacon_count,
        content_metrics.generated_landing_garden_marker_count,
        content_metrics.generated_pond_surface_count,
        content_metrics.min_landmark_mesh_vertices,
        content_metrics.generated_weather_cloud_count,
        content_metrics.generated_weather_cloud_bank_count,
        content_metrics.min_weather_cloud_bank_depth_m(),
        content_metrics.min_weather_cloud_lobe_count,
        content_metrics.max_weather_cloud_lobe_count,
        content_metrics.min_weather_cloud_mesh_vertices,
        content_metrics.min_weather_cloud_filament_ribbon_detail_count,
    )
    .with_movement_metrics(EvalMovementMetrics {
        desired_body_yaw_error_degrees,
        body_travel_heading_error_degrees,
        body_roll_degrees: body_roll_degrees(transform.rotation),
        desired_heading_alignment_mps,
        desired_travel_heading_error_degrees,
        lateral_response_mps,
        lateral_input_active,
        movement_axis,
    });

    if let Err(error) = run.record_sample(sample) {
        run.io_error = Some(format!("failed to write eval sample: {error}"));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AuthoredAnimationSampleMetrics {
    current_clip_label: &'static str,
    desired_clip_label: &'static str,
    player_count: usize,
    transition_duration_ms: u64,
    transition_from_clip_label: &'static str,
    transition_to_clip_label: &'static str,
    transition_active: bool,
    transition_elapsed_ms: u64,
    transition_progress: f32,
    transition_class_label: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct AuthoredGliderSampleMetrics {
    max_response_degrees: f32,
    max_motion_m: f32,
}

fn authored_animation_sample_metrics(
    diagnostics: Option<&AuthoredAnimationDiagnostics>,
) -> AuthoredAnimationSampleMetrics {
    let diagnostics = diagnostics.copied().unwrap_or_default();
    AuthoredAnimationSampleMetrics {
        current_clip_label: diagnostics.current_label(),
        desired_clip_label: diagnostics.desired_label(),
        player_count: diagnostics.player_count,
        transition_duration_ms: diagnostics.transition_duration_ms,
        transition_from_clip_label: diagnostics.transition_from_label(),
        transition_to_clip_label: diagnostics.transition_to_label(),
        transition_active: diagnostics.transition_active,
        transition_elapsed_ms: diagnostics.transition_elapsed_ms,
        transition_progress: diagnostics.transition_progress,
        transition_class_label: diagnostics.transition_class_label,
    }
}

fn visible_authored_glider_metrics<'a>(
    gliders: impl Iterator<
        Item = (
            &'a AuthoredGliderPose,
            &'a Transform,
            Option<&'a Visibility>,
            Option<&'a InheritedVisibility>,
        ),
    >,
) -> AuthoredGliderSampleMetrics {
    gliders.fold(
        AuthoredGliderSampleMetrics::default(),
        |metrics, (glider, transform, visibility, inherited_visibility)| {
            if !authored_pose_part_visible(visibility, inherited_visibility) {
                return metrics;
            }
            AuthoredGliderSampleMetrics {
                max_response_degrees: metrics
                    .max_response_degrees
                    .max(glider.response_degrees(transform)),
                max_motion_m: metrics.max_motion_m.max(glider.motion_m(transform)),
            }
        },
    )
}

fn body_travel_heading_error_degrees(
    player_rotation: Quat,
    velocity: Vec3,
    mode: FlightMode,
    lateral_input_active: bool,
) -> f32 {
    if !lateral_input_active || !matches!(mode, FlightMode::Airborne | FlightMode::Gliding) {
        return f32::NAN;
    }

    let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
    if horizontal_velocity.length() < BODY_TRAVEL_HEADING_MIN_PLANAR_SPEED_MPS {
        return f32::NAN;
    }

    body_yaw_error_degrees(player_rotation, horizontal_velocity).abs()
}

#[cfg(test)]
fn visible_generated_pose_readability_metrics<'a>(
    parts: impl Iterator<Item = (&'a CharacterPart, &'a Transform, &'a Visibility)>,
    context: PlayerPoseContext,
) -> Option<PoseReadabilityMetrics> {
    visible_generated_pose_part_set(parts).readability_metrics(context)
}

fn visible_generated_pose_part_set<'a>(
    parts: impl Iterator<Item = (&'a CharacterPart, &'a Transform, &'a Visibility)>,
) -> VisiblePosePartSet {
    let mut parts_set = VisiblePosePartSet::default();

    for (part, transform, visibility) in parts {
        if matches!(*visibility, Visibility::Hidden) {
            continue;
        }

        let pose_part = VisiblePosePartTransform::from_part(part, transform, transform.translation);
        match part.role {
            CharacterPartRole::Hips => parts_set.hips = Some(pose_part),
            CharacterPartRole::Torso => parts_set.torso = Some(pose_part),
            CharacterPartRole::Head => parts_set.head = Some(pose_part),
            CharacterPartRole::Arm(Side::Left) => parts_set.left_arm = Some(pose_part),
            CharacterPartRole::Arm(Side::Right) => parts_set.right_arm = Some(pose_part),
            CharacterPartRole::Forearm(Side::Left) => parts_set.left_forearm = Some(pose_part),
            CharacterPartRole::Forearm(Side::Right) => parts_set.right_forearm = Some(pose_part),
            CharacterPartRole::Hand(Side::Left) => parts_set.left_hand = Some(pose_part),
            CharacterPartRole::Hand(Side::Right) => parts_set.right_hand = Some(pose_part),
            CharacterPartRole::Leg(Side::Left) => parts_set.left_leg = Some(pose_part),
            CharacterPartRole::Leg(Side::Right) => parts_set.right_leg = Some(pose_part),
            CharacterPartRole::LowerLeg(Side::Left) => parts_set.left_lower_leg = Some(pose_part),
            CharacterPartRole::LowerLeg(Side::Right) => {
                parts_set.right_lower_leg = Some(pose_part);
            }
            CharacterPartRole::Foot(Side::Left) => parts_set.left_foot = Some(pose_part),
            CharacterPartRole::Foot(Side::Right) => parts_set.right_foot = Some(pose_part),
            CharacterPartRole::Scarf(ScarfSegment::Anchor) => {
                parts_set.scarf_anchor = Some(pose_part);
            }
            CharacterPartRole::Scarf(ScarfSegment::Trail) => parts_set.scarf_tail = Some(pose_part),
            CharacterPartRole::Wing(_) => {}
        }
    }

    parts_set
}

fn visible_authored_pose_readability_metrics<'a>(
    nodes: impl Iterator<
        Item = (
            &'a AuthoredPlayerPoseNode,
            &'a Transform,
            &'a GlobalTransform,
            Option<&'a Visibility>,
            Option<&'a InheritedVisibility>,
        ),
    >,
    context: PlayerPoseContext,
) -> Option<PoseReadabilityMetrics> {
    visible_authored_pose_part_set(nodes).readability_metrics(context)
}

fn visible_authored_pose_part_set<'a>(
    nodes: impl Iterator<
        Item = (
            &'a AuthoredPlayerPoseNode,
            &'a Transform,
            &'a GlobalTransform,
            Option<&'a Visibility>,
            Option<&'a InheritedVisibility>,
        ),
    >,
) -> VisiblePosePartSet {
    let mut parts_set = VisiblePosePartSet::default();

    for (node, transform, global_transform, visibility, inherited_visibility) in nodes {
        if !authored_pose_part_visible(visibility, inherited_visibility) {
            continue;
        }
        let pose_part = VisiblePosePartTransform::from_part(
            &node.part,
            transform,
            global_transform.translation(),
        );
        match node.part.role {
            CharacterPartRole::Hips => parts_set.hips = Some(pose_part),
            CharacterPartRole::Torso => parts_set.torso = Some(pose_part),
            CharacterPartRole::Head => parts_set.head = Some(pose_part),
            CharacterPartRole::Arm(Side::Left) => parts_set.left_arm = Some(pose_part),
            CharacterPartRole::Arm(Side::Right) => parts_set.right_arm = Some(pose_part),
            CharacterPartRole::Forearm(Side::Left) => parts_set.left_forearm = Some(pose_part),
            CharacterPartRole::Forearm(Side::Right) => parts_set.right_forearm = Some(pose_part),
            CharacterPartRole::Hand(Side::Left) => parts_set.left_hand = Some(pose_part),
            CharacterPartRole::Hand(Side::Right) => parts_set.right_hand = Some(pose_part),
            CharacterPartRole::Leg(Side::Left) => parts_set.left_leg = Some(pose_part),
            CharacterPartRole::Leg(Side::Right) => parts_set.right_leg = Some(pose_part),
            CharacterPartRole::LowerLeg(Side::Left) => parts_set.left_lower_leg = Some(pose_part),
            CharacterPartRole::LowerLeg(Side::Right) => {
                parts_set.right_lower_leg = Some(pose_part);
            }
            CharacterPartRole::Foot(Side::Left) => parts_set.left_foot = Some(pose_part),
            CharacterPartRole::Foot(Side::Right) => parts_set.right_foot = Some(pose_part),
            CharacterPartRole::Scarf(ScarfSegment::Anchor) => {
                parts_set.scarf_anchor = Some(pose_part);
            }
            CharacterPartRole::Scarf(ScarfSegment::Trail) => parts_set.scarf_tail = Some(pose_part),
            CharacterPartRole::Wing(_) => {}
        }
    }

    parts_set
}

fn visible_authored_pose_attachment_set<'a>(
    markers: impl Iterator<
        Item = (
            &'a AuthoredPlayerAttachmentMarker,
            &'a GlobalTransform,
            Option<&'a Visibility>,
            Option<&'a InheritedVisibility>,
        ),
    >,
) -> VisiblePoseAttachmentSet {
    let mut attachments = VisiblePoseAttachmentSet::default();

    for (marker, global_transform, visibility, inherited_visibility) in markers {
        if !authored_pose_part_visible(visibility, inherited_visibility) {
            continue;
        }
        let translation = global_transform.translation();
        match *marker {
            AuthoredPlayerAttachmentMarker::Neck => attachments.neck = Some(translation),
            AuthoredPlayerAttachmentMarker::Shoulder(Side::Left) => {
                attachments.left_shoulder = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Shoulder(Side::Right) => {
                attachments.right_shoulder = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Elbow(Side::Left) => {
                attachments.left_elbow = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Elbow(Side::Right) => {
                attachments.right_elbow = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Wrist(Side::Left) => {
                attachments.left_wrist = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Wrist(Side::Right) => {
                attachments.right_wrist = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Hip(Side::Left) => {
                attachments.left_hip = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Hip(Side::Right) => {
                attachments.right_hip = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Knee(Side::Left) => {
                attachments.left_knee = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Knee(Side::Right) => {
                attachments.right_knee = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Ankle(Side::Left) => {
                attachments.left_ankle = Some(translation);
            }
            AuthoredPlayerAttachmentMarker::Ankle(Side::Right) => {
                attachments.right_ankle = Some(translation);
            }
        }
    }

    attachments
}

fn authored_pose_part_visible(
    visibility: Option<&Visibility>,
    inherited_visibility: Option<&InheritedVisibility>,
) -> bool {
    !matches!(visibility, Some(Visibility::Hidden))
        && inherited_visibility.is_none_or(|visibility| visibility.get())
}

fn missing_visible_authored_pose_metrics(
    mut metrics: PoseReadabilityMetrics,
    intent: PlayerPoseIntent,
) -> PoseReadabilityMetrics {
    if key_pose_intent(intent) {
        metrics.key_pose_readability_score = 0.0;
    }
    metrics
}

#[derive(Clone, Copy, Debug)]
struct TransitionAwarePoseReadability {
    metrics: PoseReadabilityMetrics,
    used_transition_grace: bool,
}

fn transition_aware_pose_readability(
    mut metrics: PoseReadabilityMetrics,
    current_intent: PlayerPoseIntent,
    transition_from_key_intent: Option<PlayerPoseIntent>,
    key_intent_age_frames: u32,
    pose_temporal: &EvalPoseTemporalMetrics,
) -> TransitionAwarePoseReadability {
    if key_pose_intent(current_intent)
        && metrics.key_pose_readability_score + KEY_POSE_READABILITY_EPSILON
            >= MIN_KEY_POSE_READABILITY_SCORE
    {
        metrics.key_pose_readability_score = metrics
            .key_pose_readability_score
            .max(MIN_KEY_POSE_READABILITY_SCORE);
        return TransitionAwarePoseReadability {
            metrics,
            used_transition_grace: false,
        };
    }

    let previous_transition_intent =
        transition_from_key_intent.filter(|previous_intent| *previous_intent != current_intent);
    let transition_grace_frames =
        key_pose_transition_grace_frames(current_intent, previous_transition_intent);
    let transition_within_grace =
        previous_transition_intent.is_some() && key_intent_age_frames <= transition_grace_frames;
    let (max_transition_rotation_delta, max_transition_translation_delta) =
        key_pose_transition_temporal_limits(current_intent, previous_transition_intent);
    let transition_temporally_smooth = pose_temporal
        .max_pose_part_rotation_delta_degrees
        .is_finite()
        && pose_temporal.max_pose_part_translation_delta_m.is_finite()
        && pose_temporal.max_pose_part_rotation_delta_degrees <= max_transition_rotation_delta
        && pose_temporal.max_pose_part_translation_delta_m <= max_transition_translation_delta;
    let mut transition_readability_score = metrics.key_pose_readability_score;
    if key_pose_intent(current_intent)
        && metrics.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE
        && let Some(previous_intent) = previous_transition_intent
    {
        let previous_score = key_pose_readability_score(
            previous_intent,
            metrics.torso_pitch_degrees,
            metrics.arm_spread_degrees,
            metrics.leg_tuck_degrees,
            metrics.landing_crouch_m,
            metrics.landing_foot_forward_m,
            metrics.landing_foot_split_m,
        );
        transition_readability_score = transition_readability_score.max(previous_score);
    }
    let transition_readability_floor =
        key_pose_transition_readability_floor(current_intent, previous_transition_intent);
    if key_pose_intent(current_intent)
        && metrics.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE
        && transition_within_grace
        && transition_temporally_smooth
        && transition_readability_score >= transition_readability_floor
    {
        metrics.key_pose_readability_score = MIN_KEY_POSE_READABILITY_SCORE;
        return TransitionAwarePoseReadability {
            metrics,
            used_transition_grace: true,
        };
    }
    TransitionAwarePoseReadability {
        metrics,
        used_transition_grace: false,
    }
}

fn key_pose_transition_readability_floor(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> f32 {
    if air_brake_release_transition(current_intent, previous_intent) {
        KEY_POSE_AIR_BRAKE_RELEASE_TRANSITION_READABILITY_FLOOR
    } else if landing_flip_transition(current_intent, previous_intent) {
        KEY_POSE_LANDING_FLIP_TRANSITION_READABILITY_FLOOR
    } else if landing_absorb_transition(current_intent, previous_intent)
        || landing_release_transition(current_intent, previous_intent)
    {
        KEY_POSE_LANDING_RELEASE_TRANSITION_READABILITY_FLOOR
    } else {
        KEY_POSE_TRANSITION_READABILITY_FLOOR
    }
}

fn key_pose_transition_temporal_limits(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> (f32, f32) {
    if landing_flip_transition(current_intent, previous_intent)
        || landing_absorb_transition(current_intent, previous_intent)
        || landing_release_transition(current_intent, previous_intent)
    {
        (
            KEY_POSE_LANDING_TRANSITION_MAX_ROTATION_DELTA_DEGREES,
            KEY_POSE_LANDING_TRANSITION_MAX_TRANSLATION_DELTA_M,
        )
    } else {
        (
            KEY_POSE_TRANSITION_MAX_ROTATION_DELTA_DEGREES,
            KEY_POSE_TRANSITION_MAX_TRANSLATION_DELTA_M,
        )
    }
}

fn key_pose_transition_grace_frames(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> u32 {
    if glide_to_dive_transition(current_intent, previous_intent) {
        KEY_POSE_EXTENDED_TRANSITION_GRACE_FRAMES
    } else if landing_flip_transition(current_intent, previous_intent)
        || landing_absorb_transition(current_intent, previous_intent)
        || landing_release_transition(current_intent, previous_intent)
    {
        KEY_POSE_LANDING_TRANSITION_GRACE_FRAMES
    } else {
        KEY_POSE_TRANSITION_GRACE_FRAMES
    }
}

fn glide_to_dive_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> bool {
    current_intent == PlayerPoseIntent::Diving && previous_intent == Some(PlayerPoseIntent::Gliding)
}

fn air_brake_release_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> bool {
    current_intent == PlayerPoseIntent::Gliding
        && previous_intent == Some(PlayerPoseIntent::AirBrake)
}

fn landing_flip_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> bool {
    current_intent == PlayerPoseIntent::LandingAnticipation
        && matches!(
            previous_intent,
            Some(PlayerPoseIntent::Diving | PlayerPoseIntent::Gliding | PlayerPoseIntent::Falling)
        )
}

fn landing_absorb_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> bool {
    current_intent == PlayerPoseIntent::LandingRecovery
        && previous_intent == Some(PlayerPoseIntent::LandingAnticipation)
}

fn landing_release_transition(
    current_intent: PlayerPoseIntent,
    previous_intent: Option<PlayerPoseIntent>,
) -> bool {
    current_intent == PlayerPoseIntent::Gliding
        && previous_intent == Some(PlayerPoseIntent::LandingAnticipation)
}

fn authored_pose_readability_metrics<'a>(
    mut authored_players: impl Iterator<Item = &'a AuthoredPlayerAnimation>,
    mut metrics: PoseReadabilityMetrics,
    intent: PlayerPoseIntent,
    speed_mps: f32,
    input: FlightInput,
) -> PoseReadabilityMetrics {
    if !key_pose_intent(intent) {
        return metrics;
    }

    let desired_clip = authored_player_clip_for_pose_intent_with_input(intent, speed_mps, input);
    let Some(first_authored_player) = authored_players.next() else {
        metrics.key_pose_readability_score = 0.0;
        return metrics;
    };
    let current_clip_matches = first_authored_player.current == desired_clip
        && authored_players.all(|authored_player| authored_player.current == desired_clip);

    if !current_clip_matches {
        metrics.key_pose_readability_score = 0.0;
    }
    metrics
}

fn key_pose_intent(intent: PlayerPoseIntent) -> bool {
    matches!(
        intent,
        PlayerPoseIntent::Launching
            | PlayerPoseIntent::Falling
            | PlayerPoseIntent::Gliding
            | PlayerPoseIntent::AirTurn
            | PlayerPoseIntent::Diving
            | PlayerPoseIntent::AirBrake
            | PlayerPoseIntent::LandingAnticipation
            | PlayerPoseIntent::LandingRecovery
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authored_assets::AuthoredPlayerClip;
    use nau_engine::movement::{FlightInput, FlightMode};

    #[test]
    fn authored_animation_sample_metrics_use_available_diagnostics() {
        let metrics = authored_animation_sample_metrics(Some(&AuthoredAnimationDiagnostics {
            player_count: 2,
            current_clip: Some(AuthoredPlayerClip::Dive),
            desired_clip: Some(AuthoredPlayerClip::Glide),
            transition_from_clip: Some(AuthoredPlayerClip::Glide),
            transition_to_clip: Some(AuthoredPlayerClip::Dive),
            transition_active: true,
            transition_elapsed_ms: 64,
            transition_duration_ms: 190,
            transition_progress: 0.34,
            transition_class_label: "traversal_blend",
        }));

        assert_eq!(
            metrics,
            AuthoredAnimationSampleMetrics {
                current_clip_label: "dive",
                desired_clip_label: "glide",
                player_count: 2,
                transition_duration_ms: 190,
                transition_from_clip_label: "glide",
                transition_to_clip_label: "dive",
                transition_active: true,
                transition_elapsed_ms: 64,
                transition_progress: 0.34,
                transition_class_label: "traversal_blend",
            }
        );
        assert_eq!(
            authored_animation_sample_metrics(None),
            AuthoredAnimationSampleMetrics {
                current_clip_label: "none",
                desired_clip_label: "none",
                player_count: 0,
                transition_duration_ms: 0,
                transition_from_clip_label: "none",
                transition_to_clip_label: "none",
                transition_active: false,
                transition_elapsed_ms: 0,
                transition_progress: 0.0,
                transition_class_label: "none",
            }
        );
    }

    #[test]
    fn visible_generated_pose_metrics_use_rendered_part_transforms() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(24.0, -2.0, -30.0),
            FlightInput {
                right: true,
                ..default()
            },
            30.0,
        );
        let parts = [
            (
                CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY),
                Transform::from_rotation(Quat::from_rotation_z(-0.20)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Arm(Side::Left),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_z(-1.0)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Arm(Side::Right),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_z(1.0)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Leg(Side::Left),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_x(0.65)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Leg(Side::Right),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_x(0.65)),
                Visibility::Inherited,
            ),
        ];

        let metrics = visible_generated_pose_readability_metrics(
            parts
                .iter()
                .map(|(part, transform, visibility)| (part, transform, visibility)),
            context,
        )
        .expect("visible generated pose metrics");

        assert!(metrics.lateral_lean_degrees > 8.0);
        assert!(metrics.arm_spread_degrees > 100.0);
    }

    #[test]
    fn visible_generated_pose_metrics_include_scarf_motion_without_core_count() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(8.0, -8.0, -34.0),
            FlightInput::default(),
            30.0,
        );
        let scarf_anchor_base = Vec3::new(0.0, 1.25, 0.02);
        let scarf_tail_base = Vec3::new(0.0, 1.08, 0.05);
        let parts = [
            (
                CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY),
                Transform::from_rotation(Quat::from_rotation_x(-0.32)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Arm(Side::Left),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_z(1.08)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Arm(Side::Right),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_z(-1.08)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Leg(Side::Left),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_x(0.2)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Leg(Side::Right),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_x(0.2)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Scarf(ScarfSegment::Anchor),
                    scarf_anchor_base,
                    Quat::IDENTITY,
                ),
                Transform {
                    translation: scarf_anchor_base + Vec3::new(0.01, 0.0, 0.04),
                    rotation: Quat::from_rotation_x(0.04),
                    ..default()
                },
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Scarf(ScarfSegment::Trail),
                    scarf_tail_base,
                    Quat::IDENTITY,
                ),
                Transform {
                    translation: scarf_tail_base + Vec3::new(-0.09, 0.0, 0.38),
                    rotation: Quat::from_rotation_x(0.32),
                    ..default()
                },
                Visibility::Inherited,
            ),
        ];

        let part_set = visible_generated_pose_part_set(
            parts
                .iter()
                .map(|(part, transform, visibility)| (part, transform, visibility)),
        );
        let metrics = part_set
            .readability_metrics(context)
            .expect("visible generated pose metrics");

        assert_eq!(part_set.part_count(), 5);
        assert!(metrics.key_pose_readability_score > 0.9);
        assert!(metrics.scarf_stream_m > 0.37);
        assert!(metrics.scarf_lateral_sway_m > 0.08);
        assert!(metrics.scarf_tail_flex_degrees > 15.0);
    }

    #[test]
    fn visible_scarf_without_core_parts_does_not_satisfy_pose_metrics() {
        let scarf_tail_base = Vec3::new(0.0, 1.08, 0.05);
        let parts = [(
            CharacterPart::new(
                CharacterPartRole::Scarf(ScarfSegment::Trail),
                scarf_tail_base,
                Quat::IDENTITY,
            ),
            Transform {
                translation: scarf_tail_base + Vec3::new(0.12, 0.0, 0.4),
                rotation: Quat::from_rotation_x(0.35),
                ..default()
            },
            Visibility::Inherited,
        )];

        let part_set = visible_generated_pose_part_set(
            parts
                .iter()
                .map(|(part, transform, visibility)| (part, transform, visibility)),
        );

        assert_eq!(part_set.part_count(), 0);
        assert!(
            part_set
                .readability_metrics(PlayerPoseContext::new(
                    FlightMode::Gliding,
                    Vec3::NEG_Z,
                    FlightInput::default(),
                    10.0,
                ))
                .is_none()
        );
    }

    #[test]
    fn hidden_generated_parts_do_not_satisfy_visible_pose_metrics() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -30.0),
            FlightInput::default(),
            30.0,
        );
        let parts = [(
            CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY),
            Transform::default(),
            Visibility::Hidden,
        )];

        assert!(
            visible_generated_pose_readability_metrics(
                parts
                    .iter()
                    .map(|(part, transform, visibility)| (part, transform, visibility)),
                context,
            )
            .is_none()
        );
    }

    #[test]
    fn generated_landing_crouch_uses_pose_delta_and_rotation_not_leg_base_height() {
        let context = PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -4.0, -18.0),
            FlightInput::default(),
            4.0,
        );
        let left_leg_base = Vec3::new(-0.22, 0.28, 0.0);
        let right_leg_base = Vec3::new(0.22, 0.28, 0.0);
        let parts = [
            (
                CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY),
                Transform::from_rotation(Quat::from_rotation_x(0.52)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Arm(Side::Left),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_z(-0.9)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Arm(Side::Right),
                    Vec3::ZERO,
                    Quat::IDENTITY,
                ),
                Transform::from_rotation(Quat::from_rotation_z(0.9)),
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Leg(Side::Left),
                    left_leg_base,
                    Quat::IDENTITY,
                ),
                Transform {
                    translation: left_leg_base,
                    rotation: Quat::from_rotation_x(-1.1),
                    ..default()
                },
                Visibility::Inherited,
            ),
            (
                CharacterPart::new(
                    CharacterPartRole::Leg(Side::Right),
                    right_leg_base,
                    Quat::IDENTITY,
                ),
                Transform {
                    translation: right_leg_base,
                    rotation: Quat::from_rotation_x(-1.1),
                    ..default()
                },
                Visibility::Inherited,
            ),
        ];

        let metrics = visible_generated_pose_readability_metrics(
            parts
                .iter()
                .map(|(part, transform, visibility)| (part, transform, visibility)),
            context,
        )
        .expect("visible generated pose metrics");

        assert!(metrics.landing_crouch_m > 0.07);
        assert!(metrics.landing_crouch_m < 0.08);
        assert!(metrics.key_pose_readability_score < 0.1);
    }

    #[test]
    fn visible_authored_pose_metrics_use_tagged_node_transforms() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -30.0),
            FlightInput::default(),
            30.0,
        );
        let left_leg_base = Vec3::new(-0.22, 0.32, 0.02);
        let right_leg_base = Vec3::new(0.22, 0.32, 0.02);
        let scarf_anchor_base = Vec3::new(0.0, 1.24, 0.04);
        let scarf_tail_base = Vec3::new(0.0, 1.08, 0.03);
        let nodes = [
            (
                AuthoredPlayerPoseNode::new(CharacterPart::new(
                    CharacterPartRole::Torso,
                    Vec3::new(0.0, 1.08, 0.0),
                    Quat::IDENTITY,
                )),
                Transform::from_rotation(Quat::from_rotation_x(-0.32)),
            ),
            (
                AuthoredPlayerPoseNode::new(CharacterPart::new(
                    CharacterPartRole::Arm(Side::Left),
                    Vec3::new(-0.55, 1.17, 0.0),
                    Quat::IDENTITY,
                )),
                Transform::from_rotation(Quat::from_rotation_z(1.08)),
            ),
            (
                AuthoredPlayerPoseNode::new(CharacterPart::new(
                    CharacterPartRole::Arm(Side::Right),
                    Vec3::new(0.55, 1.17, 0.0),
                    Quat::IDENTITY,
                )),
                Transform::from_rotation(Quat::from_rotation_z(-1.08)),
            ),
            (
                AuthoredPlayerPoseNode::new(CharacterPart::new(
                    CharacterPartRole::Leg(Side::Left),
                    left_leg_base,
                    Quat::IDENTITY,
                )),
                Transform {
                    translation: left_leg_base,
                    rotation: Quat::from_rotation_x(0.2),
                    ..default()
                },
            ),
            (
                AuthoredPlayerPoseNode::new(CharacterPart::new(
                    CharacterPartRole::Leg(Side::Right),
                    right_leg_base,
                    Quat::IDENTITY,
                )),
                Transform {
                    translation: right_leg_base,
                    rotation: Quat::from_rotation_x(0.2),
                    ..default()
                },
            ),
            (
                AuthoredPlayerPoseNode::new(CharacterPart::new(
                    CharacterPartRole::Scarf(ScarfSegment::Anchor),
                    scarf_anchor_base,
                    Quat::IDENTITY,
                )),
                Transform {
                    translation: scarf_anchor_base,
                    rotation: Quat::from_rotation_z(0.02),
                    ..default()
                },
            ),
            (
                AuthoredPlayerPoseNode::new(CharacterPart::new(
                    CharacterPartRole::Scarf(ScarfSegment::Trail),
                    scarf_tail_base,
                    Quat::IDENTITY,
                )),
                Transform {
                    translation: scarf_tail_base + Vec3::new(0.07, 0.0, 0.36),
                    rotation: Quat::from_rotation_x(0.31),
                    ..default()
                },
            ),
        ];
        let global_transforms = nodes
            .iter()
            .map(|(_, transform)| GlobalTransform::from(*transform))
            .collect::<Vec<_>>();

        let metrics = visible_authored_pose_readability_metrics(
            nodes.iter().zip(global_transforms.iter()).map(
                |((node, transform), global_transform)| {
                    (node, transform, global_transform, None, None)
                },
            ),
            context,
        )
        .expect("visible authored pose metrics");

        assert!(metrics.torso_pitch_degrees > 16.0);
        assert!(metrics.arm_spread_degrees > 120.0);
        assert!(metrics.key_pose_readability_score > 0.9);
        assert!(metrics.scarf_stream_m > 0.35);
        assert!(metrics.scarf_lateral_sway_m > 0.06);
        assert!(metrics.scarf_tail_flex_degrees > 15.0);
    }

    #[test]
    fn hidden_authored_pose_nodes_do_not_satisfy_visible_pose_metrics() {
        let node = AuthoredPlayerPoseNode::new(CharacterPart::new(
            CharacterPartRole::Torso,
            Vec3::ZERO,
            Quat::IDENTITY,
        ));
        let transform = Transform::default();
        let global_transform = GlobalTransform::from(transform);
        let visibility = Visibility::Inherited;
        let inherited_visibility = InheritedVisibility::HIDDEN;

        let parts = visible_authored_pose_part_set(std::iter::once((
            &node,
            &transform,
            &global_transform,
            Some(&visibility),
            Some(&inherited_visibility),
        )));

        assert_eq!(parts.part_count(), 0);
        assert!(
            parts
                .readability_metrics(PlayerPoseContext::new(
                    FlightMode::Gliding,
                    Vec3::NEG_Z,
                    FlightInput::default(),
                    10.0,
                ))
                .is_none()
        );
    }

    #[test]
    fn missing_visible_authored_nodes_fail_key_pose_readability() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -30.0),
            FlightInput::default(),
            30.0,
        );
        let metrics = pose_readability_metrics(context, 0.0);
        assert!(metrics.key_pose_readability_score > 0.9);

        let missing = missing_visible_authored_pose_metrics(metrics, context.intent());

        assert_eq!(missing.key_pose_readability_score, 0.0);
    }

    #[test]
    fn missing_visible_authored_nodes_fail_air_turn_readability() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(24.0, -2.0, -30.0),
            FlightInput {
                right: true,
                ..default()
            },
            30.0,
        );
        assert_eq!(context.intent(), PlayerPoseIntent::AirTurn);

        let metrics = pose_readability_metrics(context, 0.0);
        assert!(metrics.key_pose_readability_score > 0.9);

        let missing = missing_visible_authored_pose_metrics(metrics, context.intent());

        assert_eq!(missing.key_pose_readability_score, 0.0);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_previous_key_pose_shape() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 5.0,
            arm_spread_degrees: 155.0,
            leg_tuck_degrees: 25.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.5,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::Gliding,
                5.0,
                155.0,
                25.0,
                0.0,
                0.0,
                0.0,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Gliding,
            Some(PlayerPoseIntent::AirBrake),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 18.0,
                max_pose_part_translation_delta_m: 0.03,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(adjusted.metrics.key_pose_readability_score >= MIN_KEY_POSE_READABILITY_SCORE);
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_preserves_strong_current_score() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 16.0,
            arm_spread_degrees: 120.0,
            leg_tuck_degrees: 20.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.5,
            key_pose_readability_score: 1.0,
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Gliding,
            None,
            30,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 4.0,
                max_pose_part_translation_delta_m: 0.01,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert_eq!(adjusted.metrics.key_pose_readability_score, 1.0);
        assert!(!adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_rejects_previous_key_pose_without_temporal_evidence() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 5.0,
            arm_spread_degrees: 155.0,
            leg_tuck_degrees: 25.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.5,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::Gliding,
                5.0,
                155.0,
                25.0,
                0.0,
                0.0,
                0.0,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Gliding,
            Some(PlayerPoseIntent::AirBrake),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: f32::NAN,
                max_pose_part_translation_delta_m: f32::NAN,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(adjusted.metrics.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(!adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_rejects_unreadable_previous_shape() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 1.0,
            arm_spread_degrees: 40.0,
            leg_tuck_degrees: 0.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: 0.1,
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Gliding,
            Some(PlayerPoseIntent::AirBrake),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 20.0,
                max_pose_part_translation_delta_m: 0.03,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(adjusted.metrics.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(!adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_smooth_in_between_key_pose() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 39.0,
            arm_spread_degrees: 134.0,
            leg_tuck_degrees: 22.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.4,
            key_pose_readability_score: 0.72,
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Diving,
            Some(PlayerPoseIntent::AirBrake),
            KEY_POSE_TRANSITION_GRACE_FRAMES,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 48.0,
                max_pose_part_translation_delta_m: 0.04,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert_eq!(
            adjusted.metrics.key_pose_readability_score,
            MIN_KEY_POSE_READABILITY_SCORE
        );
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_extended_glide_to_dive_blend() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 58.29,
            arm_spread_degrees: 165.38,
            leg_tuck_degrees: 62.72,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.07,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::Diving,
                58.29,
                165.38,
                62.72,
                0.0,
                0.0,
                0.0,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Diving,
            Some(PlayerPoseIntent::Gliding),
            KEY_POSE_EXTENDED_TRANSITION_GRACE_FRAMES,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 10.34,
                max_pose_part_translation_delta_m: 0.05,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert_eq!(
            adjusted.metrics.key_pose_readability_score,
            MIN_KEY_POSE_READABILITY_SCORE
        );
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_bounded_air_brake_release_to_glide() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 1.47,
            arm_spread_degrees: 162.0,
            leg_tuck_degrees: 16.34,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.04,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::Gliding,
                1.47,
                162.0,
                16.34,
                0.0,
                0.0,
                0.0,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Gliding,
            Some(PlayerPoseIntent::AirBrake),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 24.34,
                max_pose_part_translation_delta_m: 0.025,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert_eq!(
            adjusted.metrics.key_pose_readability_score,
            MIN_KEY_POSE_READABILITY_SCORE
        );
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_bounded_dive_to_landing_flip() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 15.4,
            arm_spread_degrees: 163.7,
            leg_tuck_degrees: 51.9,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.159,
            landing_foot_forward_m: 0.40,
            landing_foot_split_m: 0.20,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::LandingAnticipation,
                15.4,
                163.7,
                51.9,
                0.159,
                0.40,
                0.20,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::LandingAnticipation,
            Some(PlayerPoseIntent::Diving),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 54.0,
                max_pose_part_translation_delta_m: 0.18,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert_eq!(
            adjusted.metrics.key_pose_readability_score,
            MIN_KEY_POSE_READABILITY_SCORE
        );
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_bounded_glide_to_landing_flip() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 28.25,
            arm_spread_degrees: 149.72,
            leg_tuck_degrees: 62.27,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.192,
            landing_foot_forward_m: 0.49,
            landing_foot_split_m: 0.344,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::LandingAnticipation,
                28.25,
                149.72,
                62.27,
                0.192,
                0.49,
                0.344,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::LandingAnticipation,
            Some(PlayerPoseIntent::Gliding),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 56.0,
                max_pose_part_translation_delta_m: 0.20,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert_eq!(
            adjusted.metrics.key_pose_readability_score,
            MIN_KEY_POSE_READABILITY_SCORE
        );
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_bounded_landing_absorb() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 68.0,
            arm_spread_degrees: 120.0,
            leg_tuck_degrees: 42.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.08,
            landing_foot_forward_m: 0.35,
            landing_foot_split_m: 0.14,
            landing_recovery_flip_degrees: 68.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::LandingRecovery,
                68.0,
                120.0,
                42.0,
                0.08,
                0.35,
                0.14,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::LandingRecovery,
            Some(PlayerPoseIntent::LandingAnticipation),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 44.0,
                max_pose_part_translation_delta_m: 0.12,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert_eq!(
            adjusted.metrics.key_pose_readability_score,
            MIN_KEY_POSE_READABILITY_SCORE
        );
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_rejects_unbounded_dive_to_landing_flip() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 15.4,
            arm_spread_degrees: 163.7,
            leg_tuck_degrees: 51.9,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.159,
            landing_foot_forward_m: 0.40,
            landing_foot_split_m: 0.20,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.0,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::LandingAnticipation,
                15.4,
                163.7,
                51.9,
                0.159,
                0.40,
                0.20,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::LandingAnticipation,
            Some(PlayerPoseIntent::Diving),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 121.0,
                max_pose_part_translation_delta_m: 0.375,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(adjusted.metrics.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(!adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_accepts_bounded_landing_release_to_glide() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 6.72,
            arm_spread_degrees: 143.76,
            leg_tuck_degrees: 19.54,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.5,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::Gliding,
                6.72,
                143.76,
                19.54,
                0.0,
                0.0,
                0.0,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Gliding,
            Some(PlayerPoseIntent::LandingAnticipation),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 49.49,
                max_pose_part_translation_delta_m: 0.20,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(raw.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert_eq!(
            adjusted.metrics.key_pose_readability_score,
            MIN_KEY_POSE_READABILITY_SCORE
        );
        assert!(adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_rejects_unbounded_landing_release_to_glide() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 6.72,
            arm_spread_degrees: 143.76,
            leg_tuck_degrees: 19.54,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.5,
            key_pose_readability_score: key_pose_readability_score(
                PlayerPoseIntent::Gliding,
                6.72,
                143.76,
                19.54,
                0.0,
                0.0,
                0.0,
            ),
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Gliding,
            Some(PlayerPoseIntent::LandingAnticipation),
            1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 121.0,
                max_pose_part_translation_delta_m: 0.285,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(adjusted.metrics.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(!adjusted.used_transition_grace);
    }

    #[test]
    fn transition_aware_pose_readability_rejects_persistent_unreadable_key_pose() {
        let raw = PoseReadabilityMetrics {
            torso_pitch_degrees: 39.0,
            arm_spread_degrees: 134.0,
            leg_tuck_degrees: 22.0,
            lateral_lean_degrees: 0.0,
            signed_lateral_lean_degrees: 0.0,
            grounded_stride_foot_travel_m: 0.0,
            grounded_stride_leg_opposition_degrees: 0.0,
            landing_crouch_m: 0.0,
            landing_foot_forward_m: 0.0,
            landing_recovery_flip_degrees: 0.0,
            wing_airflow_strength: 0.4,
            key_pose_readability_score: 0.72,
            ..default()
        };

        let adjusted = transition_aware_pose_readability(
            raw,
            PlayerPoseIntent::Diving,
            Some(PlayerPoseIntent::AirBrake),
            KEY_POSE_TRANSITION_GRACE_FRAMES + 1,
            &EvalPoseTemporalMetrics {
                visible_pose_part_count: 5,
                max_pose_part_rotation_delta_degrees: 8.0,
                max_pose_part_translation_delta_m: 0.02,
                min_pose_limb_clearance_m: 0.12,
                max_pose_limb_penetration_m: 0.0,
                max_pose_joint_gap_m: 0.0,
                pose_joint_gap_samples: 1,
            },
        );

        assert!(adjusted.metrics.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE);
        assert!(!adjusted.used_transition_grace);
    }

    #[test]
    fn visible_pose_temporal_state_reports_key_pose_part_deltas() {
        let mut state = VisiblePoseTemporalState::default();
        let first = visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO);
        let second =
            visible_pose_part_set(Quat::from_rotation_z(std::f32::consts::PI), Vec3::Y * 0.7);

        state.observe_frame(
            0,
            PlayerPoseIntent::Gliding,
            first,
            VisiblePoseAttachmentSet::default(),
        );
        let initial = state.take_sample_metrics();
        state.observe_frame(
            5,
            PlayerPoseIntent::Gliding,
            second,
            VisiblePoseAttachmentSet::default(),
        );
        let changed = state.take_sample_metrics();

        assert_eq!(initial.visible_pose_part_count, 6);
        assert!(initial.max_pose_part_rotation_delta_degrees.is_nan());
        assert_eq!(changed.visible_pose_part_count, 6);
        assert!(changed.max_pose_part_rotation_delta_degrees > 170.0);
        assert!(changed.max_pose_part_translation_delta_m > 0.69);
    }

    #[test]
    fn visible_pose_temporal_state_retains_transition_source_through_grace_window() {
        let mut state = VisiblePoseTemporalState::default();
        let parts = visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO);

        state.observe_frame(
            0,
            PlayerPoseIntent::AirBrake,
            parts,
            VisiblePoseAttachmentSet::default(),
        );
        assert_eq!(state.transition_from_key_intent(), None);

        state.observe_frame(
            1,
            PlayerPoseIntent::Gliding,
            parts,
            VisiblePoseAttachmentSet::default(),
        );
        assert_eq!(
            state.transition_from_key_intent(),
            Some(PlayerPoseIntent::AirBrake)
        );
        assert_eq!(state.key_intent_age_frames(), 1);

        for frame in 2..=5 {
            state.observe_frame(
                frame,
                PlayerPoseIntent::Gliding,
                parts,
                VisiblePoseAttachmentSet::default(),
            );
        }
        assert_eq!(
            state.transition_from_key_intent(),
            Some(PlayerPoseIntent::AirBrake)
        );
        assert_eq!(state.key_intent_age_frames(), 5);
    }

    #[test]
    fn visible_pose_temporal_state_only_emits_deltas_for_key_poses() {
        let mut state = VisiblePoseTemporalState::default();
        let first = visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO);
        let second = visible_pose_part_set(Quat::from_rotation_z(std::f32::consts::PI), Vec3::Y);

        state.observe_frame(
            0,
            PlayerPoseIntent::GroundedStride,
            first,
            VisiblePoseAttachmentSet::default(),
        );
        state.observe_frame(
            5,
            PlayerPoseIntent::GroundedStride,
            second,
            VisiblePoseAttachmentSet::default(),
        );
        let changed = state.take_sample_metrics();

        assert_eq!(changed.visible_pose_part_count, 6);
        assert!(changed.max_pose_part_rotation_delta_degrees.is_nan());
        assert!(changed.max_pose_part_translation_delta_m.is_nan());
    }

    #[test]
    fn visible_pose_temporal_state_tracks_air_turn_as_key_pose() {
        let mut state = VisiblePoseTemporalState::default();
        let first = visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO);
        let second =
            visible_pose_part_set(Quat::from_rotation_z(std::f32::consts::PI), Vec3::Y * 0.7);

        state.observe_frame(
            0,
            PlayerPoseIntent::AirTurn,
            first,
            VisiblePoseAttachmentSet::default(),
        );
        state.take_sample_metrics();
        state.observe_frame(
            5,
            PlayerPoseIntent::AirTurn,
            second,
            VisiblePoseAttachmentSet::default(),
        );
        let changed = state.take_sample_metrics();

        assert_eq!(changed.visible_pose_part_count, 6);
        assert!(changed.max_pose_part_rotation_delta_degrees > 170.0);
        assert!(changed.max_pose_part_translation_delta_m > 0.69);
    }

    #[test]
    fn visible_pose_temporal_state_keeps_inter_sample_max_delta() {
        let mut state = VisiblePoseTemporalState::default();

        state.observe_frame(
            0,
            PlayerPoseIntent::Gliding,
            visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO),
            VisiblePoseAttachmentSet::default(),
        );
        state.observe_frame(
            1,
            PlayerPoseIntent::Gliding,
            visible_pose_part_set(Quat::from_rotation_z(std::f32::consts::PI), Vec3::Y),
            VisiblePoseAttachmentSet::default(),
        );
        state.observe_frame(
            2,
            PlayerPoseIntent::Gliding,
            visible_pose_part_set(Quat::from_rotation_z(0.05), Vec3::ZERO),
            VisiblePoseAttachmentSet::default(),
        );

        let metrics = state.take_sample_metrics();
        assert_eq!(metrics.visible_pose_part_count, 6);
        assert!(metrics.max_pose_part_rotation_delta_degrees > 170.0);
        assert!(metrics.max_pose_part_translation_delta_m > 0.9);
    }

    #[test]
    fn visible_pose_temporal_state_reports_limb_clearance() {
        let mut state = VisiblePoseTemporalState::default();
        let readable_parts = visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO);
        let mut overlapping_parts = readable_parts;
        overlapping_parts.left_arm = Some(VisiblePosePartTransform {
            translation: Vec3::new(-0.08, 1.08, 0.0),
            global_translation: Vec3::new(-0.08, 1.08, 0.0),
            base_delta: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        });

        state.observe_frame(
            0,
            PlayerPoseIntent::Gliding,
            readable_parts,
            VisiblePoseAttachmentSet::default(),
        );
        let readable = state.take_sample_metrics();
        state.observe_frame(
            1,
            PlayerPoseIntent::Gliding,
            overlapping_parts,
            VisiblePoseAttachmentSet::default(),
        );
        let overlapping = state.take_sample_metrics();

        assert!(readable.min_pose_limb_clearance_m > 0.04);
        assert!(overlapping.min_pose_limb_clearance_m < 0.0);
        assert!(overlapping.max_pose_limb_penetration_m > 0.0);
    }

    #[test]
    fn visible_pose_temporal_state_reports_joint_gaps_from_markers() {
        let mut state = VisiblePoseTemporalState::default();
        let parts = visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO);
        let attachments = VisiblePoseAttachmentSet {
            neck: Some(Vec3::new(0.0, 1.80, -0.02)),
            left_shoulder: Some(Vec3::new(-0.55, 1.17, 0.0)),
            right_shoulder: Some(Vec3::new(0.55, 1.17, 0.0)),
            left_hip: Some(Vec3::new(-0.22, 0.32, 0.02)),
            right_hip: Some(Vec3::new(0.22, 0.32, 0.02)),
            ..default()
        };
        state.observe_frame(0, PlayerPoseIntent::Gliding, parts, attachments);
        let connected = state.take_sample_metrics();

        let detached = VisiblePoseAttachmentSet {
            left_shoulder: Some(Vec3::new(-0.10, 1.17, 0.0)),
            ..attachments
        };
        state.observe_frame(1, PlayerPoseIntent::Gliding, parts, detached);
        let gapped = state.take_sample_metrics();

        assert!(connected.max_pose_joint_gap_m < 0.001);
        assert!(gapped.max_pose_joint_gap_m > 0.40);
    }

    #[test]
    fn partial_visible_pose_parts_report_partial_count_without_readability() {
        let mut parts = visible_pose_part_set(Quat::IDENTITY, Vec3::ZERO);
        parts.right_leg = None;

        assert_eq!(parts.part_count(), 5);
        assert!(
            parts
                .readability_metrics(PlayerPoseContext::new(
                    FlightMode::Gliding,
                    Vec3::NEG_Z,
                    FlightInput::default(),
                    10.0,
                ))
                .is_none()
        );
    }

    fn visible_pose_part_set(
        torso_rotation: Quat,
        left_leg_translation: Vec3,
    ) -> VisiblePosePartSet {
        let left_leg_base = Vec3::new(-0.22, 0.32, 0.02);
        VisiblePosePartSet {
            torso: Some(VisiblePosePartTransform {
                translation: Vec3::new(0.0, 1.08, 0.0),
                global_translation: Vec3::new(0.0, 1.08, 0.0),
                base_delta: Vec3::ZERO,
                rotation: torso_rotation,
            }),
            head: Some(VisiblePosePartTransform {
                translation: Vec3::new(0.0, 1.80, -0.02),
                global_translation: Vec3::new(0.0, 1.80, -0.02),
                base_delta: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            }),
            left_arm: Some(VisiblePosePartTransform {
                translation: Vec3::new(-0.55, 1.17, 0.0),
                global_translation: Vec3::new(-0.55, 1.17, 0.0),
                base_delta: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            }),
            right_arm: Some(VisiblePosePartTransform {
                translation: Vec3::new(0.55, 1.17, 0.0),
                global_translation: Vec3::new(0.55, 1.17, 0.0),
                base_delta: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            }),
            left_leg: Some(VisiblePosePartTransform {
                translation: left_leg_base + left_leg_translation,
                global_translation: left_leg_base + left_leg_translation,
                base_delta: left_leg_translation,
                rotation: Quat::IDENTITY,
            }),
            right_leg: Some(VisiblePosePartTransform {
                translation: Vec3::new(0.22, 0.32, 0.02),
                global_translation: Vec3::new(0.22, 0.32, 0.02),
                base_delta: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            }),
            scarf_anchor: None,
            scarf_tail: None,
            ..default()
        }
    }
}
