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
    kind: AerialPowerUpVisualKind,
    base_scale: f32,
    phase: f32,
    angular_speed: f32,
}

#[derive(Clone, Copy, Debug)]
enum AerialPowerUpVisualKind {
    Core,
    Ring { alignment: Quat },
}

pub(crate) fn spawn_power_up_guides(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
) {
    let ring_mesh = meshes.add(
        Torus::new(0.92, 1.0)
            .mesh()
            .minor_resolution(8)
            .major_resolution(24),
    );
    let core_mesh = meshes.add(Sphere::new(1.1));

    for (power_index, power_up) in AERIAL_POWER_UP_ROUTE.into_iter().enumerate() {
        let phase = power_index as f32 * 0.7;
        commands.spawn((
            Mesh3d(core_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(power_up.center),
            AerialPowerUpVisual {
                power_up,
                kind: AerialPowerUpVisualKind::Core,
                base_scale: 1.0,
                phase,
                angular_speed: 0.75,
            },
            Name::new(format!("{} core", power_up.name)),
        ));

        let forward = Vec3::new(
            power_up.forward_direction.x,
            0.0,
            power_up.forward_direction.z,
        )
        .normalize_or(Vec3::NEG_Z);
        let alignment = Quat::from_rotation_arc(Vec3::Y, forward);
        commands.spawn((
            Mesh3d(ring_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform {
                translation: power_up.center,
                rotation: alignment,
                scale: Vec3::splat(power_up.radius_m * 0.58),
            },
            AerialPowerUpVisual {
                power_up,
                kind: AerialPowerUpVisualKind::Ring { alignment },
                base_scale: power_up.radius_m * 0.58,
                phase,
                angular_speed: 0.55 + (power_index % 3) as f32 * 0.08,
            },
            Name::new(format!("{} ring", power_up.name)),
        ));
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
        transform.translation = guide.power_up.center;
        transform.scale = Vec3::splat(guide.base_scale * pulse);
        transform.rotation = match guide.kind {
            AerialPowerUpVisualKind::Core => Quat::IDENTITY,
            AerialPowerUpVisualKind::Ring { alignment } => {
                let wobble = Quat::from_rotation_x(spin.sin() * 0.08)
                    * Quat::from_rotation_z((spin * 0.7).cos() * 0.05);
                alignment * wobble
            }
        };
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

    fn spawn_test_power_up_guides(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
        spawn_power_up_guides(&mut commands, &mut meshes, Handle::default());
    }

    #[test]
    fn aerial_gate_guides_use_two_shared_directional_visuals_per_gate() {
        let mut app = App::new();
        app.insert_resource(Assets::<Mesh>::default())
            .add_systems(Startup, spawn_test_power_up_guides);
        app.update();

        let world = app.world_mut();
        let mut query = world.query::<(&AerialPowerUpVisual, &Mesh3d, &Transform)>();
        let mut core_mesh = None;
        let mut ring_mesh = None;
        let mut core_count = 0;
        let mut ring_count = 0;

        for (visual, mesh, transform) in query.iter(world) {
            match visual.kind {
                AerialPowerUpVisualKind::Core => {
                    core_count += 1;
                    assert_eq!(transform.translation, visual.power_up.center);
                    assert_eq!(transform.scale, Vec3::ONE);
                    assert!(core_mesh.is_none_or(|handle| handle == mesh.0.id()));
                    core_mesh = Some(mesh.0.id());
                }
                AerialPowerUpVisualKind::Ring { .. } => {
                    ring_count += 1;
                    let forward = visual.power_up.forward_direction.normalize();
                    assert!(transform.rotation.mul_vec3(Vec3::Y).dot(forward) > 0.999);
                    assert!(
                        (transform.scale.x - visual.power_up.radius_m * 0.58).abs() <= f32::EPSILON
                    );
                    assert!(ring_mesh.is_none_or(|handle| handle == mesh.0.id()));
                    ring_mesh = Some(mesh.0.id());
                }
            }
        }

        assert_eq!(core_count, AERIAL_POWER_UP_ROUTE.len());
        assert_eq!(ring_count, AERIAL_POWER_UP_ROUTE.len());
        assert_ne!(core_mesh, ring_mesh);
    }

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
