/*
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
use crate::traits::{Frame, Framable};
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

// Geometry elements
pub struct Point {
    coordinates: [f64; 3],
    frame: Option<Rc<Frame>>,
}

impl Point {
    pub fn new(x: f64, y: f64, z: f64) -> Point
    {
        Point {
            coordinates: [x, y, z],
            frame: None,
        }
    }

    pub fn from_tuple(t: [f64; 3]) -> Point
    {
        Point {
            coordinates: t,
            frame: None,
        }
    }
}

impl Framable for Point {
    fn change_frame(&mut self, new_frame: Rc<dyn Frame>) {
        self.coordinates = match self.frame {
            Some(ref f) => new_frame.from_gcrf(f.to_gcrf(self.coordinates)),
            None => self.coordinates,
        };

        self.frame = Some(new_frame);
    }

    fn set_frame(&mut self, frame: Rc<dyn Frame>) {
        self.frame = Some(frame);
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let frame_name = match self.frame {
            Some(ref f) => f.name(),
            None => String::from("Naive"),
        };
        write!(f, "Point ({}) [{}, {}, {}]", frame_name, self.coordinates[0], self.coordinates[1], self.coordinates[2])
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

pub struct Vector {
    vector: [f64; 3],
    frame: Option<Rc<dyn Frame>>,
}

impl Vector {
    /// Create a Vector from the spherical coordinates
    pub fn from_spherical(a: Angle, b: Angle, length: f64) -> Vector
    {
        Vector::from_cartesian(
            b.radians().cos() * a.radians().sin() * length,
            b.radians().cos() * a.radians().cos() * length,
            b.radians().sin() * length,
        )
    }

    pub fn from_tuple(vector: [f64; 3]) -> Vector
    {
        Vector::from_cartesian(vector[0], vector[1], vector[2])
    }

    pub fn from_cartesian(a: f64, b: f64, c: f64) -> Vector
    {
        Vector {
            vector: [a, b, c],
            frame: None,
        }
    }

    pub fn length(&self) -> f64
    {
        (self[0].powi(2) + self[1].powi(2) + self[2].powi(2)).sqrt()
    }

    /// Return the Angle between this vector and the other
    pub fn angle(&self, other: &Vector) -> Angle
    {
        Angle::from_vectors(self, other)
    }

    pub fn to_point(&self) -> Point
    {
        Point::from_tuple(self.vector)
    }

    /// Return the vector projected on the given plane
    /*pub fn project(&self, plane: Plane) -> Vector
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

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let frame_name = match self.frame {
            Some(ref f) => f.name(),
            None => String::from("Naive"),
        };

        write!(f, "Vector ({}): [{}, {}, {}]", frame_name, self[0], self[1], self[2])
    }
}

impl Clone for Vector {
    fn clone(&self) -> Self {
        Vector::from_tuple(self.vector)
    }
}


impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Self) -> Self::Output {
        Vector::from_cartesian(
            self.vector[0] + rhs.vector[0],
            self.vector[1] + rhs.vector[1],
            self.vector[2] + rhs.vector[2],
        )
    }
}

impl<'a, 'b> Add<&'b Vector> for &'a Vector {
    type Output = Vector;

    fn add(self, other: &'b Vector) -> Vector
    {
        Vector::from_cartesian(
            self.vector[0] + other.vector[0],
            self.vector[1] + other.vector[1],
            self.vector[2] + other.vector[2])
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector::from_cartesian(
            self.vector[0] - rhs.vector[0],
            self.vector[1] - rhs.vector[1],
            self.vector[2] - rhs.vector[2],
        )
    }
}

impl<'a, 'b> Sub<&'b Vector> for &'a Vector {
    type Output = Vector;

    fn sub(self, other: &'b Vector) -> Self::Output {
        Vector::from_cartesian(
            self.vector[0] - other.vector[0],
            self.vector[1] - other.vector[1],
            self.vector[2] - other.vector[2],
        )
    }
}

impl Mul for Vector {
    type Output = f64;

    /// Return the scalar product of the 2 Vectors
    fn mul(self, rhs: Self) -> Self::Output {
        self[0] * rhs[0] + self[1] * rhs[1] + self[2] * rhs[2]
    }
}

impl<'a, 'b> Mul<&'b Vector> for &'a Vector {
    type Output = f64;

    /// Return the scalar product of the 2 Vectors
    fn mul(self, other: &'b Vector) -> Self::Output {
        self[0] * other[0] + self[1] * other[1] + self[2] * other[2]
    }
}


impl Index<usize> for Vector {
    type Output = f64;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.vector[idx]
    }
}


impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.vector[0].eq(&other.vector[0]) &&
            self.vector[1].eq(&other.vector[1]) &&
            self.vector[2].eq(&other.vector[2])
    }
}

impl Eq for Vector {}

impl Framable for Vector {
    fn change_frame(&mut self, new_frame: Rc<dyn Frame>) {
        self.vector = match self.frame {
            Some(ref f) => new_frame.from_gcrf(f.to_gcrf(self.vector)),
            None => self.vector,
        };

        self.frame = Some(new_frame);
    }

    fn set_frame(&mut self, frame: Rc<dyn Frame>) {
        self.frame = Some(frame);
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
    pub fn from_vectors(a: &Vector, b: &Vector) -> Angle
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
