/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::fmt;
use std::rc::Rc;
use chrono::{DateTime, Utc};
use sgp4::sgp4::SGP4Result;
use crate::constants::EARTH_EQUATORIAL_RADIUS_KM;
use crate::geometry::{Angle, Point, Vector};
use crate::frames;
use crate::frames::{ECEF, GCRF};
use crate::traits::Framable;

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
    // This should be more generic: Earth should be an observer
}

impl Observer {
    pub fn new(coordinates: Coordinates) -> Observer
    {
        Observer {
            coordinates,
        }
    }

    pub fn earth() -> Observer
    {
        //FIXME: Should return an observer that is the center of earth --> Only coordinates is not enough
        Observer {
            coordinates: Coordinates::new(0.0, 0.0)
        }
    }

    pub fn coordinates(&self) -> Coordinates
    {
        self.coordinates
    }

    /*pub fn plane(&self) -> Result<Plane, String>
    {
        let obs_vector = self.coordinates.to_vector();

        // Find perpendicular vectors to the obs vector
        //TODO: Should also support vectors that are on base axis (like [0, 0, earth_radius])
        //TODO: This can be managed by the Plane struct: add a Plane::from_normal(normal: &Vector)
        let base_vect1 = &obs_vector + &Vector::from_cartesian(
            -obs_vector[1],
            obs_vector[0],
            0.,
        );

        let base_vect2 = &obs_vector + &Vector::from_cartesian(
            -obs_vector[2],
            0.,
            obs_vector[0],
        );

        //println!("{} {} {}", obs_vector, base_vect1, base_vect2);

        Plane::from_vectors_normal(&obs_vector,
                                        &base_vect1,
                                        &base_vect2
        )
    }*/
}

///
/// Specifies where the observable object is in GCRF coordinates
pub struct Observation {
    pub time: DateTime<Utc>,        // The time at which this observation is valid
    pub observer: Observer,         // Observer on earth
    pub position: Point,
    pub speed: Vector,
    pub brightness: f64             // Brightness of the satellite
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Observation at {} from {}: position: {} speed: {}",
               self.time,
               self.observer.coordinates,
               self.position,
               self.speed)
    }
}
