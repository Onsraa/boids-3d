use crate::components::boid::{Boid, Obstacle, Velocity, Acceleration};
use crate::globals::{DEPTH, HEIGHT, MIN_HEIGHT, WIDTH};
use crate::resources::settings::BoidSettings;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

fn spawn_boid_entity(
    commands: &mut Commands,
    boid_settings: &BoidSettings,
    asset_server: &Res<AssetServer>,
) {
    let mut rng = rand::rng();
    let group = rng.random_range(0..2);

    let random_pos = Vec3::new(
        rng.random_range(-WIDTH * 0.45..WIDTH * 0.45),
        rng.random_range(MIN_HEIGHT..HEIGHT * 0.95),
        rng.random_range(-DEPTH * 0.45..DEPTH * 0.45),
    );

    // Vitesse initiale aléatoire
    let theta = rng.random_range(0.0..2.0 * PI);
    let phi = rng.random_range(0.0..PI);
    let initial_velocity = Vec3::new(
        f32::sin(phi) * f32::cos(theta),
        f32::sin(phi) * f32::sin(theta),
        f32::cos(phi),
    ) * boid_settings.min_speed;

    commands.spawn((
        Boid { group },
        Velocity { velocity: initial_velocity },
        Acceleration { acceleration: Vec3::ZERO },
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/bird.gltf"))),
        Transform {
            translation: random_pos,
            scale: Vec3::splat(boid_settings.size),
            ..default()
        },
    ));
}

pub fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let positions = vec![
        Vec3::new(30.0, 40.0, 0.0),
        Vec3::new(-30.0, 60.0, 20.0),
        Vec3::new(0.0, 30.0, -30.0),
    ];

    for pos in positions {
        let radius = 10.0;
        let shape = meshes.add(Sphere::new(radius).mesh().uv(32, 18));

        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        });

        commands.spawn((
            Obstacle { radius },
            Mesh3d(shape),
            MeshMaterial3d(material),
            Transform::from_translation(pos),
        ));
    }
}

pub fn spawn_boids(
    mut commands: Commands,
    boid_settings: Res<BoidSettings>,
    asset_server: Res<AssetServer>,
) {
    for _ in 0..boid_settings.count {
        spawn_boid_entity(&mut commands, &boid_settings, &asset_server);
    }
}