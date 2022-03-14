use crate::constants::EARTH_EQUATORIAL_RADIUS_KM;
use crate::utils::Observer;
use crate::viewer::camera::CameraControlPlugin;
use crate::viewer::satellite::{move_satellites, update_satellites, SatelliteMat};
use crate::viewer::tle_stream::TleStream;
use bevy::prelude::*;

pub struct ViewerPlugin {
    pub tle_source_url: String,
}

impl Plugin for ViewerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa { samples: 4 })
            .insert_resource(TleStream::new(&self.tle_source_url))
            .insert_resource(Observer::earth())
            .add_plugin(CameraControlPlugin)
            .add_startup_system(setup)
            .add_system(move_satellites)
            .add_system(update_satellites);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // earth
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: EARTH_EQUATORIAL_RADIUS_KM as f32 / 1000.,
            ..default()
        })),
        material: materials.add(Color::rgb(0.1, 0.1, 0.9).into()),
        transform: Transform::from_xyz(0., 0., 0.),
        ..Default::default()
    });

    // sun
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 50_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // satellite materials (same instance to share batches)
    commands.insert_resource(SatelliteMat(materials.add(StandardMaterial {
        base_color: Color::ORANGE_RED,
        unlit: true,
        ..default()
    })));
}
