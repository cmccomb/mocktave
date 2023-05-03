use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Possible types that can be returned from Octave through this library.
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

/// Implementation of the Default trait
/// ```
/// use mocktave::OctaveType;
/// let dflt = OctaveType::default();
/// assert_eq!(dflt, OctaveType::Empty);
/// ```
impl Default for OctaveType {
    fn default() -> Self {
        OctaveType::Empty
    }
}

/// Implementation of the Display trait
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

/// Mark a lot of Primitive types
trait Primitive {}
impl Primitive for f32 {}
impl Primitive for f64 {}
impl Primitive for isize {}
impl Primitive for i8 {}
impl Primitive for i16 {}
impl Primitive for i32 {}
impl Primitive for i64 {}
impl Primitive for i128 {}
impl Primitive for usize {}
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for u128 {}

/// Convert an `OctaveType::Empty` into an '()'
/// ```
/// use mocktave::OctaveType;
/// let x: () = OctaveType::Empty.into();
/// assert_eq!((), x);
/// ```
impl From<OctaveType> for () {
    fn from(_value: OctaveType) -> Self {
        ()
    }
}

/// Convert an `()` into an `OctaveType::Empty`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = ().into();
/// assert_eq!(x, OctaveType::Empty)
/// ```
impl From<()> for OctaveType {
    fn from(_value: ()) -> Self {
        OctaveType::Empty
    }
}

/// Convert an `OctaveType::Scalar` into an 'f32'
/// ```
/// use mocktave::OctaveType;
/// let x: f32 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1.0_f32);
/// ```
impl From<OctaveType> for f32 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as f32
    }
}

/// Convert an `f32` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1.0_f32.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<f32> for OctaveType {
    fn from(value: f32) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into an 'f64'
/// ```
/// use mocktave::OctaveType;
/// let x: f64 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1.0_f64);
/// ```
impl From<OctaveType> for f64 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap()
    }
}

/// Convert an `f64` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1.0_f64.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<f64> for OctaveType {
    fn from(value: f64) -> Self {
        OctaveType::Scalar(value)
    }
}

/// Convert an `OctaveType::Scalar` into a `u64`
/// ```
/// use mocktave::OctaveType;
/// let x: isize = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_isize);
/// ```
impl From<OctaveType> for isize {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as isize
    }
}

/// Convert a `isize` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_isize.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<isize> for OctaveType {
    fn from(value: isize) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into an 'i8'
/// ```
/// use mocktave::OctaveType;
/// let x: i8 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_i8);
/// ```
impl From<OctaveType> for i8 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i8
    }
}

/// Convert a `i8` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_i8.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<i8> for OctaveType {
    fn from(value: i8) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into an 'i16'
/// ```
/// use mocktave::OctaveType;
/// let x: i16 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_i16);
/// ```
impl From<OctaveType> for i16 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i16
    }
}

/// Convert a `i16` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_i16.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<i16> for OctaveType {
    fn from(value: i16) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into an 'i32'
/// ```
/// use mocktave::OctaveType;
/// let x: i32 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_i32);
/// ```
impl From<OctaveType> for i32 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i32
    }
}

/// Convert a `i32` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_i32.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<i32> for OctaveType {
    fn from(value: i32) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into an 'i64'
/// ```
/// use mocktave::OctaveType;
/// let x: i64 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_i64);
/// ```
impl From<OctaveType> for i64 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i64
    }
}

/// Convert a `i64` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_i64.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<i64> for OctaveType {
    fn from(value: i64) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into an 'i128'
/// ```
/// use mocktave::OctaveType;
/// let x: i128 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_i128);
/// ```
impl From<OctaveType> for i128 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as i128
    }
}

/// Convert a `i128` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_i128.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<i128> for OctaveType {
    fn from(value: i128) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into a 'usize'
/// ```
/// use mocktave::OctaveType;
/// let x: usize = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_usize);
/// ```
impl From<OctaveType> for usize {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as usize
    }
}

/// Convert a `usize` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_usize.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<usize> for OctaveType {
    fn from(value: usize) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into a 'u8'
/// ```
/// use mocktave::OctaveType;
/// let x: u8 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_u8);
/// ```
impl From<OctaveType> for u8 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u8
    }
}

/// Convert a `u8` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_u8.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<u8> for OctaveType {
    fn from(value: u8) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into a 'u16'
/// ```
/// use mocktave::OctaveType;
/// let x: u16 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_u16);
/// ```
impl From<OctaveType> for u16 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u16
    }
}

/// Convert a `u16` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_u16.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<u16> for OctaveType {
    fn from(value: u16) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into a 'u32'
