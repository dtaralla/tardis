/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */
#[macro_use]
extern crate simple_error;

mod constants;
mod error;

mod algebra;
pub mod frames;
pub mod geometry;
mod kf5;
pub mod sun;
mod time;
pub mod tle;
pub mod traits;
pub mod utils;

pub use error::*;

#[cfg(feature = "viewer")]
pub mod viewer;
