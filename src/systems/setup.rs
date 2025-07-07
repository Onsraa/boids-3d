use crate::globals::{DEPTH, HEIGHT, WIDTH};
use bevy::prelude::*;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let shape = meshes.add(Cuboid {
        half_size: Vec3::new(WIDTH / 2.0, HEIGHT / 2.0, DEPTH / 2.0),
    });

    let boundary_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.5, 0.0, 0.05),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        Mesh3d(shape),
        MeshMaterial3d(boundary_material),
        Transform {
            translation: Vec3::new(0.0, HEIGHT / 2.0, 0.0),
            ..default()
        },
    ));
}