/// ```
/// use mocktave::OctaveType;
/// let x: u32 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_u32);
/// ```
impl From<OctaveType> for u32 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u32
    }
}

/// Convert a `u32` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_u32.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<u32> for OctaveType {
    fn from(value: u32) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into a 'u64'
/// ```
/// use mocktave::OctaveType;
/// let x: u64 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_u64);
/// ```
impl From<OctaveType> for u64 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u64
    }
}

/// Convert a `u64` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_u64.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<u64> for OctaveType {
    fn from(value: u64) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Scalar` into a `u128`
/// ```
/// use mocktave::OctaveType;
/// let x: u128 = OctaveType::Scalar(1.0).into();
/// assert_eq!(x, 1_u128);
/// ```
impl From<OctaveType> for u128 {
    fn from(value: OctaveType) -> Self {
        value.try_into_f64().unwrap() as u128
    }
}

/// Convert a `u128` into an `OctaveType::Scalar`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = 1_u128.into();
/// assert_eq!(x, OctaveType::Scalar(1.0))
/// ```
impl From<u128> for OctaveType {
    fn from(value: u128) -> Self {
        OctaveType::Scalar(value as f64)
    }
}

/// Convert an `OctaveType::Matrix` into a `Vec<Vec<T>>`
/// ```
/// use mocktave::OctaveType;
/// let x: Vec<Vec<usize>> = OctaveType::Matrix(vec![vec![0.0; 2]; 2]).into();
/// assert_eq!(x, vec![vec![0_usize; 2]; 2])
/// ```
/// ```
/// use mocktave::{OctaveType, wrap};
/// let ones = wrap("ones".into());
/// let mat: OctaveType = ones([3]);
/// let x: Vec<Vec<usize>> = mat.into();
/// assert_eq!(x, vec![vec![1_usize; 3]; 3])
/// ```
impl<T: From<OctaveType> + Primitive> From<OctaveType> for Vec<Vec<T>> {
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

/// Convert a `Vec<Vec<T>>` into an `OctaveType::Matrix`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = vec![vec![0.0; 2]; 2].into();
/// assert_eq!(x, OctaveType::Matrix(vec![vec![0.0; 2]; 2]))
/// ```
impl<T: Into<OctaveType> + Primitive> From<Vec<Vec<T>>> for OctaveType {
    fn from(value: Vec<Vec<T>>) -> Self {
        OctaveType::Matrix(
            value
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|el| el.into().try_into_f64().unwrap())
                        .collect::<Vec<f64>>()
                })
                .collect::<Vec<Vec<f64>>>(),
        )
    }
}

/// Convert an `OctaveType::Matrix` into a `Vec<T>`
/// ```
/// use mocktave::OctaveType;
/// let x: Vec<usize> = OctaveType::Matrix(vec![vec![0.0; 5]; 1]).into();
/// assert_eq!(x, vec![0_usize; 5])
/// ```
impl<T: Primitive + From<OctaveType>> From<OctaveType> for Vec<T> {
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

/// Convert a `Vec<T>` into an `OctaveType::Matrix`
/// ```
/// use mocktave::OctaveType;
/// let x: OctaveType = vec![0.0; 5].into();
/// assert_eq!(x, OctaveType::Matrix(vec![vec![0.0; 5]; 1]))
/// ```
impl<T: Into<OctaveType> + Primitive> From<Vec<T>> for OctaveType {
    fn from(value: Vec<T>) -> Self {
        OctaveType::Matrix(vec![value
            .into_iter()
            .map(|el| el.into().try_into_f64().unwrap())
            .collect::<Vec<f64>>()])
    }
}

/// Convert an `OctaveType::String` into a proper rust `String`
/// ```
/// use mocktave::OctaveType;
/// let x: String = OctaveType::String("asdf".to_string()).into();
/// assert_eq!(x, "asdf".to_string());
/// ```
impl From<OctaveType> for String {
    fn from(value: OctaveType) -> Self {
        value.try_into_string().unwrap()
    }
}

/// Convert an `OctaveType::CellArray` into `Vec<Vec<OctaveType>>`
/// ```
/// use mocktave::OctaveType;
/// let x: Vec<Vec<OctaveType>> = OctaveType::CellArray(vec![vec![OctaveType::default(); 1]]).into();
/// assert_eq!(x, vec![vec![OctaveType::default(); 1]]);
/// ```
impl From<OctaveType> for Vec<Vec<OctaveType>> {
    fn from(value: OctaveType) -> Self {
        value.try_into_vec_octave_type().unwrap()
    }
}
