/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::rc::Rc;
use chrono::{DateTime, Utc};
use sgp4::sgp4::SGP4;
use crate::frames::ICRF;
use crate::geometry::{Angle, Vector};
use crate::traits::{Framable, Observable};
use crate::utils::Observation;

pub const EARTH_SUN_DISTANCE_KM: u64 = 147_440_000;
pub const SUN_RADIUS_KM: u64 = 696_340;

pub struct Sun {
    name: String
}

impl Sun {
    pub fn new() -> Sun
    {
        Sun {
            name: String::from("Sun")
        }
    }
}


/// Planets position from JPL BSP files: https://www.astrogreg.com/jpl-ephemeris-format/jpl-ephemeris-format.html
/// They are in NAIF/DAF format:
///  * https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/daf.html
///  * Maybe https://github.com/mattbornski/spice
/// 

impl Observable for Sun {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn observation(&self) -> Result<Observation, String> {
        self.observation_at(Utc::now())
    }

    fn observation_at(&self, time: DateTime<Utc>) -> Result<Observation, String> {
        // Sun direction (See https://en.wikipedia.org/wiki/Position_of_the_Sun#Ecliptic_coordinates)
        /*
        let jd = SGP4::julian_day(time) - 2451545.0;
        let l = Angle::from_degrees(280.460 + 0.9856474 * jd); //TODO: Maybe ignore aberration of light
        let g = Angle::from_degrees(357.528 + 0.9856003 * jd);
        let ecliptic_lon = Angle::from_degrees(l.degrees() + 1.915 * g.radians().sin() + 0.020 * (g.radians()*2.0).sin());
        println!("lon: {}", ecliptic_lon.normalized());

        Ok(Observation {
            brightness: 1.0,
            time,
            position: Vector::from_spherical(ecliptic_lon, Angle::from_degrees(0.0), EARTH_SUN_DISTANCE_KM as f64).to_point(),
            speed: Vector::from_cartesian(0.0, 0.0, 0.0),
        })
        */

        let mut sun_pos = Vector::from_cartesian(0.0, 0.0, 0.0).to_point();
        let mut sun_speed = Vector::from_cartesian(0.0, 0.0, 0.0);

        sun_pos.set_frame(Rc::new(ICRF::new(time)));
        sun_speed.set_frame(Rc::new(ICRF::new(time)));

        Ok(Observation {
            brightness: 1.0,
            time,
            position: sun_pos,
            speed: sun_speed,
        })
    }
}