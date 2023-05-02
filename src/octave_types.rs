use num::FromPrimitive;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Possible types that can be returned from Octave through this library. These can also be used to
/// create convenient inputs to a function created using `wrap`.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OctaveType {
    /// A scalar value, accounting for both integers and floats. The underlying type is `f64`.
    Scalar(f64),
    /// A complex scalar value, accounting for both integers and floats. The underlying type is a
    /// pair of `f64`s.
    ComplexScalar(f64, f64),
    /// A numerical matrix, accounting 2 dimensional matrices of scalars. The underlying type is
    /// `Vec<Vec<f64>>`.
    Matrix(Vec<Vec<f64>>),
    /// A string value, accounting for both single and double quote strings. The underlying type is
    /// `String`.
    String(String),
    /// A cell array, which is essentially a matrix of non-numeric types.  The underlying type is
    /// `Vec<Vec<OctaveType>>`.
    CellArray(Vec<Vec<OctaveType>>),
    /// Something a value might be empty. This is mostly for the implementation of `Default`.
    Empty,
    /// Sometimes a value might be an error too.
    Error(String),
}

impl Default for OctaveType {
    fn default() -> Self {
        OctaveType::Empty
    }
}

impl Display for OctaveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                OctaveType::Scalar(scalar) => {
                    format!("{scalar}")
                }
                OctaveType::ComplexScalar(im, re) => {
                    format!("{im}{re:+}i")
                }
                OctaveType::Matrix(vec) => {
                    format!("{vec:?}")
                }
                OctaveType::String(string) => {
                    format!("{string}")
                }
                OctaveType::CellArray(ot) => {
                    format!("{ot:?}")
                }
                OctaveType::Empty => {
                    format!("")
                }
                OctaveType::Error(message) => {
                    format!("Error: {message}")
                }
            }
        )
    }
}

#[derive(Debug)]
pub struct OctaveTryIntoError(String);

impl Display for OctaveTryIntoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for OctaveTryIntoError {}

impl OctaveType {
    /// Unwrap a scalar octave type into `f64`
    /// ```
    /// let x: f64 = mocktave::OctaveType::Scalar(0.0).try_into_f64().unwrap();
    /// ```
    pub fn try_into_f64(self) -> Result<f64, OctaveTryIntoError> {
        if let OctaveType::Scalar(value) = self {
            return Ok(value);
        } else {
            Err(OctaveTryIntoError(
                "This is not an `OctaveType::Scalar` and therefore cannot be converted into f64."
                    .to_string(),
            ))
        }
    }
    /// Unwrap a string octave type into `String`
    /// ```
    /// let x: String = mocktave::OctaveType::String("0.0".to_string()).try_into_string().unwrap();
    /// ```
    pub fn try_into_string(self) -> Result<String, OctaveTryIntoError> {
        if let OctaveType::String(value) = self {
            return Ok(value);
        } else {
            Err(OctaveTryIntoError(
                "This is not an instance of `OctaveType::String` and therefore cannot be converted into String."
                    .to_string(),
            ))
        }
    }
    /// Unwrap a matrix octave type into `Vec<Vec<f64>>`
    /// ```
    /// let x: Vec<Vec<f64>> = mocktave::OctaveType::Matrix(vec![vec![0.0_f64;2];2]).try_into_vec_f64().unwrap();
    /// ```
    pub fn try_into_vec_f64(self) -> Result<Vec<Vec<f64>>, OctaveTryIntoError> {
        if let OctaveType::Matrix(value) = self {
            return Ok(value);
        } else {
            Err(OctaveTryIntoError("This is not an instance of `OctaveType::Matrix` and therefore cannot be converted into Vec<Vec<f64>>.".to_string()))
        }
    }
    /// Unwrap a cell array octave type into `Vec<Vec<OctaveType>>`
    /// ```
    /// let x: Vec<Vec<mocktave::OctaveType>> = mocktave::OctaveType::CellArray(vec![vec![mocktave::OctaveType::Scalar(1.0)]]).try_into_vec_octave_type().unwrap();
    /// ```
    pub fn try_into_vec_octave_type(self) -> Result<Vec<Vec<OctaveType>>, OctaveTryIntoError> {
        if let OctaveType::CellArray(value) = self {
            return Ok(value);
        } else {
            Err(OctaveTryIntoError("This is not an instance of `OctaveType::CellArray` and therefore cannot be converted into Vec<Vec<OctaveType>>.".to_string()))
        }
    }
    /// Unwrap an Empty octave type into `()`
    /// ```
    /// let x: () = mocktave::OctaveType::default().try_into_empty().unwrap();
    /// ```
    pub fn try_into_empty(self) -> Result<(), OctaveTryIntoError> {
        if let OctaveType::Empty = self {
            return Ok(());
        } else {
            Err(OctaveTryIntoError("This is not an instance of OctaveType::Empty and therefore cannot be converted into ().".to_string()))
        }
    }
}

