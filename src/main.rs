mod components;
mod globals;
mod plugins;
mod resources;
mod systems;

use crate::plugins::boids::BoidsPlugin;
use crate::plugins::setup::SetupPlugin;
use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use std::{f32::consts::FRAC_PI_2, ops::Range};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SetupPlugin)
        .add_plugins(BoidsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, orbit)
        .init_resource::<CameraSettings>()
        .run();
}

#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    pub pitch_range: Range<f32>,
    pub roll_speed: f32,
    pub yaw_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 250.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            roll_speed: 1.0,
            yaw_speed: 0.004,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(200.0, 200.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Light"),
        PointLight::default(),
        Transform::from_xyz(300.0, 50.0, 5.0),
    ));
}

fn orbit(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    let delta = mouse_motion.delta;
    let mut delta_roll = 0.0;

    if mouse_buttons.pressed(MouseButton::Left) {
        let delta_pitch = delta.y * camera_settings.pitch_speed;
        let delta_yaw = delta.x * camera_settings.yaw_speed;

        delta_roll *= camera_settings.roll_speed * time.delta_secs();

        let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

        let pitch = (pitch + delta_pitch).clamp(
            camera_settings.pitch_range.start,
            camera_settings.pitch_range.end,
        );
        let roll = roll + delta_roll;
        let yaw = yaw + delta_yaw;
        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

        let target = Vec3::ZERO;
        camera.translation = target - camera.forward() * camera_settings.orbit_distance;
    }
}
