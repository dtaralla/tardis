/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::fmt;
use std::rc::Rc;
use chrono::{DateTime, Utc};
use crate::constants::EARTH_EQUATORIAL_RADIUS_KM;
use crate::geometry::{Angle, Point, Vector};
use crate::frames::ECEF;
use crate::traits::{Framable, Frame};

#[derive(Copy, Clone)]
pub struct Coordinates {
    lat: f64,
    lon: f64,
}

impl Coordinates {
    pub fn new(lat: f64, lon: f64) -> Coordinates
    {
        Coordinates {
            lat,
            lon
        }
    }

    /// Return a the position of the observer as a vector.
    /// TODO: This should actually be changed to a Frame.
    pub fn to_vector(&self) -> Vector
    {
        let mut v = Vector::from_spherical(
            Angle::from_degrees(self.lon),
            Angle::from_degrees(self.lat),
            EARTH_EQUATORIAL_RADIUS_KM
        );

        v.set_frame(Rc::new(ECEF::new(Utc::now())));

        v
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Coordinates: [lat: {}, lon: {}]", self.lat, self.lon)
    }
}

#[derive(Copy, Clone)]
pub struct Observer {
    coordinates: Coordinates,   // Coordinates on Earth
    time: DateTime<Utc>
}

impl Observer {
    pub fn new(coordinates: Coordinates, time: DateTime<Utc>) -> Observer
    {
        Observer {
            coordinates,
            time
        }
    }

    pub fn coordinates(&self) -> Coordinates
    {
        self.coordinates
    }
}

impl Frame for Observer {
    fn name(&self) -> String {
        String::from("Observer at ") + self.coordinates.to_string().as_str()
    }

    fn to_gcrf(&self, point: [f64; 3]) -> [f64; 3] {
        point
    }

    fn from_gcrf(&self, point: [f64; 3]) -> [f64; 3] {
        point
    }
}

///
/// Specifies where the observable object is in GCRF coordinates
pub struct Observation {
    pub time: DateTime<Utc>,        // The time at which this observation is valid
    pub position: Point,
    pub speed: Vector,
    pub brightness: f64             // Brightness of the satellite
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Observation at {}: position: {} speed: {}",
               self.time,
               self.position,
               self.speed)
    }
}
