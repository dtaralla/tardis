use std::{
    fmt,
    ops::Add,
    ops::Sub,
    ops::Mul,
    cmp::Eq,
    cmp::PartialEq,
    f64::consts::PI,
};
use std::ops::Index;
use std::rc::Rc;
use crate::traits::ReferencedElement;

pub struct Referential {
    translation: Option<Vector>,
    rotation: Option<[Angle; 3]>,
    is_base: bool,
}

//TODO: Each geometrical element could implement a Referential Trait that makes it able to go from one to another
impl Referential {
    /// The base referential is the one placed at (0,0,0).
    /// All the other ones are based on this one
    /// TODO: Find a ways to use a Rc for this referential
    const BASE_REFERENTIAL: Referential = Referential {
        translation: None,
        rotation: None,
        is_base: true,
    };

    pub fn base_referential() -> Rc<Referential> {
        Rc::new(Referential::BASE_REFERENTIAL)
    }

    pub fn make_referential(&self, translation: Option<Vector>, rotation: Option<[Angle; 3]>) -> Referential
    {
        // We don't chain or keep references to parent referentials.
        // All Referential instance is against the base referential.
        // In a first time, only support making a referential from the base one.
        Referential {
            translation,
            rotation,
            is_base: false
        }
    }
}

/// A plane is defined by 3 points
pub struct Plane {
    points: [Vector; 3],
    normal: Option<Vector>,
    referential: Rc<Referential>,
}

impl Plane {
    pub fn from_vectors(a: &Vector, b: &Vector, c: &Vector) -> Result<Plane, String>
    {
        Plane::from_vectors_with_ref(a, b, c,Referential::base_referential())
    }

    pub fn from_vectors_normal(normal: &Vector, b: &Vector, c: &Vector, referential: Rc<Referential>) -> Result<Plane, String>
    {
        Plane::from_vectors_normal_with_ref(normal, b, c,Referential::base_referential())
    }

    pub fn from_vectors_with_ref(a: &Vector, b: &Vector, c: &Vector, referential: Rc<Referential>) -> Result<Plane, String>
    {
        if a == b || b == c || c == a {
            return Err(String::from("The vectors must all be different"));
        }

        Ok(Plane {
            points: [a.clone(), b.clone(), c.clone()],
            normal: None,
            referential: Rc::clone(&referential)
        })
    }

    pub fn from_vectors_normal_with_ref(normal: &Vector, b: &Vector, c: &Vector, referential: Rc<Referential>) -> Result<Plane, String>
    {
        return match Plane::from_vectors_with_ref(normal, b, c, referential) {
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
}

//TODO: Find a better name ?
impl ReferencedElement for Plane {
    type Item = Plane;

    fn change_referential(&self, referential: Rc<Referential>) -> Self::Item {
        //TODO: Perform the translation and rotation of the Vector
        //TODO: Set a reference to the new referential
        todo!()
    }

    fn referential(&self) -> Rc<Referential> {
        Rc::clone(&self.referential)
    }
}

// Each referential will have a to_icrf() function to convert its coordinates in that frame.
// That way, each referential can use that format to convert the coordinates into its own frame.
pub struct Vector {
    vector: [f64; 3],
    referential: Rc<Referential>,
}

impl Vector {
    /// Create a Vector from the cartesian coordinates
    pub fn from_cartesian(a: f64, b: f64, c: f64) -> Vector
    {
        Vector::from_cartesian_with_ref(a, b, c, Referential::base_referential())
    }

    /// Create a Vector from the spherical coordinates
    pub fn from_spherical(a: Angle, b: Angle, length: f64) -> Vector
    {
        Vector::from_cartesian(
            b.radians().cos() * a.radians().sin() * length,
            b.radians().cos() * a.radians().cos() * length,
            b.radians().sin() * length
        )
    }

    pub fn from_tuple(vector: [f64; 3]) -> Vector
    {
        Vector::from_cartesian(vector[0], vector[1], vector[2])
    }

    pub fn from_cartesian_with_ref(a: f64, b: f64, c: f64, referential: Rc<Referential>) -> Vector
    {
        Vector {
            vector: [a, b, c],
            referential: Rc::clone(&referential)
        }
    }

    /// Create a Vector from the spherical coordinates
    pub fn from_spherical_with_ref(a: Angle, b: Angle, length: f64, referential: Rc<Referential>) -> Vector
    {
        Vector::from_cartesian_with_ref(
            b.radians().cos() * a.radians().sin() * length,
            b.radians().cos() * a.radians().cos() * length,
            b.radians().sin() * length,
            referential
        )
    }

    pub fn from_tuple_with_ref(vector: [f64; 3], referential: Rc<Referential>) -> Vector
    {
        Vector::from_cartesian_with_ref(vector[0], vector[1], vector[2], referential)
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

    /// Return the vector projected on the given plane
    pub fn project(&self, plane: Plane) -> Vector
    {
        let n = &plane.normal_vector();
        let mult = (self*n) / n.length().powi(2);

        self - &Vector::from_cartesian(
            mult * n[0],
            mult * n[1],
            mult * n[2],
        )
    }

    pub fn is_null(&self) -> bool
    {
        self[0] == 0f64 && self[1] == 0f64 && self[2] == 0f64
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector: [{}, {}, {}]", self[0], self[1], self[2])
    }
}

impl Clone for Vector {
    fn clone(&self) -> Self {
        Vector::from_tuple_with_ref(self.vector, Rc::clone(&self.referential))
    }
}

//TODO: Find a better name ?
impl ReferencedElement for Vector {
    type Item = Vector;

    fn change_referential(&self, referential: Rc<Referential>) -> Self::Item {
        //TODO: Perform the translation and rotation of the Vector
        //TODO: Set a reference to the new referential
        todo!()
    }

    fn referential(&self) -> Rc<Referential> {
        Rc::clone(&self.referential)
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
            radians: self.radians % (2.0*PI),
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
