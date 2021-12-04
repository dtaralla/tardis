/**
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::f64::consts::PI;
use crate::traits::Frame;
use crate::geometry::{
    Matrix,
    RotationAxis,
};
use crate::time::jd_utc_to_tt;
use chrono::{
    DateTime,
    Timelike,
    Utc
};
use sgp4::sgp4::SGP4;
use crate::{kf5, time};
use crate::algebra::evalpoly;


///
/// TEME is the frame used by TLE and sgp4. Note that the frame is dependant on time
///
pub struct TEME {
    date_time: DateTime<Utc>,
}

impl TEME {
    fn mod_to_gcrf_fk5(tt: f64) -> Matrix {
        let precession = kf5::precession(tt);
        Matrix::rot_from_angles(precession[2], -precession[1], precession[0], RotationAxis::ZYZ)
    }

    fn teme_to_mod(tt: f64, delta_eps: f64, delta_psi: f64) -> Matrix {

        // Compute the nutation in the Julian Day (Terrestrial Time) `JD_TT`.
        let nutation = kf5::nutation(tt);

        // Add the corrections to the nutation in obliquity and longitude.
        let a = nutation[0];
        let b = nutation[1] + delta_eps;
        let c = nutation[2] + delta_psi;

        let obliquity = a + b;

        // Evaluate the Delaunay parameters associated with the Moon in the interval
        // [0,2π]°.
        //
        // The parameters here were updated as stated in the errata [2].
        let t_tt = (tt - time::JD_J2000) / 36525.0;
        let r = 360.0;
        let mut delaunay = evalpoly(
            t_tt,
            vec![125.04452222,
                 -5.0 * r - 134.1362608,
                 0.0020708,
                 2.2e-6],
        );
        delaunay = (delaunay % 360.0) * PI / 180.0;

        // Compute the equation of Equinoxes.
        //
        // According to [2], the constant unit before `sin(2Ω_m)` is also in [rad].
        let Eq_equinox1982 = c * a.cos() +
            (0.002640 * delaunay.sin() + 0.000063 * (2.0 * delaunay).sin()) * PI / 648000.0;

        // Compute the rotation.
        let tod_teme = Matrix::rot_from_angles(-Eq_equinox1982, 0.0, 0.0, RotationAxis::ZYX);
        let mod_tod = Matrix::rot_from_angles(obliquity, c, -a, RotationAxis::XZX);

        Matrix::compose(mod_tod, tod_teme)
    }

    fn gcrf_to_teme_matrix(&self) -> Matrix {
        self.teme_to_gcrf_matrix().invert().unwrap()
    }

    fn teme_to_gcrf_matrix(&self) -> Matrix {
        //let milliarcsec_to_rad = PI / 648000000.0;

        // Get the time in TT.
        let jd_tt = jd_utc_to_tt(SGP4::julian_day(self.date_time));

        // Get the EOP data related to the desired epoch.
        // This is currently not supported. TODO: Find a table with this information
        // Because of this missing information, GCRF is closer to J2000
        let eps_1980 = 0.0;//eop_data.dEps(jd_tt) * milliarcsec_to_rad;
        let psi_1980 = 0.0;//eop_data.dPsi(jd_tt) * milliarcsec_to_rad;

        // Return the rotation.
        let r_TEME_MOD = TEME::teme_to_mod(jd_tt, eps_1980, psi_1980);
        let r_MOD_GCRF = TEME::mod_to_gcrf_fk5(jd_tt);

        // Compose the full rotation.
        Matrix::compose(r_MOD_GCRF, r_TEME_MOD)
    }
}

impl Frame for TEME {
    fn new(date_time: DateTime<Utc>) -> Self {
        TEME {
            date_time
        }
    }

    fn name(&self) -> String {
        String::from("TEME")
    }


    //fn has_obs_time(&self) -> bool {
    //    true
    //}

    fn to_gcrf(&self, point: [f64; 3]) -> [f64; 3]
    {
        self.teme_to_gcrf_matrix().rotate(point)
    }

    fn from_gcrf(&self, point: [f64; 3]) -> [f64; 3]
    {
        self.gcrf_to_teme_matrix().rotate(point)
    }
}


pub struct GCRF {
    date_time: DateTime<Utc>,
}

impl Frame for GCRF {
    fn new(date_time: DateTime<Utc>) -> Self {
        GCRF {
            date_time
        }
    }
    fn name(&self) -> String {
        String::from("GCRF")
    }

    fn to_gcrf(&self, point: [f64; 3]) -> [f64; 3]
    {
        point
    }

    fn from_gcrf(&self, point: [f64; 3]) -> [f64; 3]
    {
        point
    }
}

/* Earth-Centered Earth-Fixed */
// It is only valid for a fixed given time
pub struct ECEF {
    date_time: DateTime<Utc>,
}

impl Frame for ECEF {
    fn new(date_time: DateTime<Utc>) -> Self {
        ECEF {
            date_time
        }
    }

    fn name(&self) -> String {
        String::from("ECEF") + &self.date_time.to_string()
    }

    fn to_gcrf(&self, point: [f64; 3]) -> [f64; 3]
    {
        //TODO
        point
    }

    fn from_gcrf(&self, point: [f64; 3]) -> [f64; 3]
    {
        //TODO
        [point[0] + self.date_time.time().second() as f64, point[1] + self.date_time.time().second() as f64, point[2] + self.date_time.time().second() as f64]
    }
}

// ECI: (Earth Center Inertial) -> Not turning with earth
//  - GCRS  (Geocentric Celestial Reference Frame)
//  - TEME  (True Equator, Mean Equinox)
//  - J2000 (Earth's Mean Equator and Equinox at 12:00 Terrestrial Time on 1 January 200)
// ECEF: (Earth Center Earth Fixed) -> Turning with earth
//  - ITRF (International Terrestrial Reference Frame)
//  - PEF  (Pseudo-Earth Fixed)
//  - TIRS (Terrestrial Intermediate Reference System)
// Sun-centered
//  - ICRF (International Celestial Reference Frame)
//
// WARNING: Speeds cannot be converted to another frame if the obs_time is not the same.
//  Either convert the speed with physics (might be really hard to do)
//  Or Fail when trying to convert a speed at another time. There should be a Speed element.
//  TODO: Vector should be used for speed vectors. Point should be used for positions
//   It would be the role of each element to check if they can be safely converted from a frame to
//   another one
//   Speeds are ignored for now
