#![warn(clippy::all)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bollard::container::{Config, RemoveContainerOptions};
use bollard::Docker;
use std::collections::HashMap;
use std::str::FromStr;

use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;

/// Contains the workspace that resulted from running the octave command in `eval`
#[derive(Debug)]
pub struct OctaveResults {
    scalars: HashMap<String, f64>,
    matrices: HashMap<String, Vec<Vec<f64>>>,
    strings: HashMap<String, String>,
}

impl OctaveResults {
    /// Get a scalar by name
    pub fn get_scalar_named(&self, name: &str) -> Option<&f64> {
        self.scalars.get(name)
    }
    /// Get a matrix by name
    pub fn get_matrix_named(&self, name: &str) -> Option<&Vec<Vec<f64>>> {
        self.matrices.get(name)
    }
    /// Get a string by name
    pub fn get_string_named(&self, name: &str) -> Option<&String> {
        self.strings.get(name)
    }
}

impl From<String> for OctaveResults {
    fn from(output: String) -> Self {
        let mut results = OctaveResults {
            scalars: Default::default(),
            matrices: Default::default(),
            strings: Default::default(),
        };

        let split_output = output.split("\n");
        let mut name: String = "".to_owned();
        let mut curently_reading: String = "".to_owned();
        let mut matrix: Vec<Vec<f64>> = vec![];
        let mut current_row: usize = 0;
        let mut max_rows: usize = 0;
        let mut columns: usize = 0;
        for line in split_output {
            if curently_reading.len() == 0 {
                if line.starts_with("# Created") {
                    continue;
                } else if line.starts_with("# name: ") {
                    name = line.to_string().replace("# name: ", "").replace("\n", "");
                } else if line.starts_with("# type: ") {
                    curently_reading = line
                        .to_string()
                        .replace("# type: ", "")
                        .replace("\n", "")
                        .replace("sq_", "")
                }
            } else {
                if curently_reading == "scalar" && !line.is_empty() {
                    results
                        .scalars
                        .insert(name.clone(), f64::from_str(line).unwrap());
                    curently_reading = "".to_owned();
                } else if curently_reading == "string" && !line.is_empty() {
                    if line.starts_with("# elements: ") || line.starts_with("# length: ") {
                        continue;
                    } else {
                        results.strings.insert(name.clone(), line.parse().unwrap());
                        curently_reading = "".to_owned();
                    }
                } else if curently_reading == "matrix" && !line.is_empty() {
                    if line.starts_with("# rows: ") {
                        current_row = 0;
                        max_rows =
                            usize::from_str(&*line.to_string().replace("# rows: ", "")).unwrap();
                    } else if line.starts_with("# columns: ") {
                        columns =
                            usize::from_str(&*line.to_string().replace("# columns: ", "")).unwrap();
                    } else {
                        if !line.is_empty() {
                            let mut this_row = vec![];
                            println!("{line}");
                            for elem in line.split(" ") {
                                if elem.is_empty() {
                                    continue;
                                } else {
                                    println!("{elem}");
                                    this_row.push(f64::from_str(elem).unwrap());
                                }
                            }
                            matrix.push(this_row);
                            current_row += 1;
                        }
                        if current_row == max_rows {
                            results.matrices.insert(name.clone(), matrix.clone());
                            matrix = vec![];
                            curently_reading = "".to_owned();
                        }
                    }
                } else if curently_reading == "diagonal matrix" && !line.is_empty() {
                    if line.starts_with("# rows: ") {
                        current_row = 0;
                        max_rows =
                            usize::from_str(&*line.to_string().replace("# rows: ", "")).unwrap();
                    } else if line.starts_with("# columns: ") {
                        columns =
                            usize::from_str(&*line.to_string().replace("# columns: ", "")).unwrap();
                    } else {
                        if !line.is_empty() {
                            let mut this_row = vec![0.0_f64; columns];
                            this_row[current_row] = f64::from_str(line).unwrap();
                            matrix.push(this_row);
                            current_row += 1;
                        }
                        if current_row == max_rows {
                            results.matrices.insert(name.clone(), matrix.clone());
                            matrix = vec![];
                            curently_reading = "".to_owned();
                        }
                    }
                }
            }
        }

        results
    }
}

/// Evaluate lines of Octave code and extract the results.
/// ```
/// let res = mocktave::eval("a = 5+2");
/// assert_eq!(res.get_scalar_named("a").unwrap(), &7_f64);
/// ```
/// ```
/// let res = mocktave::eval("a = ones(2, 2)");
/// assert_eq!(res.get_matrix_named("a").unwrap(), &vec![vec![1.0_f64; 2]; 2]);
/// ```
/// ```
/// let res = mocktave::eval("a = 'asdf'");
/// assert_eq!(res.get_string_named("a").unwrap(), "asdf");
/// ```
#[tokio::main]
pub async fn eval(input: &str) -> OctaveResults {
    const IMAGE: &str = "mtmiller/octave:7.0.0";
    let docker =
        Docker::connect_with_socket_defaults().expect("Could not connect with socket defaults");

    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await
        .expect("Could not create image.");

    let alpine_config = Config {
        image: Some(IMAGE),
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
        .expect("Could not start container.");

    // non interactive
    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(vec![
                    "octave",
                    "--eval",
                    &(input.to_string() + ";save(\"-\", \"*\");"),
                ]),
                ..Default::default()
            },
        )
        .await
        .expect("Could not create command to execute.")
        .id;

    let mut output_text = vec!["".to_string(); 0];

    if let StartExecResults::Attached { mut output, .. } = docker
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

    docker
        .remove_container(
            &id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .expect("Could not remove container.");

    OctaveResults::from(output_text.join(""))
}
