use bevy::prelude::*;
use nau_engine::environment::{AERIAL_POWER_UP_ROUTE, AerialPowerUp, apply_aerial_power_up};
use nau_engine::movement::{FlightMode, FlightState};
use std::collections::HashSet;

#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct PowerUpCollectionState {
    collected: HashSet<&'static str>,
    activations_this_frame: usize,
    total_activations: usize,
    effect_timer_secs: f32,
}

impl PowerUpCollectionState {
    pub(crate) fn begin_frame(&mut self, dt: f32) {
        self.activations_this_frame = 0;
        self.effect_timer_secs = (self.effect_timer_secs - dt.max(0.0)).max(0.0);
    }

    fn collect(&mut self, power_up: AerialPowerUp) -> bool {
        if !self.collected.insert(power_up.name) {
            return false;
        }

        self.activations_this_frame += 1;
        self.total_activations += 1;
        self.effect_timer_secs = self.effect_timer_secs.max(power_up.effect_duration_secs);
        true
    }

    pub(crate) fn is_collected(&self, power_up: AerialPowerUp) -> bool {
        self.collected.contains(power_up.name)
    }

    pub(crate) fn collected_count(&self) -> usize {
        self.collected.len()
    }

    pub(crate) fn total_count(&self) -> usize {
        AERIAL_POWER_UP_ROUTE.len()
    }

    pub(crate) fn visible_count(&self) -> usize {
        AERIAL_POWER_UP_ROUTE
            .len()
            .saturating_sub(self.collected.len())
    }

    pub(crate) fn active_effects(&self) -> usize {
        usize::from(self.effect_timer_secs > 0.0)
    }

    pub(crate) fn total_activations(&self) -> usize {
        self.total_activations
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AerialPowerUpVisual {
    power_up: AerialPowerUp,
    offset: Vec3,
    scale: f32,
    phase: f32,
    angular_speed: f32,
}

pub(crate) fn spawn_power_up_guides(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
) {
    let bar_mesh = meshes.add(Cuboid::new(5.0, 0.22, 0.22));
    let core_mesh = meshes.add(Sphere::new(1.1));
    let segments = 6;

    for (power_index, power_up) in AERIAL_POWER_UP_ROUTE.into_iter().enumerate() {
        let style_index = power_index % 3;
        commands.spawn((
            Mesh3d(core_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(power_up.center),
            AerialPowerUpVisual {
                power_up,
                offset: Vec3::ZERO,
                scale: 1.0,
                phase: power_index as f32 * 0.7,
                angular_speed: 0.75,
            },
            Name::new(format!("{} core", power_up.name)),
        ));

        for segment in 0..segments {
            let phase = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let radius = power_up.radius_m * 0.58;
            let offset = Vec3::new(phase.cos() * radius, phase.sin() * radius, 0.0);
            commands.spawn((
                Mesh3d(bar_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: power_up.center + offset,
                    rotation: Quat::from_rotation_z(phase + std::f32::consts::FRAC_PI_2),
                    scale: Vec3::splat(1.0),
                },
                AerialPowerUpVisual {
                    power_up,
                    offset,
                    scale: 1.0 + style_index as f32 * 0.08,
                    phase,
                    angular_speed: 0.55 + style_index as f32 * 0.08,
                },
                Name::new(format!("{} ring segment", power_up.name)),
            ));
        }
    }
}

pub(crate) fn update_power_up_guides(
    time: Res<Time>,
    collection: Res<PowerUpCollectionState>,
    mut guides: Query<(&AerialPowerUpVisual, &mut Transform, &mut Visibility)>,
) {
    let elapsed = time.elapsed_secs();

    for (guide, mut transform, mut visibility) in &mut guides {
        if collection.is_collected(guide.power_up) {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Inherited;
        let spin = guide.phase + elapsed * guide.angular_speed;
        let pulse = 1.0 + 0.08 * (elapsed * 3.4 + guide.phase).sin();
        transform.translation =
            guide.power_up.center + Quat::from_rotation_z(spin * 0.18).mul_vec3(guide.offset);
        transform.rotation = Quat::from_rotation_z(spin + std::f32::consts::FRAC_PI_2);
        transform.scale = Vec3::splat(guide.scale * pulse);
    }
}

pub(crate) fn collect_aerial_power_ups(
    state: &mut FlightState,
    collection: &mut PowerUpCollectionState,
) {
    if state.controller.mode == FlightMode::Grounded {
        return;
    }

    for power_up in AERIAL_POWER_UP_ROUTE {
        if !collection.is_collected(power_up) && power_up.contains(state.position) {
            state.velocity = apply_aerial_power_up(state.velocity, power_up);
            collection.collect(power_up);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::movement::FlightController;

    #[test]
    fn aerial_gate_scores_once() {
        let power_up = AERIAL_POWER_UP_ROUTE[0];
        let mut state = FlightState::new(
            power_up.center,
            Vec3::ZERO,
            FlightController {
                mode: FlightMode::Gliding,
                ..default()
            },
        );
        let mut collection = PowerUpCollectionState::default();

        collect_aerial_power_ups(&mut state, &mut collection);
        let boosted_velocity = state.velocity;
        collect_aerial_power_ups(&mut state, &mut collection);

        assert_eq!(collection.collected_count(), 1);
        assert_eq!(collection.total_activations(), 1);
        assert_eq!(state.velocity, boosted_velocity);
    }

    #[test]
    fn every_aerial_gate_is_individually_collectible() {
        let mut collection = PowerUpCollectionState::default();
        let mut state = FlightState::new(
            Vec3::ZERO,
            Vec3::ZERO,
            FlightController {
                mode: FlightMode::Gliding,
                ..default()
            },
        );

        for (index, power_up) in AERIAL_POWER_UP_ROUTE.into_iter().enumerate() {
            state.position = power_up.center;
            collect_aerial_power_ups(&mut state, &mut collection);
            assert_eq!(collection.collected_count(), index + 1, "{}", power_up.name);
        }

        assert_eq!(collection.total_count(), AERIAL_POWER_UP_ROUTE.len());
        assert_eq!(collection.total_activations(), AERIAL_POWER_UP_ROUTE.len());
    }

    #[test]
    fn grounded_gate_overlap_does_not_score() {
        let power_up = AERIAL_POWER_UP_ROUTE[0];
        let mut state = FlightState::new(power_up.center, Vec3::ZERO, FlightController::default());
        let mut collection = PowerUpCollectionState::default();

        collect_aerial_power_ups(&mut state, &mut collection);

        assert_eq!(collection.collected_count(), 0);
        assert_eq!(state.velocity, Vec3::ZERO);
    }
}
