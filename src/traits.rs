use std::rc::Rc;
use chrono::{
    DateTime,
    Utc
};
use crate::geometry::Referential;
use crate::utils::{
    Coordinates,
    Observer,
    Observation
};

pub trait ReferencedElement {
    type Item: Sized;

    //fn set_referential(&self, referential: Rc<Referential>); //FIXME: This could replace all the `with_ref` functions
    fn change_referential(&self, referential: Rc<Referential>) -> Self::Item;
    fn referential(&self) -> Rc<Referential>;
}

pub trait Observable {
    fn name(&self) -> String;
    fn observation(&self, observer: &Observer) -> Result<Observation, String>;
    fn observation_at(&self, observer: &Observer, time: DateTime<Utc>) -> Result<Observation, String>;
}