// Implement into empty
impl From<OctaveType> for () {
    fn from(_value: OctaveType) -> Self {
        ()
    }
}

// Implement into f32
impl From<OctaveType> for f32 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as f32
    }
}

// Implement into f64
impl From<OctaveType> for f64 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap()
    }
}

// Implement into isize
impl From<OctaveType> for isize {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as isize
    }
}

// Implement into i8
impl From<OctaveType> for i8 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i8
    }
}

// Implement into i16
impl From<OctaveType> for i16 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i16
    }
}

// Implement into i32
impl From<OctaveType> for i32 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i32
    }
}

// Implement into i64
impl From<OctaveType> for i64 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i64
    }
}

// Implement into i128
impl From<OctaveType> for i128 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i128
    }
}

// Implement into usize
impl From<OctaveType> for usize {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as usize
    }
}

// Implement into u8
impl From<OctaveType> for u8 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u8
    }
}

// Implement into u16
impl From<OctaveType> for u16 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u16
    }
}

// Implement into u32
impl From<OctaveType> for u32 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u32
    }
}

// Implement into u64
impl From<OctaveType> for u64 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u64
    }
}

// Implement into u128
impl From<OctaveType> for u128 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u128
    }
}

// Implement into `Vec<Vec<T>>`
impl<T: From<OctaveType> + num::traits::AsPrimitive<T>> From<OctaveType> for Vec<Vec<T>> {
    fn from(value: OctaveType) -> Self {
        value
            .try_into_vec_f64()
            .unwrap()
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|el| T::from(OctaveType::Scalar(el)))
                    .collect::<Vec<T>>()
            })
            .collect::<Vec<Vec<T>>>()
    }
}

// // Implement into Vec<i32>
// impl<T: FromPrimitive> From<OctaveType> for Vec<T> {
//     fn from(value: OctaveType) -> Self {
//         let new = value.try_into_vec_f64().unwrap();
//
//         let w = new.len();
//         let h = new[0].len();
//
//         if w == 1 {
//             new[0]
//                 .clone()
//                 .into_iter()
//                 .map(|el| T::from(el))
//                 .collect::<Vec<T>>()
//         } else if h == 1 {
//             new.clone()
//                 .into_iter()
//                 .map(|el| T::from(el))
//                 .collect::<Vec<T>>()
//         } else {
//             panic!()
//         }
//     }
// }

trait MarkerType {}
impl MarkerType for i32 {}

// Implement into Vec<i32>
impl<T: MarkerType + From<OctaveType>> From<OctaveType> for Vec<T> {
    fn from(value: OctaveType) -> Self {
        let new: Vec<Vec<f64>> = value.try_into_vec_f64().unwrap();

        let w = new.len();
        let h = new[0].len();

        if w == 1 {
            new[0]
                .clone()
                .into_iter()
                .map(|el| T::from(OctaveType::Scalar(el)))
                .collect::<Vec<T>>()
        } else if h == 1 {
            new.clone()
                .into_iter()
                .map(|el| T::from(OctaveType::Scalar(el[0])))
                .collect::<Vec<T>>()
        } else {
            panic!()
        }
    }
}

// Implement into String
impl From<OctaveType> for String {
    fn from(value: OctaveType) -> Self {
        value.try_into_string().unwrap()
    }
}

// Implement into Vec<Vec<OctaveType>
impl From<OctaveType> for Vec<Vec<OctaveType>> {
    fn from(value: OctaveType) -> Self {
        value.try_into_vec_octave_type().unwrap()
    }
}
