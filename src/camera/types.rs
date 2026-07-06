use bevy::prelude::*;

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
            distance: 11.25,
            height: 5.0,
            look_height: 1.4,
            look_ahead: 0.5,
            position_smoothing: 22.0,
            rotation_smoothing: 24.0,
            direction_smoothing: 1.0,
            min_height: 1.6,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct FollowCameraState {
    pub direction: Vec3,
    pub(super) initialized: bool,
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

    pub(super) fn expanded(self, clearance: f32) -> Self {
        Self {
            center: self.center,
            half_extents: self.half_extents + Vec3::splat(clearance.max(0.0)),
        }
    }

    pub(super) fn contains(self, point: Vec3) -> bool {
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
