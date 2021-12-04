/**
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use chrono::{DateTime, Utc};
use sgp4::sgp4::SGP4;
use crate::geometry::{Angle, Vector};
use crate::traits::Observable;
use crate::utils::{Observation, Observer};

const EARTH_SUN_DISTANCE_KM: u64 = 147_440_000;

struct Sun {
    val: u32
}

impl Sun {
    pub fn new() -> Sun
    {
        Sun {
            val: 42
        }
    }
}

impl Observable for Sun {
    fn name(&self) -> String {
        String::from("Sun")
    }

    fn observation(&self, observer: &Observer) -> Result<Observation, String> {
        self.observation_at(observer, Utc::now())
    }

    fn observation_at(&self, observer: &Observer, time: DateTime<Utc>) -> Result<Observation, String> {
        // Sun direction (See https://en.wikipedia.org/wiki/Position_of_the_Sun#Ecliptic_coordinates)
        let jd = SGP4::julian_day(time) - 2451545.0;
        let l = Angle::from_degrees(280.460 + 0.9856474 * jd); //TODO: Maybe ignore aberration of light
        let g = Angle::from_degrees(357.528 + 0.9856003 * jd);
        let ecliptic_lon = Angle::from_degrees(l.degrees() + 1.915 * g.radians().sin() + 0.020 * (g.radians()*2.0).sin());

        //let sun_dir = Vector::from_spherical(ecliptic_lon, Angle::from_degrees(0.0), ref_size as f64 * 2.0);
        //let sun_dir = Point3::new(sun_dir[0] as f32, sun_dir[2] as f32, sun_dir[1] as f32);

        Ok(Observation {
            observer: *observer,
            brightness: 1.0,
            time,
            position: Vector::from_spherical(Angle::from_degrees(0.0), ecliptic_lon, EARTH_SUN_DISTANCE_KM as f64),
            speed: Vector::from_cartesian(0.0, 0.0, 0.0),
        })
    }
}