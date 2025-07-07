use crate::resources::settings::BoidSettings;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_systems(EguiPrimaryContextPass, ui_system);
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut boid_settings: ResMut<BoidSettings>,
    diagnostics: Res<DiagnosticsStore>,
) -> Result {
    egui::Window::new("Paramètres Boids").show(contexts.ctx_mut()?, |ui| {
        ui.heading("Forces");

        ui.add(egui::Slider::new(&mut boid_settings.cohesion_coeff, 0.0..=50.0).text("Cohésion"));

        ui.add(
            egui::Slider::new(&mut boid_settings.alignment_coeff, 0.0..=20.0).text("Alignement"),
        );

        ui.add(
            egui::Slider::new(&mut boid_settings.separation_coeff, 0.0..=50.0).text("Séparation"),
        );

        ui.add(
            egui::Slider::new(&mut boid_settings.collision_coeff, 0.0..=100.0)
                .text("Évitement obstacles"),
        );

        ui.add(
            egui::Slider::new(&mut boid_settings.attraction_coeff, 0.0..=10.0)
                .text("Attraction cible"),
        );

        ui.separator();
        ui.heading("Distances");

        ui.add(
            egui::Slider::new(&mut boid_settings.cohesion_range, 10.0..=100.0)
                .text("Portée cohésion"),
        );

        ui.add(
            egui::Slider::new(&mut boid_settings.alignment_range, 10.0..=80.0)
                .text("Portée alignement"),
        );

        ui.add(
            egui::Slider::new(&mut boid_settings.separation_range, 5.0..=50.0)
                .text("Portée séparation"),
        );

        ui.separator();
        ui.heading("Vitesse");

        ui.add(egui::Slider::new(&mut boid_settings.min_speed, 10.0..=100.0).text("Vitesse min"));

        ui.add(egui::Slider::new(&mut boid_settings.max_speed, 50.0..=500.0).text("Vitesse max"));

        ui.separator();
        ui.heading("Autres paramètres");

        ui.add(
            egui::Slider::new(&mut boid_settings.field_of_view, 45.0..=180.0)
                .text("Champ de vision (°)"),
        );

        ui.checkbox(
            &mut boid_settings.bounce_against_walls,
            "Rebondir sur les murs",
        );

        ui.separator();

        ui.add(egui::Slider::new(&mut boid_settings.count, 1..=1000).text("Nombre de boids"));

        if let Some(value) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            ui.label(format!("FPS: {:.1}", value));
        }
    });
    Ok(())
}
