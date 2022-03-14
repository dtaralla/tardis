use crate::geometry::Point;
use crate::tle::TLE;
use crate::traits::*;
use crate::utils::Observer;
use crate::viewer::tle_stream::TleStream;
use bevy::prelude::*;

#[derive(Component)]
pub struct SatelliteInfo(TLE);

#[derive(Bundle)]
pub struct Satellite {
    pub info: SatelliteInfo,

    #[bundle]
    pub visuals: PbrBundle,
}

#[derive(Default)]
pub struct SatelliteMat(pub Handle<StandardMaterial>);

impl Satellite {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<StandardMaterial>,
        tle: TLE,
    ) -> Self {
        Self {
            info: SatelliteInfo(tle),
            visuals: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                material,
                ..default()
            },
        }
    }
}

pub fn move_satellites(
    mut commands: Commands,
    mut satellites: Query<(Entity, &SatelliteInfo, &mut Transform)>,
    observer: Res<Observer>,
) {
    for (entity, info, mut tfm) in satellites.iter_mut() {
        let sat_tle: &TLE = &info.0;
        let obs = match sat_tle.observation(&observer) {
            Ok(o) => o,
            Err(_) => {
                commands.entity(entity).despawn();
                continue;
            }
        };
        tfm.translation = point2vec(&obs.position);
    }
}

pub fn update_satellites(
    shared_mat: Res<SatelliteMat>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut satellites: Query<(Entity, &mut SatelliteInfo)>,
    mut tle_stream: ResMut<TleStream>,
) {
    if let Ok(mut tles) = tle_stream.next() {
        for (entity, mut info) in satellites.iter_mut() {
            if let Some(tle) = tles.remove(&info.0.name()) {
                // Update existing satellites with new TLE
                info.0 = tle;
            } else {
                // Remove existing satellites that disappeared from the source (crashed? :D)
                commands.entity(entity).despawn();
            }
        }

        // Spawn new satellites with unmatched TLEs
        let mut sats = Vec::<Satellite>::with_capacity(tles.len());
        for (_, tle) in tles {
            sats.push(Satellite::new(&mut meshes, shared_mat.0.clone(), tle))
        }
        commands.spawn_batch(sats);
    }
}

fn point2vec(p: &Point) -> Vec3 {
    Vec3::new(
        p[0] as f32 / 1000.,
        p[2] as f32 / 1000.,
        p[1] as f32 / 1000.,
    )
}
