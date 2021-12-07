/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use chrono::{NaiveDate, Utc, Timelike, Datelike, TimeZone};
use tardis::sun::Sun;
use tardis::tle::TLE;
use tardis::traits::Observable;

use spice;


fn main() -> Result<(), String> {
    let tle_lines = vec![
        "ISS (ZARYA)                                                          ".as_bytes(),
        "1 25544U 98067A   21316.58314353 -.00007551  00000-0 -13101-3 0  9994".as_bytes(),
        "2 25544  51.6442 328.9484 0004731 186.1225 318.0089 15.48559922311590".as_bytes(),
    ];

    spice::furnsh(String::from("/home/detlev/Downloads/de405.bsp").as_str());
    //let et = spice::str2et("2021-DEC-12 16:00:00");
    //println!("{}", et);
    let (position, light_time) = spice::spkpos("MARS", 0.0, "J2000", "NONE", "SUN");
    println!("{:?} {:?}", position, light_time);

    /*
    let mont_royal_coordinates = Coordinates::new(
        45.508888,
        -73.561668,
    );

    let observer = Observer::new(mont_royal_coordinates, Utc::now());
    */
    let satellite = match TLE::from_lines(&tle_lines[1], &tle_lines[2], &tle_lines[0]) {
        Ok(s) => s,
        Err(err) => {
            return Err("Cannot parse TLE: ".to_owned() + &err);
        }
    };

    println!("TLE: {}", satellite);

    println!("Earth satellite name: {}", satellite.name());

    let obs = match satellite.observation() {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    println!("Current observation: {}", obs);

    let sun = Sun::new();
    println!("Sun: {}", sun.observation_at(Utc.ymd(2021, 12, 21).and_hms(12, 0, 0)).unwrap());

    Ok(())
}
