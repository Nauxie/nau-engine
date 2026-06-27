use bevy::prelude::*;

const DIRECTION_EPSILON: f32 = 0.0001;
const FIELD_PAIR_EPSILON: f32 = 0.001;
const WIND_SOFT_EDGE_START: f32 = 0.62;
const WIND_SOFT_EDGE_END: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindFieldKind {
    Crosswind,
    Updraft,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct WindField {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub direction: Vec3,
    pub visual_speed: f32,
    pub kind: WindFieldKind,
}

impl WindField {
    pub fn crosswind(center: Vec3, half_extents: Vec3, direction: Vec3, visual_speed: f32) -> Self {
        let horizontal_direction = Vec3::new(direction.x, 0.0, direction.z);
        let direction = if horizontal_direction.length_squared() > DIRECTION_EPSILON {
            horizontal_direction.normalize()
        } else {
            Vec3::X
        };

        Self {
            center,
            half_extents,
            direction,
            visual_speed: visual_speed.max(0.0),
            kind: WindFieldKind::Crosswind,
        }
    }

    pub fn updraft(center: Vec3, half_extents: Vec3, visual_speed: f32) -> Self {
        Self {
            center,
            half_extents,
            direction: Vec3::Y,
            visual_speed: visual_speed.max(0.0),
            kind: WindFieldKind::Updraft,
        }
    }

    pub fn contains(self, position: Vec3) -> bool {
        let local_offset = self.local_offset(position);
        let extents = self.half_extents + Vec3::splat(FIELD_PAIR_EPSILON);
        local_offset.x.abs() <= extents.x
            && local_offset.y.abs() <= extents.y
            && local_offset.z.abs() <= extents.z
    }

    pub fn flow_vector(self) -> Vec3 {
        self.direction * self.visual_speed
    }

    pub fn flow_at(self, position: Vec3, elapsed_secs: f32) -> Option<WindFlowSample> {
        if !self.contains(position) {
            return None;
        }

        let local = self.normalized_local_position(position);
        let time = elapsed_secs.max(0.0);
        let field_phase = self.field_phase();
        let traveling_wave =
            (time * 0.86 - local.x * 2.35 + local.y * 0.62 - local.z * 1.18 + field_phase).sin();
        let lane_wave = (time * 1.43 + local.z * 2.1 - local.y * 1.34 + field_phase * 0.7).cos();
        let pulse_wave =
            (time * 2.18 + local.x * 3.0 + local.y * 1.6 + local.z * 0.9 + field_phase * 1.3).sin();
        let gust_cell = (time * 1.07 + local.x * 4.35 - local.z * 2.7 + field_phase * 0.43).sin()
            * (time * 0.73 + local.y * 2.45 + local.z * 3.15 - field_phase * 0.31).cos();
        let wake_wave =
            (time * 1.71 - local.x * 1.15 + local.y * 3.4 + local.z * 2.25 + field_phase).sin();
        let variation = (0.16
            + traveling_wave.abs() * 0.25
            + lane_wave.abs() * 0.13
            + pulse_wave.abs() * 0.08
            + gust_cell.abs() * 0.16
            + wake_wave.abs() * 0.07)
            .clamp(0.0, 1.0);
        let gust_strength = (0.78
            + traveling_wave * 0.15
            + lane_wave * 0.08
            + pulse_wave * 0.05
            + gust_cell * 0.16
            + wake_wave * 0.05)
            .clamp(0.42, 1.34);
        let edge_falloff = wind_soft_edge_falloff(local);
        let speed = self.visual_speed * gust_strength * edge_falloff;
        let vector = match self.kind {
            WindFieldKind::Crosswind => {
                let lateral =
                    Vec3::new(-self.direction.z, 0.0, self.direction.x).normalize_or_zero();
                let downwind_channel = (1.0 + local.x * 0.05 + gust_cell * 0.035).clamp(0.86, 1.12);
                let shear =
                    lane_wave * 0.11 + pulse_wave * 0.045 + gust_cell * 0.075 - wake_wave * 0.035;
                self.direction * (speed * downwind_channel) + lateral * (speed * shear)
            }
            WindFieldKind::Updraft => {
                let radial = Vec3::new(local.x, 0.0, local.z);
                let fallback_angle = time * 0.74 + local.y * 1.45 + field_phase + lane_wave * 0.28;
                let fallback_radial = Vec3::new(fallback_angle.cos(), 0.0, fallback_angle.sin());
                let radial_axis = if radial.length_squared() > DIRECTION_EPSILON {
                    let wobble = fallback_radial * (0.12 + lane_wave.abs() * 0.1);
                    (radial.normalize() + wobble).normalize_or_zero()
                } else {
                    fallback_radial
                };
                let tangent = Vec3::new(-radial_axis.z, 0.0, radial_axis.x).normalize_or_zero();
                let curl_pulse = 0.82
                    + (time * 1.18 + local.y * 1.8 + field_phase * 0.5).sin() * 0.18
                    + gust_cell * 0.1;
                let thermal_core =
                    (1.0 - Vec2::new(local.x, local.z).length().clamp(0.0, 1.0)).powf(0.85);
                let vertical_pulse =
                    (0.9 + thermal_core * 0.15 + gust_cell * 0.1).clamp(0.74, 1.26);
                let swirl = tangent
                    * (speed * (0.2 + variation * 0.2 + gust_cell.abs() * 0.08) * curl_pulse);
                let breath = radial_axis * (speed * (lane_wave * 0.055 + wake_wave * 0.035));
                Vec3::Y * (speed * vertical_pulse) + swirl + breath
            }
        };

        Some(WindFlowSample {
            vector,
            speed_mps: vector.length(),
            gust_strength,
            variation,
        })
    }

    pub fn stream_origin(self, index: usize, stream_count: usize) -> Vec3 {
        let stream_count = stream_count.max(1);
        let columns = (stream_count as f32).sqrt().ceil() as usize;
        let column = index % columns;
        let row = (index / columns).min(columns.saturating_sub(1));
        let x_t = centered_unit(column, columns);
        let y_t = centered_unit(row, columns);

        match self.kind {
            WindFieldKind::Crosswind => {
                let leading_edge = self.center - self.direction * self.half_extents.x;
                let lateral = Vec3::new(-self.direction.z, 0.0, self.direction.x).normalize();
                leading_edge
                    + Vec3::Y * (y_t * self.half_extents.y * 0.72)
                    + lateral * (x_t * self.half_extents.z * 0.72)
            }
            WindFieldKind::Updraft => {
                let base = self.center - Vec3::Y * self.half_extents.y;
                base + Vec3::X * (x_t * self.half_extents.x * 0.72)
                    + Vec3::Z * (y_t * self.half_extents.z * 0.72)
            }
        }
    }

    fn local_offset(self, position: Vec3) -> Vec3 {
        let offset = position - self.center;
        match self.kind {
            WindFieldKind::Crosswind => {
                let lateral = Vec3::new(-self.direction.z, 0.0, self.direction.x).normalize();
                Vec3::new(offset.dot(self.direction), offset.y, offset.dot(lateral))
            }
            WindFieldKind::Updraft => offset,
        }
    }

    fn normalized_local_position(self, position: Vec3) -> Vec3 {
        let safe_extents = self.half_extents.max(Vec3::splat(0.1));
        self.local_offset(position) / safe_extents
    }

    fn field_phase(self) -> f32 {
        (self.center.dot(Vec3::new(0.071, 0.113, -0.053)) + self.visual_speed * 0.137)
            .rem_euclid(std::f32::consts::TAU)
    }
}

fn wind_soft_edge_falloff(local: Vec3) -> f32 {
    let edge_distance = local.abs().max_element();
    let edge_fade = smoothstep(WIND_SOFT_EDGE_START, WIND_SOFT_EDGE_END, edge_distance);
    (1.0 - edge_fade * 0.38).clamp(0.56, 1.0)
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindFlowSample {
    pub vector: Vec3,
    pub speed_mps: f32,
    pub gust_strength: f32,
    pub variation: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindFlowMetrics {
    pub active_fields: usize,
    pub max_speed_mps: f32,
    pub max_variation: f32,
}

pub fn wind_flow_metrics_at(
    position: Vec3,
    elapsed_secs: f32,
    fields: impl IntoIterator<Item = WindField>,
) -> WindFlowMetrics {
    fields
        .into_iter()
        .filter_map(|field| field.flow_at(position, elapsed_secs))
        .fold(WindFlowMetrics::default(), |mut metrics, sample| {
            metrics.active_fields += 1;
            metrics.max_speed_mps = metrics.max_speed_mps.max(sample.speed_mps);
            metrics.max_variation = metrics.max_variation.max(sample.variation);
            metrics
        })
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindForceApplication {
    pub velocity: Vec3,
    pub active_fields: usize,
    pub crosswind_fields: usize,
    pub updraft_swirl_fields: usize,
    pub applied_delta: Vec3,
    pub crosswind_delta: Vec3,
    pub updraft_swirl_delta: Vec3,
    pub max_flow_speed_mps: f32,
    pub max_variation: f32,
    pub max_flow_alignment: f32,
    pub max_crosswind_flow_alignment: f32,
    pub max_updraft_swirl_flow_alignment: f32,
    pub max_flow_aligned_delta_mps: f32,
    pub max_crosswind_flow_aligned_delta_mps: f32,
    pub max_updraft_swirl_flow_aligned_delta_mps: f32,
}

impl WindForceApplication {
    pub fn applied_delta_mps(self) -> f32 {
        self.applied_delta.length()
    }

    pub fn crosswind_delta_mps(self) -> f32 {
        self.crosswind_delta.length()
    }

    pub fn updraft_swirl_delta_mps(self) -> f32 {
        self.updraft_swirl_delta.length()
    }

    pub fn for_airborne_diagnostics(self, airborne: bool) -> Self {
        if airborne {
            self
        } else {
            Self {
                velocity: self.velocity,
                ..default()
            }
        }
    }
}

pub fn apply_wind_fields(
    position: Vec3,
    velocity: Vec3,
    fields: impl IntoIterator<Item = WindField>,
    elapsed_secs: f32,
    dt: f32,
    enabled: bool,
) -> WindForceApplication {
    let mut application = WindForceApplication {
        velocity,
        ..default()
    };

    if !enabled || dt <= 0.0 {
        return application;
    }

    for field in fields {
        let Some(flow) = field.flow_at(position, elapsed_secs) else {
            continue;
        };
        let horizontal_flow = Vec3::new(flow.vector.x, 0.0, flow.vector.z);
        let horizontal_speed = horizontal_flow.length();
        if horizontal_speed <= DIRECTION_EPSILON {
            continue;
        }

        let axis = horizontal_flow / horizontal_speed;
        let current_axis_speed =
            Vec3::new(application.velocity.x, 0.0, application.velocity.z).dot(axis);
        let response_rate = wind_force_response_rate(field.kind);
        let max_step_delta = wind_force_max_step_delta(field.kind, dt);
        let axis_speed_error = horizontal_speed - current_axis_speed;
        let delta_speed =
            (axis_speed_error * response_rate * dt).clamp(-max_step_delta, max_step_delta);
        if delta_speed.abs() <= DIRECTION_EPSILON {
            continue;
        }

        let delta = axis * delta_speed;
        let correction_axis = axis * axis_speed_error.signum();
        let flow_alignment = delta
            .normalize_or_zero()
            .dot(correction_axis)
            .clamp(-1.0, 1.0);
        let flow_aligned_delta_mps = delta.dot(correction_axis).max(0.0);
        application.velocity += delta;
        application.applied_delta += delta;
        application.active_fields += 1;
        application.max_flow_speed_mps = application.max_flow_speed_mps.max(horizontal_speed);
        application.max_variation = application.max_variation.max(flow.variation);
        application.max_flow_alignment = application.max_flow_alignment.max(flow_alignment);
        application.max_flow_aligned_delta_mps = application
            .max_flow_aligned_delta_mps
            .max(flow_aligned_delta_mps);

        match field.kind {
            WindFieldKind::Crosswind => {
                application.crosswind_fields += 1;
                application.crosswind_delta += delta;
                application.max_crosswind_flow_alignment =
                    application.max_crosswind_flow_alignment.max(flow_alignment);
                application.max_crosswind_flow_aligned_delta_mps = application
                    .max_crosswind_flow_aligned_delta_mps
                    .max(flow_aligned_delta_mps);
            }
            WindFieldKind::Updraft => {
                application.updraft_swirl_fields += 1;
                application.updraft_swirl_delta += delta;
                application.max_updraft_swirl_flow_alignment = application
                    .max_updraft_swirl_flow_alignment
                    .max(flow_alignment);
                application.max_updraft_swirl_flow_aligned_delta_mps = application
                    .max_updraft_swirl_flow_aligned_delta_mps
                    .max(flow_aligned_delta_mps);
            }
        }
    }

    application
}

fn wind_force_response_rate(kind: WindFieldKind) -> f32 {
    match kind {
        WindFieldKind::Crosswind => 0.48,
        WindFieldKind::Updraft => 0.24,
    }
}

fn wind_force_max_step_delta(kind: WindFieldKind, dt: f32) -> f32 {
    let max_accel = match kind {
        WindFieldKind::Crosswind => 6.0,
        WindFieldKind::Updraft => 2.0,
    };
    max_accel * dt.max(0.0)
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct LiftField {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub lift_accel: f32,
    pub max_upward_speed: f32,
}

impl LiftField {
    pub fn updraft(
        center: Vec3,
        half_extents: Vec3,
        lift_accel: f32,
        max_upward_speed: f32,
    ) -> Self {
        Self {
            center,
            half_extents: half_extents.max(Vec3::splat(0.1)),
            lift_accel: lift_accel.max(0.0),
            max_upward_speed: max_upward_speed.max(0.0),
        }
    }

    pub fn contains(self, position: Vec3) -> bool {
        let offset = position - self.center;
        offset.x.abs() <= self.half_extents.x + FIELD_PAIR_EPSILON
            && offset.y.abs() <= self.half_extents.y + FIELD_PAIR_EPSILON
            && offset.z.abs() <= self.half_extents.z + FIELD_PAIR_EPSILON
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiftRouteNode {
    pub name: &'static str,
    pub center: Vec3,
    pub half_extents: Vec3,
    pub lift_accel: f32,
    pub max_upward_speed: f32,
    pub visual_speed: f32,
}

impl LiftRouteNode {
    pub fn lift_field(self) -> LiftField {
        LiftField::updraft(
            self.center,
            self.half_extents,
            self.lift_accel,
            self.max_upward_speed,
        )
    }

    pub fn visual_field(self) -> WindField {
        WindField::updraft(self.center, self.half_extents, self.visual_speed)
    }
}

pub const GAMEPLAY_LIFT_ROUTE: [LiftRouteNode; 2] = [
    LiftRouteNode {
        name: "near route updraft",
        center: Vec3::new(38.0, 68.0, -112.0),
        half_extents: Vec3::new(20.0, 34.0, 22.0),
        lift_accel: 28.0,
        max_upward_speed: 20.0,
        visual_speed: 12.0,
    },
    LiftRouteNode {
        name: "distant recovery updraft",
        center: Vec3::new(24.0, 74.0, -430.0),
        half_extents: Vec3::new(26.0, 42.0, 26.0),
        lift_accel: 24.0,
        max_upward_speed: 22.0,
        visual_speed: 14.0,
    },
];

pub const VISUAL_CROSSWIND_FIELD_COUNT: usize = 3;

pub fn visual_crosswind_fields() -> [WindField; VISUAL_CROSSWIND_FIELD_COUNT] {
    [
        WindField::crosswind(
            Vec3::new(20.0, 52.0, -68.0),
            Vec3::new(38.0, 24.0, 20.0),
            Vec3::X,
            10.0,
        ),
        WindField::crosswind(
            Vec3::new(30.0, 78.0, -214.0),
            Vec3::new(42.0, 26.0, 20.0),
            Vec3::new(-1.0, 0.0, 0.35),
            8.5,
        ),
        WindField::crosswind(
            GAMEPLAY_LIFT_ROUTE[0].center,
            Vec3::new(34.0, 30.0, 30.0),
            Vec3::new(0.7, 0.0, -0.4),
            8.0,
        ),
    ]
}

pub fn visual_wind_fields() -> Vec<WindField> {
    let mut fields = visual_crosswind_fields().to_vec();
    fields.extend(GAMEPLAY_LIFT_ROUTE.iter().map(|node| node.visual_field()));
    fields
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AerialPowerUp {
    pub name: &'static str,
    pub center: Vec3,
    pub radius_m: f32,
    pub forward_direction: Vec3,
    pub forward_speed_boost: f32,
    pub upward_speed_boost: f32,
    pub max_upward_speed: f32,
    pub effect_duration_secs: f32,
}

impl AerialPowerUp {
    pub fn contains(self, position: Vec3) -> bool {
        (position - self.center).length() <= self.radius_m.max(0.0)
    }
}

pub const AERIAL_POWER_UP_ROUTE: [AerialPowerUp; 3] = [
    AerialPowerUp {
        name: "midair gust gate",
        center: Vec3::new(26.0, 92.0, -126.0),
        radius_m: 24.0,
        forward_direction: Vec3::NEG_Z,
        forward_speed_boost: 7.5,
        upward_speed_boost: 5.0,
        max_upward_speed: 20.0,
        effect_duration_secs: 0.75,
    },
    AerialPowerUp {
        name: "drift boost gate",
        center: Vec3::new(32.0, 124.0, -300.0),
        radius_m: 26.0,
        forward_direction: Vec3::NEG_Z,
        forward_speed_boost: 7.0,
        upward_speed_boost: 4.0,
        max_upward_speed: 18.0,
        effect_duration_secs: 0.75,
    },
    AerialPowerUp {
        name: "recovery lift gate",
        center: Vec3::new(42.0, 114.0, -430.0),
        radius_m: 26.0,
        forward_direction: Vec3::NEG_Z,
        forward_speed_boost: 6.0,
        upward_speed_boost: 5.0,
        max_upward_speed: 18.0,
        effect_duration_secs: 0.75,
    },
];

pub fn apply_aerial_power_up(mut velocity: Vec3, power_up: AerialPowerUp) -> Vec3 {
    let forward = horizontal_or(power_up.forward_direction, Vec3::NEG_Z);
    velocity += forward * power_up.forward_speed_boost.max(0.0);

    if velocity.y < power_up.max_upward_speed {
        velocity.y =
            (velocity.y + power_up.upward_speed_boost.max(0.0)).min(power_up.max_upward_speed);
    }

    velocity
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiftApplication {
    pub velocity: Vec3,
    pub active_fields: usize,
    pub applied_delta_y: f32,
}

pub fn apply_lift_fields(
    position: Vec3,
    mut velocity: Vec3,
    fields: impl IntoIterator<Item = LiftField>,
    dt: f32,
    enabled: bool,
) -> LiftApplication {
    let mut active_fields = 0;
    let mut lift_accel = 0.0_f32;
    let mut max_upward_speed = velocity.y;

    for field in fields {
        if field.contains(position) {
            active_fields += 1;
            lift_accel += field.lift_accel;
            max_upward_speed = max_upward_speed.max(field.max_upward_speed);
        }
    }

    let applied_delta_y = if enabled && active_fields > 0 && velocity.y < max_upward_speed {
        let delta = (lift_accel * dt.max(0.0)).min(max_upward_speed - velocity.y);
        velocity.y += delta;
        delta
    } else {
        0.0
    };

    LiftApplication {
        velocity,
        active_fields,
        applied_delta_y,
    }
}

pub fn active_lift_fields_at(position: Vec3, fields: impl IntoIterator<Item = LiftField>) -> usize {
    fields
        .into_iter()
        .filter(|field| field.contains(position))
        .count()
}

fn centered_unit(index: usize, count: usize) -> f32 {
    if count <= 1 {
        0.0
    } else {
        (index as f32 / (count - 1) as f32) * 2.0 - 1.0
    }
}

fn horizontal_or(v: Vec3, fallback: Vec3) -> Vec3 {
    let horizontal = Vec3::new(v.x, 0.0, v.z);
    if horizontal.length_squared() > DIRECTION_EPSILON {
        horizontal.normalize()
    } else {
        fallback.normalize()
    }
}

pub fn visible_fields_at(position: Vec3, fields: impl IntoIterator<Item = WindField>) -> usize {
    fields
        .into_iter()
        .filter(|field| field.contains(position))
        .count()
}

pub fn readable_lift_fields_at(
    position: Vec3,
    lift_fields: impl IntoIterator<Item = LiftField>,
    visual_fields: impl IntoIterator<Item = WindField>,
) -> usize {
    let visible_updrafts = visual_fields
        .into_iter()
        .filter(|field| field.kind == WindFieldKind::Updraft && field.contains(position))
        .collect::<Vec<_>>();

    lift_fields
        .into_iter()
        .filter(|lift| {
            lift.contains(position)
                && visible_updrafts
                    .iter()
                    .any(|visual| lift_matches_visual_updraft(*lift, *visual))
        })
        .count()
}

fn lift_matches_visual_updraft(lift: LiftField, visual: WindField) -> bool {
    vec3_near(lift.center, visual.center) && vec3_near(lift.half_extents, visual.half_extents)
}

fn vec3_near(left: Vec3, right: Vec3) -> bool {
    (left - right).abs().max_element() <= FIELD_PAIR_EPSILON
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindSwayMotion {
    pub offset: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub offset_magnitude_m: f32,
}

pub fn wind_sway_motion(
    elapsed_secs: f32,
    phase: f32,
    amplitude_m: f32,
    bend_radians: f32,
    gust_speed: f32,
    wind_direction: Vec3,
) -> WindSwayMotion {
    let direction = horizontal_or(wind_direction, Vec3::X);
    let time = elapsed_secs.max(0.0);
    let amplitude = amplitude_m.max(0.0);
    let bend = bend_radians.max(0.0);
    let speed = gust_speed.max(0.0);
    let wave = (time * speed + phase).sin();
    let gust = 0.62 + 0.38 * (time * speed * 0.43 + phase * 1.7).sin();
    let strength = wave * gust.clamp(0.2, 1.0);
    let flutter = (time * speed * 1.9 + phase * 0.6).cos() * 0.12;
    let axis = Vec3::new(direction.z, 0.0, -direction.x).normalize_or_zero();
    let rotation_axis = if axis.length_squared() > DIRECTION_EPSILON {
        axis
    } else {
        Vec3::Z
    };
    let offset = direction * amplitude * strength
        + Vec3::Y * amplitude * flutter * (0.5 + strength.abs() * 0.5);
    let scale_pulse = 1.0 + strength.abs() * 0.018;

    WindSwayMotion {
        offset,
        rotation: Quat::from_axis_angle(rotation_axis, bend * strength),
        scale: Vec3::new(scale_pulse, 1.0 - strength.abs() * 0.01, scale_pulse),
        offset_magnitude_m: offset.length(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crosswind_normalizes_to_horizontal_flow() {
        let field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(4.0, 2.0, 4.0),
            Vec3::new(1.0, 1.0, 0.0),
            8.0,
        );

        assert_eq!(field.kind, WindFieldKind::Crosswind);
        assert_eq!(field.direction, Vec3::X);
        assert_eq!(field.flow_vector(), Vec3::new(8.0, 0.0, 0.0));
    }

    #[test]
    fn updraft_is_visual_and_vertical() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(2.0, 8.0, 2.0), 6.0);

        assert_eq!(field.kind, WindFieldKind::Updraft);
        assert_eq!(field.flow_vector(), Vec3::new(0.0, 6.0, 0.0));
    }

    #[test]
    fn field_contains_only_inside_bounds() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(4.0), Vec3::X, 8.0);

        assert!(field.contains(Vec3::new(4.0, 0.0, 0.0)));
        assert!(!field.contains(Vec3::new(4.1, 0.0, 0.0)));
    }

    #[test]
    fn stream_origins_stay_inside_visual_field() {
        let fields = [
            WindField::updraft(Vec3::ZERO, Vec3::new(4.0, 8.0, 4.0), 6.0),
            WindField::crosswind(
                Vec3::new(34.0, 10.0, -8.0),
                Vec3::new(18.0, 8.0, 10.0),
                Vec3::new(-1.0, 0.0, 0.35),
                7.0,
            ),
        ];

        for field in fields {
            for index in 0..16 {
                assert!(field.contains(field.stream_origin(index, 16)));
            }
        }
    }

    #[test]
    fn diagonal_crosswind_stream_paths_stay_inside_rotated_field() {
        let field = WindField::crosswind(
            Vec3::new(34.0, 10.0, -8.0),
            Vec3::new(18.0, 8.0, 10.0),
            Vec3::new(-1.0, 0.0, 0.35),
            7.0,
        );
        let path_length = field.half_extents.x * 2.0;

        for index in 0..36 {
            let origin = field.stream_origin(index, 36);
            for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let position = origin + field.direction * (progress * path_length);
                assert!(field.contains(position));
                assert!(field.flow_at(position, 0.4).is_some());
            }
        }
    }

    #[test]
    fn crosswind_stream_origins_follow_field_lateral_axis() {
        let field = WindField::crosswind(
            Vec3::ZERO,
            Vec3::new(24.0, 8.0, 24.0),
            Vec3::new(-1.0, 0.0, 0.35),
            7.0,
        );
        let lateral = Vec3::new(-field.direction.z, 0.0, field.direction.x).normalize();
        let origins = (0..16)
            .map(|index| field.stream_origin(index, 16))
            .collect::<Vec<_>>();
        let min_lateral = origins
            .iter()
            .map(|origin| origin.dot(lateral))
            .fold(f32::MAX, f32::min);
        let max_lateral = origins
            .iter()
            .map(|origin| origin.dot(lateral))
            .fold(f32::MIN, f32::max);

        assert!(max_lateral - min_lateral > 12.0);
    }

    #[test]
    fn visible_field_count_is_deterministic() {
        let near = WindField::crosswind(Vec3::ZERO, Vec3::splat(4.0), Vec3::X, 8.0);
        let far = WindField::updraft(Vec3::new(20.0, 0.0, 0.0), Vec3::splat(4.0), 6.0);

        assert_eq!(visible_fields_at(Vec3::ZERO, [near, far]), 1);
        assert_eq!(visible_fields_at(Vec3::new(20.0, 0.0, 0.0), [near, far]), 1);
        assert_eq!(visible_fields_at(Vec3::new(10.0, 0.0, 0.0), [near, far]), 0);
    }

    #[test]
    fn crosswind_dynamic_flow_stays_horizontal_and_varies() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(8.0), Vec3::X, 10.0);
        let early = field.flow_at(Vec3::new(1.0, 0.0, 2.0), 0.0).unwrap();
        let later = field.flow_at(Vec3::new(1.0, 0.0, 2.0), 1.25).unwrap();

        assert!(early.vector.y.abs() < 0.001);
        assert!(early.speed_mps > 6.0);
        assert!(early.variation > 0.15);
        assert!(early.vector.distance(later.vector) > 0.1);
        assert!(early.vector.normalize().dot(field.direction) > 0.97);
        assert!(
            (early.vector.z - later.vector.z).abs() > 0.2,
            "expected crosswind lane shear to vary over time, early={early:?}, later={later:?}"
        );
    }

    #[test]
    fn crosswind_gust_cells_vary_across_neighboring_lanes() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(10.0), Vec3::X, 10.0);
        let left_lane = field.flow_at(Vec3::new(-1.0, 1.0, -4.0), 0.8).unwrap();
        let right_lane = field.flow_at(Vec3::new(-1.0, 1.0, 4.0), 0.8).unwrap();

        assert!(left_lane.vector.y.abs() < 0.001);
        assert!(right_lane.vector.y.abs() < 0.001);
        assert!(left_lane.vector.normalize().dot(field.direction) > 0.94);
        assert!(right_lane.vector.normalize().dot(field.direction) > 0.94);
        assert!(
            (left_lane.gust_strength - right_lane.gust_strength).abs() > 0.08
                || (left_lane.variation - right_lane.variation).abs() > 0.08,
            "expected neighboring crosswind lanes to carry different gust cells, left={left_lane:?}, right={right_lane:?}"
        );
    }

    #[test]
    fn updraft_dynamic_flow_keeps_upward_bias_and_swirl() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0);
        let position = Vec3::new(3.0, 0.0, 2.0);
        let flow = field.flow_at(position, 0.7).unwrap();
        let later = field.flow_at(position + Vec3::Y * 5.0, 2.1).unwrap();
        let radial = Vec3::new(position.x, 0.0, position.z).normalize();
        let tangent = Vec3::new(-radial.z, 0.0, radial.x).normalize();

        assert!(flow.vector.y > 7.0);
        assert!(flow.vector.y > flow.vector.xz().length() * 2.0);
        assert!(flow.vector.xz().length() > 1.6);
        assert!(flow.vector.dot(tangent) > 1.4);
        assert!(
            flow.vector
                .xz()
                .normalize()
                .dot(later.vector.xz().normalize())
                < 0.995,
            "expected updraft curl direction to evolve with time/height, flow={flow:?}, later={later:?}"
        );
        assert!(flow.variation > 0.15);
    }

    #[test]
    fn updraft_gust_cells_vary_lift_and_swirl_across_the_column() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(10.0, 18.0, 10.0), 12.0);
        let first = field.flow_at(Vec3::new(2.5, 2.0, 1.0), 1.1).unwrap();
        let second = field.flow_at(Vec3::new(-2.5, 2.0, -1.0), 1.1).unwrap();

        assert!(first.vector.y > first.vector.xz().length() * 1.7);
        assert!(second.vector.y > second.vector.xz().length() * 1.7);
        assert!(
            (first.vector.y - second.vector.y).abs() > 0.4
                || first.vector.xz().distance(second.vector.xz()) > 0.8,
            "expected updraft gust cells to break uniform lift/swirl, first={first:?}, second={second:?}"
        );
    }

    #[test]
    fn wind_flow_softens_toward_field_edges_without_collapsing() {
        assert_eq!(wind_soft_edge_falloff(Vec3::ZERO), 1.0);
        let mid_field = wind_soft_edge_falloff(Vec3::splat(0.7));
        let near_edge = wind_soft_edge_falloff(Vec3::splat(0.96));

        assert!(mid_field < 1.0);
        assert!(near_edge < mid_field);
        assert!(near_edge > 0.36);
    }

    #[test]
    fn wind_flow_metrics_require_contained_dynamic_fields() {
        let near = WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0);
        let far = WindField::crosswind(Vec3::new(40.0, 0.0, 0.0), Vec3::splat(6.0), Vec3::X, 8.0);

        let inside = wind_flow_metrics_at(Vec3::new(2.0, 0.0, 1.0), 0.5, [near, far]);
        let outside = wind_flow_metrics_at(Vec3::new(20.0, 0.0, 0.0), 0.5, [near, far]);

        assert_eq!(inside.active_fields, 1);
        assert!(inside.max_speed_mps > 8.0);
        assert!(inside.max_variation > 0.15);
        assert_eq!(outside.active_fields, 0);
    }

    #[test]
    fn wind_force_pushes_toward_visible_crosswind_flow() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(12.0), Vec3::X, 10.0);
        let application = apply_wind_fields(Vec3::ZERO, Vec3::ZERO, [field], 0.5, 0.5, true);

        assert_eq!(application.active_fields, 1);
        assert_eq!(application.crosswind_fields, 1);
        assert!(application.crosswind_delta.x > 0.0);
        assert!(application.crosswind_delta_mps() <= 6.0);
        assert_eq!(application.velocity, application.applied_delta);
        assert!(application.max_flow_speed_mps > 6.0);
        assert!(application.max_variation > 0.15);
        assert!(application.max_flow_alignment > 0.99);
        assert!(application.max_crosswind_flow_alignment > 0.99);
        assert!(
            application.max_flow_aligned_delta_mps + 0.001 >= application.crosswind_delta_mps()
        );
        assert!(
            application.max_crosswind_flow_aligned_delta_mps + 0.001
                >= application.crosswind_delta_mps()
        );
    }

    #[test]
    fn wind_force_flow_alignment_accepts_braking_toward_field_speed() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(12.0), Vec3::X, 10.0);
        let application = apply_wind_fields(Vec3::ZERO, Vec3::X * 24.0, [field], 0.5, 0.5, true);

        assert_eq!(application.active_fields, 1);
        assert!(application.crosswind_delta.x < 0.0);
        assert!(application.max_flow_alignment > 0.99);
        assert!(application.max_crosswind_flow_alignment > 0.99);
        assert!(
            application.max_flow_aligned_delta_mps + 0.001 >= application.crosswind_delta_mps()
        );
        assert!(
            application.max_crosswind_flow_aligned_delta_mps + 0.001
                >= application.crosswind_delta_mps()
        );
    }

    #[test]
    fn wind_force_samples_updraft_swirl_without_vertical_lift() {
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(8.0, 16.0, 8.0), 12.0);
        let application = apply_wind_fields(
            Vec3::new(3.0, 0.0, 2.0),
            Vec3::ZERO,
            [field],
            0.7,
            0.5,
            true,
        );

        assert_eq!(application.active_fields, 1);
        assert_eq!(application.updraft_swirl_fields, 1);
        assert!(application.updraft_swirl_delta.xz().length() > 0.0);
        assert_eq!(application.updraft_swirl_delta.y, 0.0);
        assert_eq!(application.velocity.y, 0.0);
        assert!(application.max_updraft_swirl_flow_alignment > 0.99);
        assert!(
            application.max_updraft_swirl_flow_aligned_delta_mps + 0.001
                >= application.updraft_swirl_delta_mps()
        );
    }

    #[test]
    fn wind_force_is_disabled_on_ground() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(12.0), Vec3::X, 10.0);
        let velocity = Vec3::new(1.0, 0.0, 0.0);
        let application = apply_wind_fields(Vec3::ZERO, velocity, [field], 0.5, 0.5, false);

        assert_eq!(application.velocity, velocity);
        assert_eq!(application.active_fields, 0);
        assert_eq!(application.applied_delta, Vec3::ZERO);
    }

    #[test]
    fn wind_force_diagnostics_clear_for_final_grounded_samples() {
        let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(12.0), Vec3::X, 10.0);
        let application = apply_wind_fields(Vec3::ZERO, Vec3::ZERO, [field], 0.5, 0.5, true)
            .for_airborne_diagnostics(false);

        assert_eq!(application.active_fields, 0);
        assert_eq!(application.crosswind_fields, 0);
        assert_eq!(application.applied_delta, Vec3::ZERO);
        assert!(application.velocity.x > 0.0);
    }

    #[test]
    fn gameplay_lift_route_pairs_lift_and_visual_volumes() {
        for node in GAMEPLAY_LIFT_ROUTE {
            let lift = node.lift_field();
            let visual = node.visual_field();

            assert_eq!(lift.center, visual.center);
            assert_eq!(lift.half_extents, visual.half_extents);
            assert!(lift.contains(node.center));
            assert!(visual.contains(node.center));
            assert_eq!(visual.kind, WindFieldKind::Updraft);
        }
    }

    #[test]
    fn visual_wind_catalog_pairs_crosswinds_and_updrafts() {
        let fields = visual_wind_fields();
        let crosswinds = fields
            .iter()
            .filter(|field| field.kind == WindFieldKind::Crosswind)
            .count();
        let updrafts = fields
            .iter()
            .filter(|field| field.kind == WindFieldKind::Updraft)
            .count();

        assert_eq!(crosswinds, VISUAL_CROSSWIND_FIELD_COUNT);
        assert_eq!(updrafts, GAMEPLAY_LIFT_ROUTE.len());
        for node in GAMEPLAY_LIFT_ROUTE {
            assert!(fields.iter().any(|field| *field == node.visual_field()));
        }
    }

    #[test]
    fn near_route_updraft_has_crosswind_overlap() {
        let near_updraft = GAMEPLAY_LIFT_ROUTE[0];
        let fields = visual_wind_fields();
        let overlapping_fields = fields
            .iter()
            .filter(|field| field.contains(near_updraft.center))
            .collect::<Vec<_>>();
        let crosswind_count = overlapping_fields
            .iter()
            .filter(|field| field.kind == WindFieldKind::Crosswind)
            .count();
        let updraft_count = overlapping_fields
            .iter()
            .filter(|field| field.kind == WindFieldKind::Updraft)
            .count();
        let flow = wind_flow_metrics_at(near_updraft.center, 0.75, fields.iter().copied());

        assert!(crosswind_count >= 1);
        assert!(updraft_count >= 1);
        assert!(flow.active_fields >= 2);
        assert!(flow.max_speed_mps >= 9.0);
        assert!(flow.max_variation > 0.0);
    }

    #[test]
    fn near_route_updraft_applies_layered_crosswind_and_swirl_force() {
        let near_updraft = GAMEPLAY_LIFT_ROUTE[0];
        let application = apply_wind_fields(
            near_updraft.center,
            Vec3::ZERO,
            visual_wind_fields(),
            0.75,
            1.0 / 60.0,
            true,
        );

        assert!(application.active_fields >= 2);
        assert!(application.crosswind_fields >= 1);
        assert!(application.updraft_swirl_fields >= 1);
        assert!(application.applied_delta_mps() > 0.0);
        assert!(application.crosswind_delta_mps() > 0.0);
        assert!(application.updraft_swirl_delta_mps() > 0.0);
    }

    #[test]
    fn aerial_power_up_route_is_collectible_and_directional() {
        for power_up in AERIAL_POWER_UP_ROUTE {
            assert!(power_up.contains(power_up.center));
            assert!(power_up.radius_m >= 20.0);
            assert!(power_up.forward_speed_boost > 0.0);
            assert!(power_up.upward_speed_boost > 0.0);
            assert!(power_up.effect_duration_secs > 0.0);
        }
    }

    #[test]
    fn aerial_power_up_applies_capped_forward_and_upward_boost() {
        let power_up = AERIAL_POWER_UP_ROUTE[0];
        let boosted = apply_aerial_power_up(Vec3::new(0.0, 16.0, -12.0), power_up);

        assert!(boosted.z < -12.0);
        assert!(boosted.y > 16.0);
        assert!(boosted.y <= power_up.max_upward_speed);

        let already_fast_up = apply_aerial_power_up(Vec3::new(0.0, 28.0, -12.0), power_up);
        assert_eq!(already_fast_up.y, 28.0);
    }

    #[test]
    fn readable_lift_requires_overlapping_paired_updraft_visual() {
        let node = GAMEPLAY_LIFT_ROUTE[0];
        let lift = node.lift_field();
        let paired_visual = node.visual_field();
        let crosswind =
            WindField::crosswind(node.center, node.half_extents, Vec3::X, node.visual_speed);
        let shifted_visual =
            WindField::updraft(node.center + Vec3::X, node.half_extents, node.visual_speed);

        assert_eq!(
            readable_lift_fields_at(node.center, [lift], [paired_visual]),
            1
        );
        assert_eq!(readable_lift_fields_at(node.center, [lift], [crosswind]), 0);
        assert_eq!(
            readable_lift_fields_at(node.center, [lift], [shifted_visual]),
            0
        );
    }

    #[test]
    fn lift_field_only_applies_inside_bounds_when_enabled() {
        let field = LiftField::updraft(Vec3::ZERO, Vec3::splat(4.0), 20.0, 12.0);
        let outside = apply_lift_fields(Vec3::new(10.0, 0.0, 0.0), Vec3::ZERO, [field], 0.5, true);
        let disabled = apply_lift_fields(Vec3::ZERO, Vec3::ZERO, [field], 0.5, false);
        let active = apply_lift_fields(Vec3::ZERO, Vec3::ZERO, [field], 0.5, true);

        assert_eq!(outside.active_fields, 0);
        assert_eq!(outside.velocity, Vec3::ZERO);
        assert_eq!(disabled.active_fields, 1);
        assert_eq!(disabled.applied_delta_y, 0.0);
        assert_eq!(active.active_fields, 1);
        assert!(active.velocity.y > 0.0);
        assert!(active.velocity.y <= field.max_upward_speed);
    }

    #[test]
    fn wind_sway_motion_is_bounded_and_horizontal() {
        let motion = wind_sway_motion(1.2, 0.4, 0.35, 0.08, 1.6, Vec3::new(0.0, 4.0, -2.0));

        assert!(motion.offset.z < 0.0);
        assert!(motion.offset.x.abs() < 0.001);
        assert!(motion.offset_magnitude_m <= 0.38);
        assert!(motion.scale.x > 1.0);
        assert!(motion.scale.y <= 1.0);
    }

    #[test]
    fn wind_sway_motion_clamps_negative_inputs_to_stillness() {
        let motion = wind_sway_motion(-1.0, 0.0, -0.2, -0.1, -2.0, Vec3::ZERO);

        assert_eq!(motion.offset, Vec3::ZERO);
        assert_eq!(motion.scale, Vec3::ONE);
        assert_eq!(motion.offset_magnitude_m, 0.0);
    }
}
