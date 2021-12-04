/**
 * Copyright Contributors to the tardis project
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

use std::{
    fmt,
    ops::Add,
    ops::Sub,
    ops::Mul,
    cmp::Eq,
    cmp::PartialEq,
    f64::consts::PI,
};
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::rc::Rc;
use chrono::{DateTime, Timelike, Utc};
use sgp4::sgp4::SGP4;
use crate::traits::{Frame, FramedElement};
use crate::kf5::{nutation, precession};
use crate::time::{get_leap_seconds, jd_utc_to_tt};

pub enum RotationAxis {
    ZYZ,
    ZYX,
    XZX,
}

pub struct Matrix {
    values: [[f64; 3]; 3],
}

impl Display for Matrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix:\n [\n  {:?}\n  {:?}\n  {:?}\n ]", self.values[0], self.values[1], self.values[2])
    }
}

impl Matrix {
    pub fn new(values: [[f64; 3]; 3]) -> Matrix {
        Matrix {
            values
        }
    }

    pub fn determinant(&self) -> f64 {
        self.values[0][0] * self.values[1][1] * self.values[2][2] - self.values[0][0] * self.values[1][2] * self.values[2][1] +
            self.values[0][1] * self.values[1][0] * self.values[2][2] - self.values[0][1] * self.values[1][2] * self.values[2][0] +
            self.values[0][2] * self.values[1][0] * self.values[2][1] - self.values[0][2] * self.values[1][1] * self.values[2][0]
    }

    pub fn transpose(&self) -> Matrix {
        Matrix {
            values: [[self.values[0][0], self.values[1][0], self.values[2][0]],
                [self.values[0][1], self.values[1][1], self.values[2][1]],
                [self.values[0][2], self.values[1][2], self.values[2][2]]]
        }
    }

    pub fn invert(&self) -> Result<Matrix, String> {
        let det = self.determinant();

        if det.abs() - 0.0 < 1e-10 {
            return Err(String::from("Matrix cannot be inverted"));
        }

        let t = self.transpose();

        Ok(Matrix::new([[t.values[0][0] / det, t.values[0][1] / det, t.values[0][2] / det],
            [t.values[1][0] / det, t.values[1][1] / det, t.values[1][2] / det],
            [t.values[2][0] / det, t.values[2][1] / det, t.values[2][2] / det]]
        ))
    }

    pub fn compose(a: Matrix, b: Matrix) -> Matrix {
        let mut ret = Matrix {
            values: [
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0]
            ]
        };

        let mut i = 0;
        let mut j = 0;
        let mut k = 0;

        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    ret.values[i][j] += a.values[i][k] * b.values[k][j];
                }
            }
        }

        ret
    }

    ///
    /// Compute a rotation matrix from the 3 angles, with the given Rotation axis
    /// TODO: Understand the rotation axis argument
    /// TODO: Use Angles instead of f64
    pub fn rot_from_angles(a: f64, b: f64, c: f64, r: RotationAxis) -> Matrix {
        let sx = a.sin();
        let cx = a.cos();
        let sy = b.sin();
        let cy = b.cos();
        let sz = c.sin();
        let cz = c.cos();

        match r {
            RotationAxis::ZYZ => Matrix::new(
                [[cx * cz * cy - sx * sz, sx * cz * cy + cx * sz, -sy * cz],
                    [-cx * cy * sz - sx * cz, -sx * cy * sz + cx * cz, sy * sz],
                    [cx * sy, sx * sy, cy]]
            ),

            RotationAxis::ZYX => Matrix::new(
                [[cx * cy, cy * sx, -sy],
                    [sz * sy * cx - cz * sx, sz * sy * sx + cz * cx, sz * cy],
                    [cz * sy * cx + sz * sx, cz * sy * sx - sz * cx, cz * cy]]
            ),

            RotationAxis::XZX => Matrix::new(
                [[cy, cx * sy, sx * sy],
                    [-sy * cz, cx * cz * cy - sx * sz, sx * cz * cy + cx * sz],
                    [sy * sz, -cx * cy * sz - sx * cz, -sx * cy * sz + cx * cz]]
            ),
        }
    }

    pub fn rotate(&self, point: [f64; 3]) -> [f64; 3] {
        ///
        /// M x p:
        ///
        /// x1 x2 x3     xp
        /// y1 y2 y3  x  yp
        /// z1 z2 z3     zp
        ///
        /// xp' = x1 * xp + x2 * yp + x3 * zp
        /// yp' = y1 * xp + y2 * yp + y3 * zp
        /// zp' = z1 * xp + z2 * yp + z3 * zp
        ///

        let m = self.values;
        let p = point;

        [
            m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2],
            m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2],
            m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2]
        ]
    }
}

/*
TODO: This is a point with no Frame, used for convenience. A method
   framed(frame: Frame) -> Point<Frame>
 will allow creation of a framed Point
 This is useful to create a simple point and have its frame set later

struct NaivePoint {
    coordinates: [f64; 3]
}
*/

