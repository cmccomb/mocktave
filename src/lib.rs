#![warn(clippy::all)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod cookbook;
mod interpreter_results;
pub use interpreter_results::InterpreterResults;
mod octave_types;
pub use octave_types::OctaveType;

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
pub fn eval(input: &str) -> InterpreterResults {
    Interpreter::default().eval(input)
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

/// Create a persistent interpreter that can call a single container multiple times, resulting in
/// more efficiency code execution.
/// ```
/// let mut interp = mocktave::Interpreter::default();
/// let res1 = interp.eval("a = 5+2");
/// assert_eq!(res1.get_scalar("a").unwrap(), 7_f64);
/// let res2 = interp.eval("a = ones(2, 2)");
/// assert_eq!(res2.get_matrix("a").unwrap(), vec![vec![1.0_f64; 2]; 2]);
/// let res3 = interp.eval("a = 'asdf'");
/// assert_eq!(res3.get_string("a").unwrap(), "asdf");
/// ```
#[cfg(all(
    feature = "docker",
    not(feature = "brew-local"),
    not(feature = "brew-src")
))]
pub struct Interpreter {
    docker: bollard::Docker,
    id: String,
}
#[cfg(any(feature = "brew-src", feature = "brew-local"))]
pub struct Interpreter {}

impl Default for Interpreter {
    fn default() -> Self {
        #[cfg(all(
            feature = "docker",
            not(feature = "brew-local"),
            not(feature = "brew-src")
        ))]
        {
            use futures_util::{stream::StreamExt, TryStreamExt};
            return tokio::runtime::Runtime::new()
                .expect("Cannot create tokio runtime")
                .block_on(async {
                    let docker = bollard::Docker::connect_with_local_defaults()
                        .expect("Could not connect with local defaults");
                    docker
                        .create_image(
                            Some(bollard::image::CreateImageOptions {
                                from_image: "gnuoctave/octave",
                                tag: "8.1.0",
                                ..Default::default()
                            }),
                            None,
                            None,
                        )
                        .try_collect::<Vec<_>>()
                        .await
                        .expect("Could not create image.");

                    let alpine_config = bollard::container::Config {
                        image: Some("gnuoctave/octave:8.1.0"),
                        tty: Some(true),
                        ..Default::default()
                    };

                    let id = docker
                        .create_container::<&str, &str>(None, alpine_config)
                        .await
                        .expect("Could not create container.")
                        .id;

                    docker
                        .start_container::<String>(&id, None)
                        .await
                        .expect("Could not start container");

                    Interpreter { docker, id }
                });
        }

        #[cfg(all(feature = "brew-local", not(feature = "brew-src")))]
        return Interpreter {};

        #[cfg(feature = "brew-src")]
        return Interpreter {};
    }
}

impl Interpreter {
    /// This function does the heavy lifting in the interpreter struct.
    pub fn eval(&self, input: &str) -> InterpreterResults {
        #[cfg(all(
            feature = "docker",
            not(feature = "brew-local"),
            not(feature = "brew-src")
        ))]
        {
            use futures_util::{stream::StreamExt, TryStreamExt};
            return tokio::runtime::Runtime::new()
                .expect("Cannot create tokio runtime")
                .block_on(async {
                    // non interactive
                    let exec = self
                        .docker
                        .create_exec(
                            &self.id.clone(),
                            bollard::exec::CreateExecOptions {
                                attach_stdout: Some(true),
                                attach_stderr: Some(true),
                                cmd: Some(vec![
                                    "octave",
                                    "--eval",
                                    &(input.to_string() + "\n\nsave(\"-\", \"*\");"),
                                ]),
                                ..Default::default()
                            },
                        )
                        .await
                        .expect("Could not create command to execute.")
                        .id;

                    let mut output_text = vec!["".to_string(); 0];

                    if let bollard::exec::StartExecResults::Attached { mut output, .. } = self
                        .docker
                        .start_exec(&exec, None)
                        .await
                        .expect("Execution of command failed.")
                    {
                        while let Some(Ok(msg)) = output.next().await {
                            output_text.push(msg.to_string());
                            print!("{}", msg);
                        }
                    } else {
                        unreachable!();
                    }

                    return InterpreterResults::from(output_text.join(""));
                });
        }

        #[cfg(all(feature = "brew-local", not(feature = "brew-src")))]
        {
            let output = std::process::Command::new("octave")
                .arg("--eval")
                .arg(&(input.to_string() + "\n\nsave(\"-\", \"*\");"))
                .output()
                .expect("");

            return InterpreterResults::from(String::from_utf8(output.stdout).unwrap());
        }

        #[cfg(feature = "brew-src")]
        {
            let output = std::process::Command::new("octave")
                .arg("--eval")
                .arg(&(input.to_string() + "\n\nsave(\"-\", \"*\");"))
                .output()
                .expect("");

            return InterpreterResults::from(String::from_utf8(output.stdout).unwrap());
        }
    }
}

#[cfg(all(
    feature = "docker",
    not(feature = "brew-local"),
    not(feature = "brew-src")
))]
impl Drop for Interpreter {
    fn drop(&mut self) {
        tokio::runtime::Runtime::new()
            .expect("Cannot create tokio runtime to to remove container")
            .block_on(self.docker.remove_container(
                &self.id.clone(),
                Some(bollard::container::RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            ))
            .expect("Could not remove container.");
    }
}
