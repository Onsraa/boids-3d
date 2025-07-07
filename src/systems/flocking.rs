use bevy::prelude::*;
use bevy_spatial::SpatialAccess;
use std::sync::Mutex;
use crate::components::boid::{Boid, Velocity, Acceleration, Obstacle};
use crate::components::spatial::NNTree3D;
use crate::resources::settings::{BoidSettings, GroupsTargets};
use crate::events::ApplyForceEvent;
use crate::globals::{WIDTH, HEIGHT, DEPTH, MIN_HEIGHT};

pub fn flocking_system(
    boid_query: Query<(Entity, &Transform, &Velocity, &Boid), With<Boid>>,
    event_writer: EventWriter<ApplyForceEvent>,
    boid_settings: Res<BoidSettings>,
    groups_targets: Res<GroupsTargets>,
    kd_tree: Res<NNTree3D>,
) {
    let event_writer = Mutex::new(event_writer);

    boid_query.par_iter().for_each(|(entity, transform, velocity, boid)| {
        let position = transform.translation;
        let mut cohesion_neighbors: Vec<Vec3> = Vec::new();
        let mut repulsion_neighbors: Vec<(Vec3, f32)> = Vec::new();
        let mut alignment_neighbors: Vec<Vec3> = Vec::new();

        // Utiliser le KD-Tree pour trouver les voisins
        for (_, neighbor_entity) in kd_tree.within_distance(position, boid_settings.cohesion_range) {
            if let Some(neighbor_entity) = neighbor_entity {
                if neighbor_entity == entity { continue; }

                if let Ok((_, neighbor_transform, neighbor_velocity, _)) = boid_query.get(neighbor_entity) {
                    let neighbor_pos = neighbor_transform.translation;

                    if let Some(distance) = is_in_field_of_view(&position, &velocity.velocity, &neighbor_pos, &boid_settings.field_of_view) {
                        if distance < boid_settings.separation_range {
                            repulsion_neighbors.push((neighbor_pos, distance));
                        } else if distance < boid_settings.alignment_range {
                            alignment_neighbors.push(neighbor_velocity.velocity);
                        } else if distance < boid_settings.cohesion_range {
                            cohesion_neighbors.push(neighbor_pos);
                        }
                    }
                }
            }
        }

        // Calculer les forces
        let cohesion_force = cohesion(&position, &cohesion_neighbors, &boid_settings.cohesion_coeff);
        let separation_force = separation(&position, &repulsion_neighbors, &boid_settings.separation_coeff);
        let alignment_force = alignment(&velocity.velocity, &alignment_neighbors, &boid_settings.alignment_coeff);

        // Attraction vers la cible du groupe
        let target = groups_targets.targets[boid.group as usize];
        let attraction_force = attraction_to_target(&position, &target, &boid_settings.attraction_coeff);

        let total_force = cohesion_force + separation_force + alignment_force + attraction_force;

        let mut event_writer = event_writer.lock().unwrap();
        event_writer.write(ApplyForceEvent {
            entity,
            force: total_force
        });
    });
}

fn cohesion(position: &Vec3, cohesion_neighbors: &Vec<Vec3>, cohesion_coeff: &f32) -> Vec3 {
    if cohesion_neighbors.is_empty() {
        return Vec3::ZERO;
    }

    let center: Vec3 = cohesion_neighbors.iter().sum::<Vec3>() / cohesion_neighbors.len() as f32;
    (center - *position) * *cohesion_coeff
}

fn separation(position: &Vec3, repulsion_neighbors: &Vec<(Vec3, f32)>, separation_coeff: &f32) -> Vec3 {
    let mut separation_force = Vec3::ZERO;

    for (other_pos, distance) in repulsion_neighbors {
        if *distance > 0.0 {
            separation_force += (*position - *other_pos) / *distance;
        }
    }

    separation_force * *separation_coeff
}

fn alignment(velocity: &Vec3, alignment_neighbors: &Vec<Vec3>, alignment_coeff: &f32) -> Vec3 {
    if alignment_neighbors.is_empty() {
        return Vec3::ZERO;
    }

    let avg_velocity: Vec3 = alignment_neighbors.iter().sum::<Vec3>() / alignment_neighbors.len() as f32;
    (avg_velocity - *velocity) * *alignment_coeff
}

fn attraction_to_target(position: &Vec3, target: &Vec3, attraction_coeff: &f32) -> Vec3 {
    (*target - *position) * *attraction_coeff
}