// Geometry elements
pub struct Point<T: Frame> {
    coordinates: [f64; 3],
    frame: T,
}

impl<T: Frame> Point<T> {
    pub fn new(x: f64, y: f64, z: f64) -> Point<T>
    {
        Point {
            coordinates: [x, y, z],
            frame: T::new(Utc::now()),
        }
    }
}

impl<T: Frame, U: Frame> FramedElement<U> for Point<T> {
    type Item = Point<U>;

    fn change_frame(&self, new_frame: U) -> Self::Item {
        let icrs_coord = self.frame.to_gcrf(self.coordinates);

        Point {
            coordinates: new_frame.from_gcrf(icrs_coord),
            frame: U::new(Utc::now()),
        }
    }
}

impl<T: Frame> fmt::Display for Point<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point: [{}, {}, {}]", self.coordinates[0], self.coordinates[1], self.coordinates[2])
    }
}

/// A plane is defined by 3 points
/*pub struct Plane {
    points: [Vector; 3],
    normal: Option<Vector>,
}

impl Plane {
    pub fn from_vectors(a: &Vector, b: &Vector, c: &Vector) -> Result<Plane, String>
    {
        if a == b || b == c || c == a {
            return Err(String::from("The vectors must all be different"));
        }

        Ok(Plane {
            points: [a.clone(), b.clone(), c.clone()],
            normal: None,
            //referential: Rc::clone(&referential)
        })
    }

    pub fn from_vectors_normal(normal: &Vector, b: &Vector, c: &Vector) -> Result<Plane, String>
    {
        return match Plane::from_vectors(normal, b, c) {
            Ok(mut p) => {
                p.normal = Some(normal.clone());
                Ok(p)
            },
            Err(e) => Err(e)
        };
    }

    pub fn normal_vector(&self) -> Vector
    {
        match self.normal {
            Some(ref v) => v.clone(),
            None => Vector::from_cartesian(0., 0., 0.) //TODO: Compute a normal vector
        }
    }
}*/

// Each referential will have a to_icrf() function to convert its coordinates in that frame.
// That way, each referential can use that format to convert the coordinates into its own frame.
pub struct Vector<T: Frame> {
    vector: [f64; 3],
    frame: T,
}

impl<T: Frame> Vector<T> {
    /// Create a Vector from the spherical coordinates
    pub fn from_spherical(a: Angle, b: Angle, length: f64) -> Vector<T>
    {
        Vector::<T>::from_cartesian(
            b.radians().cos() * a.radians().sin() * length,
            b.radians().cos() * a.radians().cos() * length,
            b.radians().sin() * length,
        )
    }

    pub fn from_tuple(vector: [f64; 3]) -> Vector<T>
    {
        Vector::from_cartesian(vector[0], vector[1], vector[2])
    }

    pub fn from_cartesian(a: f64, b: f64, c: f64) -> Vector<T>
    {
        Vector::<T> {
            vector: [a, b, c],
            frame: T::new(Utc::now()),
        }
    }

    pub fn length(&self) -> f64
    {
        (self[0].powi(2) + self[1].powi(2) + self[2].powi(2)).sqrt()
    }

    /// Return the Angle between this vector and the other
    pub fn angle(&self, other: &Vector<T>) -> Angle
    {
        Angle::from_vectors(self, other)
    }

    /// Return the vector projected on the given plane
    /*pub fn project(&self, plane: Plane) -> Vector<T>
    {
        let n = &plane.normal_vector();
        let mult = (self*n) / n.length().powi(2);

        self - &Vector::from_cartesian(
            mult * n[0],
            mult * n[1],
            mult * n[2],
        )
    }*/

    /// Return the polar angle of the spherical coordinates of the vector
    pub fn polar_angle(&self) -> Angle
    {
        todo!()
    }

    /// Return the azimuth angle of the spherical coordinates of the vector
    pub fn azimuth_angle(&self) -> Angle
    {
        todo!()
    }

