use crate::resources::settings::{BoidSettings, GroupsTargets};
use crate::systems::boids::{spawn_boids, spawn_obstacles};
use crate::systems::flocking::*;
use crate::events::ApplyForceEvent;
use crate::ui::UiPlugin;
use bevy::prelude::*;

pub struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoidSettings::default())
            .insert_resource(GroupsTargets::default())
            .add_event::<ApplyForceEvent>()
            .add_plugins(UiPlugin)
            .add_systems(Startup, (spawn_boids, spawn_obstacles))
            .add_systems(Update, (
                flocking_system,
                avoid_obstacles,
                apply_forces,
                update_boids,
                confine_boids,
            ).chain());
    }
}