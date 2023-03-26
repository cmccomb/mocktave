#![warn(clippy::all)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bollard::{
    container::{Config, RemoveContainerOptions},
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    Docker,
};
use futures_util::{stream::StreamExt, TryStreamExt};
use std::num::ParseFloatError;
use std::{collections::HashMap, str::FromStr};

use human_regex::{
    any, beginning, digit, end, exactly, multi_line_mode, named_capture, one_or_more, or,
    printable, text, whitespace, word, zero_or_more, zero_or_one,
};

pub mod cookbook;

/// Contains the workspace that resulted from running the octave command in `eval`
#[derive(Debug)]
pub struct OctaveResults {
    /// Scalar variables
    scalars: HashMap<String, f64>,
    /// Matrix variables
    matrices: HashMap<String, Vec<Vec<f64>>>,
    /// String variables
    strings: HashMap<String, String>,
}

impl OctaveResults {
    /// Get a scalar by name
    pub fn get_scalar_named(&self, name: &str) -> Option<f64> {
        self.scalars.get(name).cloned()
    }
    /// Get a matrix by name
    pub fn get_matrix_named(&self, name: &str) -> Option<Vec<Vec<f64>>> {
        self.matrices.get(name).cloned()
    }
    /// Get a string by name
    pub fn get_string_named(&self, name: &str) -> Option<String> {
        self.strings.get(name).cloned()
    }
}

impl Default for OctaveResults {
    fn default() -> Self {
        OctaveResults {
            scalars: Default::default(),
            matrices: Default::default(),
            strings: Default::default(),
        }
    }
}

impl From<String> for OctaveResults {
    fn from(output: String) -> Self {
        let mut results = OctaveResults::default();

        let scalar_match = multi_line_mode(
            beginning()
                + text("# name: ")
                + named_capture(one_or_more(word()), "name")
                + text("\n# type: scalar\n")
                + named_capture(exactly(1, beginning() + one_or_more(any()) + end()), "data"),
        );

        for capture in scalar_match.to_regex().captures_iter(&*output) {
            let name = capture
                .name("name")
                .expect("Name not found")
                .as_str()
                .to_string();
            results.scalars.insert(
                name,
                f64::from_str(
                    &*capture
                        .name("data")
                        .expect("No value for scalar data.")
                        .as_str()
                        .replace("\n", ""),
                )
                .expect("Could not parse f64 from string."),
            );
        }

        let string_match = multi_line_mode(
            beginning()
                + text("# name: ")
                + named_capture(one_or_more(word()), "name")
                + or(&[text("\n# type: sq_string"), text("\n# type: string")])
                + text("\n# elements: ")
                + named_capture(one_or_more(digit()), "elements")
                + text("\n# length: ")
                + named_capture(one_or_more(digit()), "length")
                + text("\n")
                + named_capture(exactly(1, beginning() + one_or_more(any()) + end()), "data"),
        );

        for capture in string_match.to_regex().captures_iter(&*output) {
            let name = capture
                .name("name")
                .expect("Name not found")
                .as_str()
                .to_string();

            results.strings.insert(
                name,
                capture
                    .name("data")
                    .expect("No value for scalar data.")
                    .as_str()
                    .to_string(),
            );
        }

        let matrix_match = multi_line_mode(
            beginning()
                + text("# name: ")
                + named_capture(one_or_more(word()), "name")
                + or(&[text("\n# type: matrix"), text("\n# type: diagonal matrix")])
                + text("\n# rows: ")
                + named_capture(one_or_more(digit()), "rows")
                + text("\n# columns: ")
                + named_capture(one_or_more(digit()), "columns")
                + text("\n")
                + named_capture(zero_or_more(one_or_more(printable()) + text("\n")), "data"),
        );

        for capture in matrix_match.to_regex().captures_iter(&*output) {
            let name = capture
                .name("name")
                .expect("Name not found")
                .as_str()
                .to_string();
            let rows =
                usize::from_str(&*capture.name("rows").expect("No key named rows.").as_str())
                    .expect("Could not parse usize from string.");
            let columns = usize::from_str(
                &*capture
                    .name("columns")
                    .expect("No key named columns.")
                    .as_str(),
            )
            .expect("Could not parse usize from string.");

            let mut matrix = vec![vec![0.0_f64; columns]; rows];
            matrix = match capture.name("data") {
                None => matrix,
                Some(s) => {
                    if capture.get(2).unwrap().as_str().contains("diagonal") {
                        let data = s
                            .as_str()
                            .replace("\n", " ")
                            .split(" ")
                            .map(|elem| match f64::from_str(elem) {
                                Ok(val) => val,
                                Err(_) => f64::NAN,
                            })
                            .collect::<Vec<f64>>();
                        let mut counter: usize = 0;
                        for i in 0..rows {
                            matrix[i][i] = data[counter];
                            counter += 1;
                        }
                    } else {
                        let data = s
                            .as_str()
                            .replacen(" ", "", 1)
                            .replace("\n", "")
                            .split(" ")
                            .map(|elem| match f64::from_str(elem) {
                                Ok(val) => val,
                                Err(_) => f64::NAN,
                            })
                            .collect::<Vec<f64>>();
                        let mut counter: usize = 0;
                        for i in 0..rows {
                            for j in 0..columns {
                                matrix[i][j] = data[counter];
                                counter += 1;
                            }
                        }
                    }
                    matrix
                }
            };

            results.matrices.insert(name, matrix);
        }

        // # name: g
        // # type: cell
        // # rows: 1
        // # columns: 2
        // # name: <cell-element>
        // # type: sq_string
        // # elements: 1
        // # length: 1
        // a
        //
        //
        //
        // # name: <cell-element>
        // # type: sq_string
        // # elements: 1
        // # length: 1
        // b
        //
        //

        // let cell_match = multi_line_mode(
        //     beginning()
        //         + text("# name: ")
        //         + named_capture(one_or_more(word()), "name")
        //         + text("\n# type: cell")
        //         + text("\n# rows: ")
        //         + named_capture(one_or_more(digit()), "rows")
        //         + text("\n# columns: ")
        //         + named_capture(one_or_more(digit()), "columns"),
        // );
        //
        // let cell_element_match = multi_line_mode(
        //     beginning()
        //         + text("# name: <cell-element>\n")
        //         + text("# type: ")
        //         + named_capture(one_or_more(word()), "type")
        //         + zero_or_more(text("\n# ") + one_or_more(any()))
        //         + zero_or_one(named_capture(
        //             one_or_more(whitespace() + beginning() + one_or_more(any())),
        //             "data",
        //         )),
        // );
        //
        // for capture in cell_match.to_regex().captures_iter(&*output) {
        //     println!("CELL: {capture:?}");
        // }
        //
        // for capture in cell_element_match.to_regex().captures_iter(&*output) {
        //     println!("CELL-ELEMENT: {capture:?}");
        // }
        results
    }
}

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
pub fn eval(input: &str) -> OctaveResults {
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
    pub fn eval(&self, input: &str) -> OctaveResults {
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

                OctaveResults::from(output_text.join(""))
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
