use bevy::prelude::*;
use crate::components::boid::{Boid, Velocity, Acceleration, Obstacle};
use crate::resources::settings::BoidSettings;
use crate::events::ApplyForceEvent;
use crate::globals::{WIDTH, HEIGHT, DEPTH, MIN_HEIGHT};

pub fn flocking_system(
    boid_query: Query<(Entity, &Transform, &Velocity, &Boid), With<Boid>>,
    mut event_writer: EventWriter<ApplyForceEvent>,
    boid_settings: Res<BoidSettings>,
) {
    let boids: Vec<_> = boid_query.iter().collect();

    for (entity, transform, velocity, boid) in &boids {
        let position = transform.translation;
        let mut cohesion_sum = Vec3::ZERO;
        let mut cohesion_count = 0;
        let mut alignment_sum = Vec3::ZERO;
        let mut alignment_count = 0;
        let mut separation_sum = Vec3::ZERO;

        // Parcourir tous les autres boids
        for (other_entity, other_transform, other_velocity, other_boid) in &boids {
            if entity == other_entity {
                continue;
            }

            let other_position = other_transform.translation;
            let distance = position.distance(other_position);

            // Vérifier si dans le champ de vision
            if !is_in_field_of_view(&position, &velocity.velocity, &other_position, boid_settings.field_of_view) {
                continue;
            }

            // Séparation (éviter les collisions)
            if distance < boid_settings.separation_range && distance > 0.0 {
                let diff = (position - other_position) / distance;
                separation_sum += diff;
            }

            // Alignement (suivre la même direction)
            if distance < boid_settings.alignment_range {
                alignment_sum += other_velocity.velocity;
                alignment_count += 1;
            }

            // Cohésion (rester groupé)
            if distance < boid_settings.cohesion_range {
                cohesion_sum += other_position;
                cohesion_count += 1;
            }
        }

        // Calculer les forces
        let mut total_force = Vec3::ZERO;

        // Force de cohésion
        if cohesion_count > 0 {
            let center = cohesion_sum / cohesion_count as f32;
            let cohesion_force = (center - position) * boid_settings.cohesion_coeff;
            total_force += cohesion_force;
        }

        // Force d'alignement
        if alignment_count > 0 {
            let avg_velocity = alignment_sum / alignment_count as f32;
            let alignment_force = (avg_velocity - velocity.velocity) * boid_settings.alignment_coeff;
            total_force += alignment_force;
        }

        // Force de séparation
        total_force += separation_sum * boid_settings.separation_coeff;

        // Envoyer l'événement pour appliquer la force
        event_writer.write(ApplyForceEvent {
            entity: *entity,
            force: total_force,
        });
    }
}

pub fn avoid_obstacles(
    boid_query: Query<(Entity, &Transform, &mut Velocity), With<Boid>>,
    obstacle_query: Query<(&Transform, &Obstacle), Without<Boid>>,
    mut event_writer: EventWriter<ApplyForceEvent>,
    boid_settings: Res<BoidSettings>,
) {
    for (entity, transform, velocity) in boid_query.iter() {
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

        event_writer.send(ApplyForceEvent {
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
            let forward = velocity.velocity.normalize();
            // Utiliser look_to pour orienter le modèle correctement
            transform.look_to(forward, Vec3::Y);
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
    let turn_factor = 1.0;

    for (mut transform, mut velocity) in query.iter_mut() {
        let pos = &mut transform.translation;

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

fn is_in_field_of_view(position: &Vec3, velocity: &Vec3, other_pos: &Vec3, fov_degrees: f32) -> bool {
    let to_other = *other_pos - *position;

    if velocity.length_squared() == 0.0 || to_other.length_squared() == 0.0 {
        return true; // Si pas de mouvement, voir tout
    }

    let cos_fov = (fov_degrees / 2.0).to_radians().cos();
    let dot = velocity.normalize().dot(to_other.normalize());

    dot >= cos_fov
}