#![warn(clippy::all)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod cookbook;
mod interpreter_results;
pub use interpreter_results::InterpreterResults;
mod octave_types;
pub use octave_types::OctaveType;

#[cfg(not(any(feature = "docker", feature = "native")))]
compile_error!("Enable a backend: `bundled`, `native`, or the default `docker` feature.");

#[cfg(feature = "docker")]
mod docker;
#[cfg(feature = "native")]
mod native;
#[cfg(feature = "native")]
pub use native::{NativeError, NativeInterpreter};
#[cfg(feature = "bundled")]
mod bundled;
#[cfg(any(feature = "docker", feature = "native"))]
mod interpreter;
#[cfg(any(feature = "docker", feature = "native"))]
pub use interpreter::{Backend, Error, Interpreter};

/// Evaluate a few lines of Octave code and extract the results.
/// ```
/// let res = mocktave::eval("a = 5+2");
/// assert_eq!(res.get_scalar("a").unwrap(), 7_f64);
/// ```
/// ```
/// let res = mocktave::eval("a = ones(2, 2)");
/// assert_eq!(res.get_matrix("a").unwrap(), vec![vec![1.0_f64; 2]; 2]);
/// ```
/// ```
/// let res = mocktave::eval("a = 'asdf'");
/// assert_eq!(res.get_string("a").unwrap(), "asdf");
/// ```
#[cfg(any(feature = "docker", feature = "native"))]
pub fn eval(input: &str) -> InterpreterResults {
    try_eval(input).unwrap_or_else(|error| panic!("{error}"))
}

/// Evaluate code using the backend selected by Cargo features.
///
/// With default features disabled, `bundled` selects the private runtime and
/// `native` selects an installed Octave. If `docker` is enabled it takes priority.
/// Initialization and execution failures are returned as errors.
///
/// ```
/// let results = mocktave::try_eval("x = primes(10);")?;
/// assert_eq!(results.get_matrix("x"), Some(vec![vec![2.0, 3.0, 5.0, 7.0]]));
/// # Ok::<(), mocktave::Error>(())
/// ```
#[cfg(any(feature = "docker", feature = "native"))]
pub fn try_eval(input: &str) -> Result<InterpreterResults, Error> {
    Interpreter::new(Backend::default())?.try_eval(input)
}

/// This function provides the ability to wrap Octave functions for convenient later use.
/// ```
/// let primes = mocktave::wrap("primes".into());
/// let all_primes_less_than_100: Vec<Vec<i32>> = primes([100]);
/// assert_eq!(all_primes_less_than_100, vec![vec![2_i32, 3, 5, 7,
///     11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67,
///     71, 73, 79, 83, 89, 97]])
/// ```
/// Use functions with multiple inputs
/// ```
/// let max = mocktave::wrap("max".into());
/// let should_be_101: i32 = max([100, 101]);
/// assert_eq!(should_be_101, 101_i32);
/// ```
/// And even use functions with disimilar types
/// ```
/// use mocktave::OctaveType;
/// let norm = mocktave::wrap("norm".into());
/// let x = [
///     OctaveType::Matrix(vec![vec![0.0; 2]; 2]),
///     OctaveType::Scalar(2.0)
/// ];
/// let should_be_zero: f64 = norm(x);
/// assert_eq!(should_be_zero, 0.0_f64);
/// ```
#[cfg(any(feature = "docker", feature = "native"))]
pub fn wrap<Y, Z>(function: String) -> Box<dyn Fn(Y) -> Z>
where
    Y: IntoIterator,
    <Y as IntoIterator>::Item: ToString,
    Z: From<OctaveType>,
{
    Box::new(move |inputs| {
        let mut args = vec![String::new(); 0];
        for input in inputs.into_iter() {
            args.push(input.to_string());
        }
        Z::from(
            eval(
                &("result_of_function = ".to_owned()
                    + function.as_str()
                    + "("
                    + &args.join(", ")
                    + ")"),
            )
            .get_unchecked("result_of_function"),
        )
    })
}
