/**
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

///
/// Time
/// TODO: Move this to a time.rs file

/// Times at which a new leap second has been added
/// TODO: Is there a better way than "[f64; 27]" ? This table will grow at each new leap second
const TT_LEAP_SECONDS: [f64; 27] = [
    2441499.500000,
    2441683.500000,
    2442048.500000,
    2442413.500000,
    2442778.500000,
    2443144.500000,
    2443509.500000,
    2443874.500000,
    2444239.500000,
    2444786.500000,
    2445151.500000,
    2445516.500000,
    2446247.500000,
    2447161.500000,
    2447892.500000,
    2448257.500000,
    2448804.500000,
    2449169.500000,
    2449534.500000,
    2450083.500000,
    2450630.500000,
    2451179.500000,
    2453736.500000,
    2454832.500000,
    2456109.500000,
    2457204.500000,
    2457754.500000,
];

pub const JD_J2000: f64 = 2451545.0;

pub fn get_leap_seconds(jd: f64) -> u32 {
    let mut i: u32 = 1;
    while i < TT_LEAP_SECONDS.len() as u32 && jd < TT_LEAP_SECONDS[i as usize] {
        i += 1;
    }

    i + 10
}

//TODO: get a better understanding of this
pub fn jd_utc_to_tt(jd: f64) -> f64 {
    let ls = get_leap_seconds(jd) as f64;
    jd + (ls + 32.184) / 86400.0
}