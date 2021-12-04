/**
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

pub fn evalpoly(val: f64, coeffs: Vec<f64>) -> f64
{
    let mut ret: f64 = 0.0;
    let mut i: i32 = 0;

    for c in coeffs {
        ret += c * val.powi(i);
        i += 1;
    }

    ret
}