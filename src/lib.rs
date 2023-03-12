#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::missing_docs_in_private_items)]
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
}

impl From<String> for OctaveResults {
    fn from(output: String) -> Self {
        let mut results = OctaveResults {
            scalars: Default::default(),
            matrices: Default::default(),
        };

        let split_output = output.split("\n");
        let mut name: String = "".to_owned();
        let mut currently_building: String = "".to_owned();
        let mut matrix: Vec<Vec<f64>> = vec![];
        let mut current_row: usize = 0;
        let mut max_rows: usize = 0;
        let mut columns: usize = 0;
        for line in split_output {
            if currently_building.len() == 0 {
                if line.starts_with("# Created") {
                    continue;
                } else if line.starts_with("# name: ") {
                    name = line.to_string().replace("# name: ", "").replace("\n", "")
                } else if line.starts_with("# type: ") {
                    currently_building = line.to_string().replace("# type: ", "").replace("\n", "")
                }
            } else {
                if currently_building == "scalar" && !line.is_empty() {
                    results
                        .scalars
                        .insert(name.clone(), f64::from_str(line).unwrap());
                    currently_building = "".to_owned();
                } else if currently_building == "matrix" && !line.is_empty() {
                    if line.starts_with("# rows: ") {
                        let current_row = 0;
                        max_rows =
                            usize::from_str(&*line.to_string().replace("# rows: ", "")).unwrap();
                    } else if line.starts_with("# columns: ") {
                        columns =
                            usize::from_str(&*line.to_string().replace("# columns: ", "")).unwrap();
                    } else {
                        if current_row == max_rows {
                            results.matrices.insert(name.clone(), matrix.clone());
                            currently_building = "".to_owned();
                        } else if !line.is_empty() {
                            let mut this_row = vec![];
                            for elem in line.split(" ") {
                                if elem.is_empty() {
                                    continue;
                                } else {
                                    this_row.push(f64::from_str(elem).unwrap());
                                }
                            }
                            matrix.push(this_row);
                            current_row += 1;
                        }
                    }
                }
            }
        }

        results
    }
}

/// Evaluate lines of Octave code
/// ```
/// use mocktave::eval;
/// let should_be_seven = eval("a = 5+2");
/// assert_eq!(*(should_be_seven.get_scalar_named("a").unwrap()), 7_f64);
/// ```
#[tokio::main]
pub async fn eval(input: &str) -> OctaveResults {
    const IMAGE: &str = "mtmiller/octave:latest";
    let docker = Docker::connect_with_socket_defaults().unwrap();

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
            // print!("{}", msg);
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
