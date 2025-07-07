use crate::resources::settings::{BoidSettings, GroupsTargets};
use crate::systems::boids::{spawn_boids, spawn_obstacle};
use bevy::prelude::*;

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoidSettings::default());
        app.insert_resource(GroupsTargets::default());
        app.add_systems(Startup, (spawn_boids, spawn_obstacle));
    }
}
