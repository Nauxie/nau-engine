use bevy::prelude::Vec3;
use nau_engine::{
    environment::AERIAL_POWER_UP_ROUTE,
    movement::FlightMode,
    world::{START_POSITION, SkyRoute},
};
use std::collections::HashSet;

#[derive(Clone, Debug, Default)]
pub(crate) struct SimPowerUps {
    collected: HashSet<&'static str>,
    activations_this_frame: usize,
    pub(crate) total_activations: usize,
    effect_timer_secs: f32,
}

impl SimPowerUps {
    pub(crate) fn begin_frame(&mut self, dt: f32) {
        self.activations_this_frame = 0;
        self.effect_timer_secs = (self.effect_timer_secs - dt.max(0.0)).max(0.0);
    }

    pub(crate) fn collect(&mut self, name: &'static str, duration_secs: f32) {
        if self.collected.insert(name) {
            self.activations_this_frame += 1;
            self.total_activations += 1;
            self.effect_timer_secs = self.effect_timer_secs.max(duration_secs);
        }
    }

    pub(crate) fn is_collected(&self, name: &'static str) -> bool {
        self.collected.contains(name)
    }

    pub(crate) fn collected_count(&self) -> usize {
        self.collected.len()
    }

    pub(crate) fn visible_count(&self) -> usize {
        AERIAL_POWER_UP_ROUTE
            .len()
            .saturating_sub(self.collected.len())
    }

    pub(crate) fn active_effects(&self) -> usize {
        usize::from(self.effect_timer_secs > 0.0)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ObjectiveState {
    target_island_name: Option<&'static str>,
    pub(crate) completed_count: usize,
    pub(crate) total_count: usize,
    pub(crate) current_label: &'static str,
    pub(crate) current_distance_m: f32,
    pub(crate) complete: bool,
}

impl ObjectiveState {
    pub(crate) fn for_route(route: &SkyRoute, target_island_name: Option<&'static str>) -> Self {
        let mut state = Self {
            target_island_name,
            completed_count: 0,
            total_count: 0,
            current_label: "none",
            current_distance_m: 0.0,
            complete: false,
        };
        state.update(
            route,
            target_island_name,
            START_POSITION,
            FlightMode::Grounded,
        );
        state
    }

    pub(crate) fn update(
        &mut self,
        route: &SkyRoute,
        target_island_name: Option<&'static str>,
        position: Vec3,
        mode: FlightMode,
    ) {
        if self.target_island_name != target_island_name {
            *self = Self::for_route(route, target_island_name);
        }

        let objectives = route.route_objectives(target_island_name);
        self.total_count = objectives.len();
        self.completed_count = self.completed_count.min(objectives.len());

        while let Some(objective) = objectives.get(self.completed_count).copied() {
            if !objective.is_complete(route, position, mode) {
                break;
            }
            self.completed_count += 1;
        }

        if let Some(objective) = objectives.get(self.completed_count).copied() {
            self.current_label = objective.label;
            self.current_distance_m = objective.horizontal_distance(position);
            self.complete = false;
        } else {
            self.current_label = "complete";
            self.current_distance_m = 0.0;
            self.complete = !objectives.is_empty();
        }
    }

    pub(crate) fn current_step(&self) -> usize {
        if self.total_count == 0 {
            0
        } else {
            (self.completed_count + 1).min(self.total_count)
        }
    }
}
