/* This is a wrapper for the C SGP4 algoritm.
 * It provides a rust-like interface to use the sgp4 algorithm.
 */

use std;
use std::{
    mem,
    fmt,
    rc::Rc,
    cell::RefCell,
    f64::consts::PI,
};

use chrono::{
    Utc,
    DateTime,
    Datelike,
    Timelike
};

use crate::{
    ElsetRec,
    sgp4,
    sgp4init,
    jday
};

///
/// # Represent an error of the SGP4 algorithm
///
pub enum SGP4Error {
    MeanElements, /* 1 - mean elements, ecc >= 1.0 or ecc < -0.001 or a < 0.95 er */
    MeanMotion,   /* 2 - mean motion less than 0.0 */
    PertElements, /* 3 - pert elements, ecc < 0.0  or  ecc > 1.0 */
    SemiLatus,    /* 4 - semi-latus rectum < 0.0 */
    Epoch,        /* 5 - epoch elements are sub-orbital */
    Decayed,      /* 6 - satellite has decayed */
    UnknownError(i32),
}

impl SGP4Error {
    fn from_code(code: i32) -> SGP4Error
    {
        match code {
            1 => SGP4Error::MeanElements,
            2 => SGP4Error::MeanMotion,
            3 => SGP4Error::PertElements,
            4 => SGP4Error::SemiLatus,
            5 => SGP4Error::Epoch,
            6 => SGP4Error::Decayed,
            _ => SGP4Error::UnknownError(code),
        }
    }
}

impl fmt::Display for SGP4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SGP4Error::MeanElements => write!(f, "Mean elements error"),
            SGP4Error::MeanMotion => write!(f, "Mean motion error"),
            SGP4Error::PertElements => write!(f, "Pert elements error"),
            SGP4Error::SemiLatus => write!(f, "Semi Latus error"),
            SGP4Error::Epoch => write!(f, "Epoch error"),
            SGP4Error::Decayed => write!(f, "Decayed error"),
            SGP4Error::UnknownError(x) => write!(f, "Unknown error {}", x),
        }
    }
}

/// # Represent an SGP4 operation mode
/// Afscp if for the Air Force Space Command version of the algorithm,
/// Improved is the same algorithm with performance improvements
pub enum OpsMode {
    Afspc,
    Improved,
}

impl OpsMode {
    pub fn to_char(&self) -> char
    {
        match self {
            OpsMode::Afspc => 'a',
            OpsMode::Improved => 'i',
        }
    }
}

/// # Represent the Gravitational Constants set
/// SGP4 supports 2 gravitational constants sets.
/// These are either the version of 1972 or 1984.
///
/// Note that the set of 1984 is more precise but it is recommended to use the one that was used
/// to generate the TLE, which is usually 1972.
pub enum ConstantsSet {
    Set72,
    Set84,
}

impl ConstantsSet {
    pub fn to_int(&self) -> i32 {
        match self {
            ConstantsSet::Set72 => 72,
            ConstantsSet::Set84 => 84,
        }
    }
}

/// # A SGP4 computation result
/// It can be used to get a satellit speed, altitude, position vector and velocity vector
pub struct SGP4Result {
    position: [f64; 3],
    velocity: [f64; 3],
    time: DateTime<Utc>,
    satrec: Rc<RefCell<ElsetRec>>
}

impl SGP4Result {
    /// Return the computed altitude above the ground in kilometers
    /// The ground is considered to be at the average earth radius
    pub fn altitude(&self) -> f64
    {
        let r = self.position;
        let rec = (*self.satrec).borrow();

        (r[0]*r[0] + r[1]*r[1] + r[2]*r[2]).sqrt() - rec.radiusearthkm
    }

    /// Return the computed velocity in km/s
    pub fn velocity(&self) -> f64
    {
        let v = self.velocity;

        (v[0]* v[0] + v[1]* v[1] + v[2]* v[2]).sqrt()
    }

    /// Return the computed position vector in [km, km, km]
    pub fn position_vect(&self) -> [f64; 3]
    {
        self.position
    }

    /// Return the computed velocity vector in [km/s, km/s, km/s]
    pub fn velocity_vect(&self) -> [f64; 3]
    {
        self.velocity
    }