    /// Return the radial distance of the spherical coordinates of the vector
    pub fn radial_distance(&self) -> f64
    {
        todo!()
    }

    pub fn is_null(&self) -> bool
    {
        self[0] == 0f64 && self[1] == 0f64 && self[2] == 0f64
    }
}

impl<T: Frame> fmt::Display for Vector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector<{}>: [{}, {}, {}]", self.frame.name(), self[0], self[1], self[2])
    }
}

impl<T: Frame> Clone for Vector<T> {
    fn clone(&self) -> Self {
        Vector::from_tuple(self.vector)
    }
}


impl<T: Frame> Add for Vector<T> {
    type Output = Vector<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Vector::from_cartesian(
            self.vector[0] + rhs.vector[0],
            self.vector[1] + rhs.vector[1],
            self.vector[2] + rhs.vector[2],
        )
    }
}

impl<'a, 'b, T: Frame> Add<&'b Vector<T>> for &'a Vector<T> {
    type Output = Vector<T>;

    fn add(self, other: &'b Vector<T>) -> Vector<T>
    {
        Vector::<T>::from_cartesian(
            self.vector[0] + other.vector[0],
            self.vector[1] + other.vector[1],
            self.vector[2] + other.vector[2])
    }
}

impl<T: Frame> Sub for Vector<T> {
    type Output = Vector<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector::<T>::from_cartesian(
            self.vector[0] - rhs.vector[0],
            self.vector[1] - rhs.vector[1],
            self.vector[2] - rhs.vector[2],
        )
    }
}

impl<'a, 'b, T: Frame> Sub<&'b Vector<T>> for &'a Vector<T> {
    type Output = Vector<T>;

    fn sub(self, other: &'b Vector<T>) -> Self::Output {
        Vector::<T>::from_cartesian(
            self.vector[0] - other.vector[0],
            self.vector[1] - other.vector[1],
            self.vector[2] - other.vector[2],
        )
    }
}

impl<T: Frame> Mul for Vector<T> {
    type Output = f64;

    /// Return the scalar product of the 2 Vectors
    fn mul(self, rhs: Self) -> Self::Output {
        self[0] * rhs[0] + self[1] * rhs[1] + self[2] * rhs[2]
    }
}

impl<'a, 'b, T: Frame> Mul<&'b Vector<T>> for &'a Vector<T> {
    type Output = f64;

    /// Return the scalar product of the 2 Vectors
    fn mul(self, other: &'b Vector<T>) -> Self::Output {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
    }
}


impl<T: Frame> Index<usize> for Vector<T> {
    type Output = f64;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.vector[idx]
    }
}


impl<T: Frame> PartialEq for Vector<T> {
    fn eq(&self, other: &Self) -> bool {
        self.vector[0].eq(&other.vector[0]) &&
            self.vector[1].eq(&other.vector[1]) &&
            self.vector[2].eq(&other.vector[2])
    }
}

impl<T: Frame> Eq for Vector<T> {}

impl<T: Frame, U: Frame> FramedElement<U> for Vector<T> {
    type Item = Vector<U>;

    fn change_frame(&self, new_frame: U) -> Self::Item {
        let gcrf_coord = self.frame.to_gcrf(self.vector);

        Vector {
            vector: new_frame.from_gcrf(gcrf_coord),
            frame: new_frame,
        }
    }
}

pub struct Angle {
    degrees: f64,
    radians: f64,
}

impl Angle {
    pub fn from_degrees(degrees: f64) -> Angle
    {
        Angle {
            degrees,
            radians: (degrees * PI) / 180.0,
        }
    }

    pub fn from_radians(radians: f64) -> Angle
    {
        Angle {
            degrees: (radians * 180.0) / PI,
            radians,
        }
    }

    /// Create an Angle form the value of the angle formed by the give vectors
    pub fn from_vectors<T: Frame>(a: &Vector<T>, b: &Vector<T>) -> Angle
    {
        let den = a.length() * b.length();
        if den == 0f64 {
            return Angle::from_degrees(0f64);
        }

        let cos = (a * b) / den;

        Angle::from_radians(cos.acos())
    }

    pub fn canonical(&self) -> Angle
    {
        Angle {
            degrees: self.degrees % 360.0,
            radians: self.radians % (2.0 * PI),
        }
    }

    pub fn degrees(&self) -> f64
    {
        self.degrees
    }

    pub fn radians(&self) -> f64
    {
        self.radians
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Angle: [{}°, {}rad]", self.degrees, self.radians)
    }
}