fn is_in_field_of_view(position: &Vec3, velocity: &Vec3, other_pos: &Vec3, fov_degrees: &f32) -> Option<f32> {
    let to_other = *other_pos - *position;
    let distance = to_other.length();

    if distance <= 0.0 || velocity.length_squared() == 0.0 {
        return Some(distance);
    }

    let cos_fov = (fov_degrees / 2.0).to_radians().cos();
    let dot = velocity.normalize().dot(to_other.normalize());

    if dot >= cos_fov {
        Some(distance)
    } else {
        None
    }
}

pub fn avoid_obstacles(
    boid_query: Query<(Entity, &Transform), With<Boid>>,
    obstacle_query: Query<(&Transform, &Obstacle), Without<Boid>>,
    mut event_writer: EventWriter<ApplyForceEvent>,
    boid_settings: Res<BoidSettings>,
) {
    for (entity, transform) in boid_query.iter() {
        let position = transform.translation;
        let mut avoidance_force = Vec3::ZERO;

        for (obstacle_transform, obstacle) in obstacle_query.iter() {
            let obstacle_pos = obstacle_transform.translation;
            let to_obstacle = obstacle_pos - position;
            let distance = to_obstacle.length();

            // Distance depuis la surface de l'obstacle
            let surface_distance = distance - obstacle.radius;

            if surface_distance < boid_settings.separation_range * 2.0 && surface_distance > 0.0 {
                // Force de répulsion inversement proportionnelle à la distance
                let repulsion_dir = (position - obstacle_pos).normalize_or_zero();
                let strength = 1.0 - (surface_distance / (boid_settings.separation_range * 2.0));
                avoidance_force += repulsion_dir * strength * boid_settings.collision_coeff;
            }
        }

        event_writer.write(ApplyForceEvent {
            entity,
            force: avoidance_force,
        });
    }
}

pub fn apply_forces(
    mut events: EventReader<ApplyForceEvent>,
    mut query: Query<&mut Acceleration, With<Boid>>,
) {
    for event in events.read() {
        if let Ok(mut acceleration) = query.get_mut(event.entity) {
            acceleration.acceleration += event.force;
        }
    }
}

pub fn update_boids(
    mut query: Query<(&mut Transform, &mut Velocity, &mut Acceleration), With<Boid>>,
    boid_settings: Res<BoidSettings>,
    time: Res<Time>,
) {
    for (mut transform, mut velocity, mut acceleration) in query.iter_mut() {
        // Appliquer l'accélération à la vélocité
        velocity.velocity += acceleration.acceleration * time.delta_secs();

        // Limiter la vitesse
        let speed = velocity.velocity.length();
        if speed > 0.0 {
            if speed < boid_settings.min_speed {
                velocity.velocity = velocity.velocity.normalize() * boid_settings.min_speed;
            } else if speed > boid_settings.max_speed {
                velocity.velocity = velocity.velocity.normalize() * boid_settings.max_speed;
            }
        }

        // Mettre à jour la position
        transform.translation += velocity.velocity * time.delta_secs();

        // Orienter le boid dans la direction du mouvement
        if velocity.velocity.length_squared() > 0.0 {
            let forward = -velocity.velocity.normalize();
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, forward);
        }

        // Réinitialiser l'accélération
        acceleration.acceleration = Vec3::ZERO;
    }
}

pub fn confine_boids(
    mut query: Query<(&mut Transform, &mut Velocity), With<Boid>>,
    boid_settings: Res<BoidSettings>,
) {
    let margin = 10.0;
    let turn_factor = 10.0;

    for (transform, mut velocity) in query.iter_mut() {
        let pos = transform.translation;

        if boid_settings.bounce_against_walls {
            // Confinement horizontal
            if pos.x < -WIDTH / 2.0 + margin {
                velocity.velocity.x += turn_factor;
            } else if pos.x > WIDTH / 2.0 - margin {
                velocity.velocity.x -= turn_factor;
            }

            // Confinement vertical
            if pos.y < MIN_HEIGHT + margin {
                velocity.velocity.y += turn_factor;
            } else if pos.y > HEIGHT - margin {
                velocity.velocity.y -= turn_factor;
            }

            // Confinement en profondeur
            if pos.z < -DEPTH / 2.0 + margin {
                velocity.velocity.z += turn_factor;
            } else if pos.z > DEPTH / 2.0 - margin {
                velocity.velocity.z -= turn_factor;
            }
        }
    }
}