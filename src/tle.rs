/**
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::{
    fmt,
    f64::consts::PI,
};
use std::rc::Rc;
use chrono::{Utc, DateTime, Duration, NaiveDate, NaiveTime};
use sgp4::sgp4::{ConstantsSet, OpsMode, SGP4};

use crate::geometry::{Angle, Vector};
use crate::frames::{TEME, GCRF};
use crate::utils::{Coordinates, Observation, Observer};
use crate::traits::{FramedElement, Observable, Frame};

pub enum SatelliteClass {
    Unclassified,
    Classified,
    Secret,
}

struct Designator {
    launch_year: u8,
    launch_number: u16,
    launch_piece: String,
}


// FIXME: Fields should not be public
// TODO: Document
// TODO: Improve error management
pub struct TLE {
    name: String,
    pub number: u32,
    class: SatelliteClass,
    designator: Designator,
    pub date: DateTime<Utc>,
    pub ndot: f64,
    pub ndotdot: f64,
    pub b_star: f64,
    pub set_number: u16,
    pub inclination: Angle,
    pub right_ascension: Angle,
    pub eccentricity: f64,
    pub perigee: Angle,
    pub mean_anomaly: Angle,
    pub mean_motion: f64,
    pub revolutions: u32,
}

impl fmt::Display for TLE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TLE: [\n    \
            name: {}\n    \
            number: {}\n    \
            date: {}\n    \
            ndot: {}\n    \
            ndotdot: {}\n    \
            b_star: {}\n    \
            set_number: {}\n    \
            inclination: {}\n    \
            right_ascension: {}\n    \
            eccentricity: {}\n    \
            perigee: {}\n    \
            mean_anomaly: {}\n    \
            mean_motion: {}\n    \
            revolutions: {}\n\
            ]",
                 self.name,
                 self.number,
                 self.date,
                 self.ndot,
                 self.ndotdot,
                 self.b_star,
                 self.set_number,
                 self.inclination,
                 self.right_ascension,
                 self.eccentricity,
                 self.perigee,
                 self.mean_anomaly,
                 self.mean_motion,
                 self.revolutions)
    }
}

impl TLE {
    fn checksum(line: &[u8]) -> Result<(), String>
    {
        let checksum = match TLE::parse_number(&line[68..69]) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        let mut count = 0;
        for v in line {
            count += match *v {
                b'1' => 1,
                b'2' => 2,
                b'3' => 3,
                b'4' => 4,
                b'5' => 5,
                b'6' => 6,
                b'7' => 7,
                b'8' => 8,
                b'9' => 9,
                b'-' => 1,
                _ => 0,
            };
        }

        count -= checksum;

        if count % 10 != checksum {
            return Err(String::from("Invalid checksum"));
        }

        Ok(())
    }

    pub fn from_lines(line1: &[u8], line2: &[u8], name_line: &[u8]) -> Result<TLE, String>
    {
        match TLE::checksum(line1) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        match TLE::checksum(line2) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        /* Check name */
        let name = match TLE::parse_string(name_line) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        /* Check line numbers */
        if TLE::parse_number(&line1[0..1]).unwrap() != 1 {
            return Err("Line 1 number is incorrect".to_string());
        }

        if TLE::parse_number(&line2[0..1]).unwrap() != 2 {
            return Err("Line 2 number is incorrect".to_string());
        }

        /* Get satellite number */
        let number = match TLE::parse_number(&line1[2..7]) {
            Ok(n) => n as u32,
            Err(e) => return Err(e),
        };

        /* Get satellite classification */
        let class = match TLE::parse_string(&line1[7..8]) {
            Ok(s) => match TLE::string_to_class(&s) {
                Ok(c) => c,
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        };

        /* Get satellite designator */
        let designator = match TLE::parse_designator(&line1[9..17]) {
            Ok(d) => d,
            Err(e) => return Err(e),
        };

        let date = match TLE::parse_date(&line1[18..32]) {
            Ok(d) => d,
            Err(e) => return Err(e),
        };

        println!("1");
        let ndot = match TLE::parse_float(&line1[33..43]) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        println!("1");
        let ndotdot = match TLE::parse_pow_10(&line1[44..52]) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        println!("1");
        let b_star = match TLE::parse_pow_10(&line1[53..61]) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        println!("1");
        let set_number = match TLE::parse_number(&line1[64..68]) {
            Ok(n) => n as u16,
            Err(e) => return Err(e),
        };

        let inclination = match TLE::parse_float(&line2[8..16]) {
            Ok(n) => Angle::from_degrees(n),
            Err(e) => return Err(e),
        };

        println!("1");
        let right_ascension = match TLE::parse_float(&line2[17..25]) {
            Ok(n) => Angle::from_degrees(n),
            Err(e) => return Err(e),
        };

        println!("1");
        let eccentricity = match TLE::parse_number(&line2[26..33]) {
            Ok(n) => n as f64 * 10e-8,
            Err(e) => return Err(e),
        };

        let perigee = match TLE::parse_float(&line2[34..42]) {
            Ok(n) => Angle::from_degrees(n),
            Err(e) => return Err(e),
        };

        println!("1");
        let mean_anomaly = match TLE::parse_float(&line2[43..51]) {
            Ok(n) => Angle::from_degrees(n),
            Err(e) => return Err(e),
        };

        println!("1");
        let mean_motion = match TLE::parse_float(&line2[52..63]) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        println!("1");
        let revolutions = match TLE::parse_number(&line2[63..68]) {
            Ok(n) => n as u32,
            Err(e) => return Err(e),
        };

        Ok(TLE {
            name,
            number,
            class,
            designator,
            date,
            ndot,
            ndotdot,
            b_star,
            set_number,
            inclination,
            right_ascension,
            eccentricity,
            perigee,
            mean_anomaly,
            mean_motion,
            revolutions,
        })
    }

    fn parse_string(bytes: &[u8]) -> Result<String, String>
    {
        let name = match String::from_utf8(Vec::from(bytes)) {
            Ok(n) => n,
            Err(e) => return Err(String::from("Cannot parse String ") + &e.to_string())
        };

        Ok(name.trim_end().to_string())
    }

    fn parse_number(bytes: &[u8]) -> Result<i32, String>
    {
        let number = match String::from_utf8(Vec::from(bytes)) {
            Ok(n) => n.trim_start().to_string(),
            Err(e) => return Err(e.to_string()),
        };

        match number.parse() {
            Ok(n) => Ok(n),
            Err(e) => Err(e.to_string())
        }
    }

    fn parse_number_i(bytes: &[u8]) -> Result<f64, String>
    {
        let mut number;

        if bytes[0] == b'-' {
            number = String::from("-0.");
        } else {
            number = String::from("0.");
        }

        match String::from_utf8(Vec::from(bytes)) {
            Ok(n) => {
                number.push_str(n.trim_start_matches(
                    |c: char| c.is_whitespace() || c == '-' || c == '.'
                ))
            },
            Err(e) => return Err(e.to_string()),
        };

        println!("{}", number);

        match number.parse() {
            Ok(n) => Ok(n),
            Err(e) => Err(e.to_string())
        }
    }

    fn parse_float(bytes: &[u8]) -> Result<f64, String>
    {
        let number = match String::from_utf8(Vec::from(bytes)) {
            Ok(n) => n.trim_start().to_string(),
            Err(e) => return Err(e.to_string()),
        };

        match number.parse() {
            Ok(n) => Ok(n),
            Err(e) => Err(e.to_string())
        }
    }

    fn parse_pow_10(bytes: &[u8]) -> Result<f64, String>
    {
        println!("{}", String::from_utf8_lossy(&bytes[0..bytes.len() - 2]));
        let base = match TLE::parse_number_i(&bytes[0..bytes.len() - 2]) {
            Ok(n) => n as f64,
            Err(e) => return Err(e),
        };

        println!("{}", String::from_utf8_lossy(&bytes[bytes.len() - 2..bytes.len()]));
        let exp = match TLE::parse_number(&bytes[bytes.len() - 2..bytes.len()]) {
            Ok(n) => n,
            Err(e) => return Err(e),
        };

        Ok(base * 10_f64.powi(exp))
    }

    fn string_to_class(class: &str) -> Result<SatelliteClass, String>
    {
        match class {
            "U" => Ok(SatelliteClass::Unclassified),
            "C" => Ok(SatelliteClass::Classified),
            "S" => Ok(SatelliteClass::Secret),
            _ => Err(String::from("Invalid class string ") + class)
        }
    }

    fn parse_designator(designator: &[u8]) -> Result<Designator, String>
    {
        let launch_year = match TLE::parse_number(&designator[0..=1]) {
            Ok(n) => n as u8,
            Err(e) => return Err(e)
        };

        let launch_number = match TLE::parse_number(&designator[2..=4]) {
            Ok(n) => n as u16,
            Err(e) => return Err(e)
        };

        let launch_piece = match TLE::parse_string(&designator[5..=7]) {
            Ok(s) => s,
            Err(e) => return Err(e)
        };

        Ok(Designator {
            launch_year,
            launch_number,
            launch_piece,
        })
    }

    fn parse_date(date: &[u8]) -> Result<DateTime<Utc>, String>
    {
        let year = match TLE::parse_number(&date[0..=1]) {
            Ok(n) => n,
            Err(e) => return Err(e)
        };

        /*
         * The year is given in 2 digits. Anything before 57 if in the 2000th.
         * The rest is in 1900th
         */
        let year = match year {
            (0..=56) => 2000 + year,
            _ => 1900 + year,
        };

        let tle_days = match TLE::parse_float(&date[2..=13]) {
            Ok(f) => f,
            Err(e) => return Err(e)
        };

        /*
         * The number of days in the TLE is given as a float value.
         * The integer part is the day number in the year.
         * The decimal part is the percentage within the day.
         */
        let days = tle_days.floor();
        let time_of_day = (tle_days - days) * 24. * 60. * 60.;

        let date = NaiveDate::from_ymd(year, 1, 1);
        let date = match date.checked_add_signed(Duration::days(days as i64 - 1)) {
            Some(d) => d,
            None => return Err(String::from("Date is out of bounds")),
        };

        let time = NaiveTime::from_num_seconds_from_midnight(time_of_day.floor() as u32, 0);

        Ok(DateTime::from_utc(date.and_time(time), Utc))
    }
}

