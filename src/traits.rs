/*
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::rc::Rc;
use chrono::{
    DateTime,
    Utc
};
use crate::utils::Observation;

pub trait Framable {
    // This function will convert the coordinates of this element into the new_frame
    fn change_frame(&mut self, new_frame: Rc<dyn Frame>);

    // This function replaces the Frame of the element without conversion
    fn set_frame(&mut self, frame: Rc<dyn Frame>);
}

pub trait Frame {
    fn name(&self) -> String;
    //fn date_time(&self) -> DateTime<Utc>;

    ///
    /// Return true if the frame has a different configuration depending on time
    ///
    /// For a better Speed support, all Frames should be timed.
    //fn has_obs_time(&self) -> bool;

    // FIXME: Intermediate GCRF is good but might lack precision for objects that are far from earth.
    //  Using ICRS should be better for objects that are far from earth(other planets, other stars,...).
    //  But GCRF is better for near-earth objects.
    //  In a first time, let's focus on earth satellites and use GCRF.
    //  Later, a solution could be based on the elegant Astropy's TransformGraph system
    //  See https://github.com/astropy/astropy/blob/77208dd7d7265df382849de841c890b3af996323/astropy/coordinates/transformations.py#L76
    fn to_gcrf(&self, point: [f64; 3]) -> [f64; 3];
    fn from_gcrf(&self, point: [f64; 3]) -> [f64; 3];
}

pub trait Observable {
    fn name(&self) -> String;
    fn observation(&self) -> Result<Observation, String>;
    fn observation_at(&self, time: DateTime<Utc>) -> Result<Observation, String>;
}