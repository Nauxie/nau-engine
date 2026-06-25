use bevy::prelude::*;

const DIRECTION_EPSILON: f32 = 0.0001;
const FIELD_PAIR_EPSILON: f32 = 0.001;

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
        let offset = position - self.center;
        offset.x.abs() <= self.half_extents.x
            && offset.y.abs() <= self.half_extents.y
            && offset.z.abs() <= self.half_extents.z
    }

    pub fn flow_vector(self) -> Vec3 {
        self.direction * self.visual_speed
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
                leading_edge
                    + Vec3::Y * (y_t * self.half_extents.y * 0.72)
                    + Vec3::Z * (x_t * self.half_extents.z * 0.72)
            }
            WindFieldKind::Updraft => {
                let base = self.center - Vec3::Y * self.half_extents.y;
                base + Vec3::X * (x_t * self.half_extents.x * 0.72)
                    + Vec3::Z * (y_t * self.half_extents.z * 0.72)
            }
        }
    }
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
        offset.x.abs() <= self.half_extents.x
            && offset.y.abs() <= self.half_extents.y
            && offset.z.abs() <= self.half_extents.z
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
    fn crosswind_is_visual_only_and_horizontal() {
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
        let field = WindField::updraft(Vec3::ZERO, Vec3::new(4.0, 8.0, 4.0), 6.0);

        for index in 0..16 {
            assert!(field.contains(field.stream_origin(index, 16)));
        }
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
