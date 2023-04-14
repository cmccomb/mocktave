#![warn(clippy::all)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

extern crate core;

use bollard::{
    container::{Config, RemoveContainerOptions},
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    Docker,
};
use futures_util::{stream::StreamExt, TryStreamExt};

pub mod cookbook;
mod results;
pub use results::{InterpreterResults, OctaveType};

/// Evaluate a few lines of Octave code and extract the results.
/// ```
/// let res = mocktave::eval("a = 5+2");
/// assert_eq!(res.get_scalar_named("a").unwrap(), 7_f64);
/// ```
/// ```
/// let res = mocktave::eval("a = ones(2, 2)");
/// assert_eq!(res.get_matrix_named("a").unwrap(), vec![vec![1.0_f64; 2]; 2]);
/// ```
/// ```
/// let res = mocktave::eval("a = 'asdf'");
/// assert_eq!(res.get_string_named("a").unwrap(), "asdf");
/// ```
pub fn eval(input: &str) -> InterpreterResults {
    Interpreter::default().eval(input)
}

/// Create a persistent interpreter that can call a single container multiple times, resulting in
/// more efficiency code execution.
/// ```
/// let mut interp = mocktave::Interpreter::default();
/// let res1 = interp.eval("a = 5+2");
/// assert_eq!(res1.get_scalar_named("a").unwrap(), 7_f64);
/// let res2 = interp.eval("a = ones(2, 2)");
/// assert_eq!(res2.get_matrix_named("a").unwrap(), vec![vec![1.0_f64; 2]; 2]);
/// let res3 = interp.eval("a = 'asdf'");
/// assert_eq!(res3.get_string_named("a").unwrap(), "asdf");
/// ```
pub struct Interpreter {
    docker: Docker,
    id: String,
}

impl Default for Interpreter {
    fn default() -> Self {
        tokio::runtime::Runtime::new()
            .expect("Cannot create tokio runtime")
            .block_on(async {
                let docker = Docker::connect_with_socket_defaults()
                    .expect("Could not connect with socket defaults");
                docker
                    .create_image(
                        Some(CreateImageOptions {
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

                let alpine_config = Config {
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
            })
    }
}

impl Interpreter {
    /// This function does the heavy lifting in the interpreter struct.
    pub fn eval(&self, input: &str) -> InterpreterResults {
        tokio::runtime::Runtime::new()
            .expect("Cannot create tokio runtime")
            .block_on(async {
                // non interactive
                let exec = self
                    .docker
                    .create_exec(
                        &self.id.clone(),
                        CreateExecOptions {
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

                if let StartExecResults::Attached { mut output, .. } = self
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

                InterpreterResults::from(output_text.join(""))
            })
    }
}

impl Drop for Interpreter {
    fn drop(&mut self) {
        tokio::runtime::Runtime::new()
            .expect("Cannot create tokio runtime to to remove container")
            .block_on(self.docker.remove_container(
                &self.id.clone(),
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            ))
            .expect("Could not remove container.");
    }
}