impl Observable for TLE {
    fn name(&self) -> String
    {
        self.name.clone()
    }

    fn observation(&self, obs: &Observer) -> Result<Observation, String>
    {
        self.observation_at(obs, Utc::now())
    }

    fn observation_at(&self, obs: &Observer, time: DateTime<Utc>) -> Result<Observation, String>
    {
        //let I = Vector::from_tuple([1f64, 0f64, 0f64]);
        //let J = Vector::from_tuple([0f64, 1f64, 0f64]);
        //let K = Vector::from_tuple([0f64, 0f64, 1f64]);

        // Initialize coordinate references
        /*let ecliptic_ref = Referential::base_referential();
        let horizontal_ref = Rc::new(ecliptic_ref.make_referential(
            None,
            Some(
                [
                    Angle::from_radians(0.0),
                    Angle::from_radians(0.0),
                    Angle::from_radians(0.0)
                ]
            )
        ));*/

        let sgp4 = match SGP4::new(
            OpsMode::Afspc,
            ConstantsSet::Set72,
            self.b_star,
            self.eccentricity,
            self.date,
            self.perigee.radians(),
            self.inclination.radians(),
            self.mean_anomaly.radians(),
            self.mean_motion,
            self.right_ascension.radians()
        ) {
            Ok(s) => s,
            Err(e) => {
                println!("Error on satellite {} ({}): {}", self.number, self.name(), e);
                return Err(String::from("Cannot compute satellite position"));
            }
        };

        let res = match sgp4.compute(time) {
            Ok(r) => r,
            Err(e) => {
                println!("Error on satellite {} ({}): {}", self.number, self.name(), e);
                return Err(String::from("Cannot compute satellite position"));
            }
        };

        //println!("[{}] Satellite {} is at {} km moving at {} km/s", res.time(), self.name(), res.altitude(), res.velocity());
        //println!("{}", Angle::from_vectors(&I, &J));

        /// Compute azimuth angle //TODO: move it to a Frame conversion
        //1. Project res.position_vect() on observer plane.
        let pos_vect = Vector::<TEME>::from_tuple(res.position_vect());
        let speed_vect = Vector::<TEME>::from_tuple(res.velocity_vect());
        /*let obs_plane = match obs.plane() {
            Ok(p) =>
                p,//.change_referential(ecliptic_ref),
            Err(e) =>
                return Err(String::from("Cannot get observer plane: ") + &e),
        };
        let proj = pos_vect.project(obs_plane);
*/
        //2. Compute the angle between I and the projection
        //let azimuth = Angle::from_degrees(42.0);//proj.angle(&I);

        //println!("{} {}", proj, I);

        /// Compute altitude angle //TODO: move it to a Frame conversion
        //let altitude = Angle::from_degrees(35f64);
        //1. Find vector that goes from observer to satellite (r - o)
        //2. Project that vector on the observer plane
        //3. Compute angle between the vector and its projection

        Ok(Observation {
            time,
            observer: *obs,
            position: pos_vect.change_frame(GCRF::new(time)),
            speed: speed_vect.change_frame(GCRF::new(time)),
            brightness: 0f64
        })
    }
}