    /// Return the time of this result
    ///
    /// This is not the local time at which it was computed,
    /// This is the time that yielded the result.
    pub fn time(&self) -> DateTime<Utc>
    {
        self.time
    }
}

impl fmt::Display for SGP4Result {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SGP4Result: [position: {:?} km, velocity: {:?} km/s, time: {}]",
               self.position,
               self.velocity,
               self.time)
    }
}

/// SGP4 C library wrapper
pub struct SGP4 {
    /// The C structure that is used by the SGP4 library,
    /// It is set in a RefCell because the FFI calls need to access it as mutable but the
    /// interface has no reason to force a mutable variable
    satrec: Rc<RefCell<ElsetRec>>,
    epoch: DateTime<Utc>,
}

impl SGP4 {

    /// Initialize the SGP4 variables from the provided elements.
    /// It returns a SGP4 struct that can be used to compute an SGP4Result
    /// or an error corresponding to what the SGP4 library returned
    ///
    /// mean_motion is given in revolutions per day
    ///
    pub fn new(mode: OpsMode,
               const_set: ConstantsSet,
               bstar: f64,
               eccentricity: f64,
               epoch: DateTime<Utc>,
               arg_perigee: f64,
               inclination: f64,
               mean_anomaly: f64,
               mean_motion: f64,
               ascending_node: f64) -> Result<SGP4, SGP4Error>
    {
        let mut satrec: ElsetRec;
        let ret: i32;

        // This is used to convert the mean motion to an angular speed in rad / minute
        // from the argument that is in revolutions (2*PI)rad per day (1440 minutes))
        let xpdotp = (2.0 * PI) / 1440f64;

        unsafe {
            satrec = mem::zeroed();
        }

        satrec.whichconst = const_set.to_int();
        satrec.bstar = bstar;
        satrec.ecco = eccentricity;
        satrec.argpo = arg_perigee;
        satrec.inclo = inclination;
        satrec.mo = mean_anomaly;
        satrec.no_kozai = mean_motion * xpdotp;
        satrec.nodeo = ascending_node;

        unsafe {
            ret = sgp4init(mode.to_char() as std::os::raw::c_char, &mut satrec as *mut ElsetRec);
        }

        if ret == 0 {
            return Err(SGP4Error::from_code(satrec.error));
        }

        Ok(SGP4 {
            satrec: Rc::new(RefCell::new(satrec)),
            epoch
        })
    }

    /// Compute the velocity and position vectors at the given time.
    pub fn compute(&self, time: DateTime<Utc>) -> Result<SGP4Result, SGP4Error>
    {
        let ret: i32;
        let minutes: f64 = time.signed_duration_since(self.epoch).num_seconds() as f64 / 60f64;

        let mut r: [f64; 3] = [0f64, 0f64, 0f64]; //FIXME: is this allowed to be passed to a C function (use as_mut_ptr() ?) ?
        let mut v: [f64; 3] = [0f64, 0f64, 0f64];

        let mut rec = (*self.satrec).borrow_mut();

        unsafe {
            ret = sgp4(&mut *rec as *mut ElsetRec, minutes, &mut r as *mut f64, &mut v as *mut f64);
        }

        // The sgp4 function returns a boolean value --> 0 is an error
        if ret == 0 {
            return Err(SGP4Error::from_code(rec.error));
        }

        Ok(SGP4Result {
            position: r,
            velocity: v,
            time,
            satrec: Rc::clone(&self.satrec)
        })
    }

    pub fn earth_radius(&self) -> f64
    {
        let rec = (*self.satrec).borrow();
        rec.radiusearthkm
    }

    //Compute the Julian Day corresponding to the given date/time
    pub fn julian_day(time: DateTime<Utc>) -> f64
    {
        let mut jd: f64 = 0.0;
        let mut jdfrac: f64 = 0.0;

        unsafe {
            jday(time.date().year(),
                       time.date().month() as i32,
                       time.date().day() as i32,
                       time.time().hour() as i32,
                       time.time().minute() as i32,
                       time.time().second() as f64 + (time.time().nanosecond() as f64) / 10e9,
                       &mut jd as *mut f64, &mut jdfrac as *mut f64);
        }

        jd + jdfrac
    }
}
