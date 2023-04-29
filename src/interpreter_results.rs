use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseFloatError;
use std::ops::{Index, IndexMut};
use std::{collections::HashMap, str::FromStr};

use regex::{Captures, Match};

use crate::OctaveType;

use crate::octave_types::OctaveTryIntoError;
use human_regex::{
    any, beginning, digit, end, exactly, multi_line_mode, named_capture, one_or_more, or,
    printable, text, whitespace, word, zero_or_more, zero_or_one,
};

/// Contains the workspace that resulted from running the octave command in `eval`
#[derive(Debug, Clone)]
pub struct InterpreterResults {
    /// Raw output
    pub raw: String,
    /// Variables
    variables: HashMap<String, OctaveType>,
}

impl InterpreterResults {
    /// Get a variable by name, if it exists
    pub fn get(&self, name: &str) -> Option<OctaveType> {
        self.variables.get(name).cloned()
    }
    /// Get a variable by name and convert it ot an f64, if the variable exists and is convertible.
    pub fn get_scalar(&self, name: &str) -> Option<f64> {
        self.variables
            .get(name)
            .cloned()
            .and_then(|ot| ot.try_into_f64().ok())
    }
    /// Get a variable by name and convert it ot a Vec<Vec<f64>>, if the variable exists and is
    /// convertible.
    pub fn get_matrix(&self, name: &str) -> Option<Vec<Vec<f64>>> {
        self.variables
            .get(name)
            .cloned()
            .and_then(|ot| ot.try_into_vec_f64().ok())
    }
    /// Get a variable by name and convert it ot a String, if the variable exists and is convertible.
    pub fn get_string(&self, name: &str) -> Option<String> {
        self.variables
            .get(name)
            .cloned()
            .and_then(|ot| ot.try_into_string().ok())
    }
    /// Get a variable by name and convert it ot a Vec<Vec<OctaveType>>, if the variable exists and
    /// is convertible.
    pub fn get_cell_array(&self, name: &str) -> Option<Vec<Vec<OctaveType>>> {
        self.variables
            .get(name)
            .cloned()
            .and_then(|ot| ot.try_into_vec_octave_type().ok())
    }
    /// Get a variable without checking whether or not it exists first. Panics if variable doesn't
    /// exist.
    pub fn get_unchecked(&self, name: &str) -> OctaveType {
        self.variables
            .get(name)
            .cloned()
            .expect(&format!("The variable `{name}` does not exist"))
    }
}
impl Default for InterpreterResults {
    fn default() -> Self {
        InterpreterResults {
            raw: "".to_string(),
            variables: Default::default(),
        }
    }
}
impl From<String> for InterpreterResults {
    fn from(output: String) -> Self {
        // Instantiate results and save raw output
        let mut results = InterpreterResults {
            raw: output.clone(),
            ..Default::default()
        };

        // Make a scalar match and parse the output
        let scalar_match = multi_line_mode(
            beginning()
                + text("# name: ")
                + named_capture(one_or_more(word()), "name")
                + text("\n# type: scalar\n")
                + named_capture(exactly(1, beginning() + one_or_more(any()) + end()), "data"),
        );

        for capture in scalar_match.to_regex().captures_iter(&output) {
            let (name, value) = parse_scalar_capture(capture);
            results.variables.insert(name, OctaveType::Scalar(value));
        }

        // Make a string capture and parse the output
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

        for capture in string_match.to_regex().captures_iter(&output) {
            let (name, value) = parse_string_capture(capture);
            results.variables.insert(name, OctaveType::String(value));
        }

        // Make a matrix match and parse the output
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

        for capture in matrix_match.to_regex().captures_iter(&output) {
            let (name, value) = parse_matrix_capture(capture);
            results.variables.insert(name, OctaveType::Matrix(value));
        }

        // Make a cell-array match and parse the output
        let cell_array_match = multi_line_mode(
            beginning()
                + text("# name: ")
                + named_capture(one_or_more(word()), "name")
                + text("\n# type: cell")
                + text("\n# rows: ")
                + named_capture(one_or_more(digit()), "rows")
                + text("\n# columns: ")
                + named_capture(one_or_more(digit()), "columns"),
        );

        let cell_element_match = multi_line_mode(
            beginning()
                + named_capture(text("# name: <cell-element>\n"), "name")
                + text("# type: ")
                + named_capture(one_or_more(word()), "type")
                + zero_or_more(text("\n# rows: ") + named_capture(one_or_more(digit()), "rows"))
                + zero_or_more(
                    text("\n# columns: ") + named_capture(one_or_more(digit()), "columns"),
                )
                + zero_or_more(
                    text("\n# elements: ") + named_capture(one_or_more(digit()), "elements"),
                )
                + zero_or_more(
                    text("\n# length: ") + named_capture(one_or_more(digit()), "length"),
                )
                + zero_or_one(named_capture(
                    one_or_more(whitespace() + beginning() + one_or_more(any())),
                    "data",
                )),
        );

        results.variables.extend(parse_cell_array_capture(
            cell_array_match.to_regex().captures_iter(&output),
            cell_element_match.to_regex().captures_iter(&output),
        ));

        results
    }
}

impl Index<&str> for InterpreterResults {
    type Output = OctaveType;

    fn index(&self, index: &str) -> &Self::Output {
        &self.variables.get(index).unwrap()
    }
}

impl IndexMut<&str> for InterpreterResults {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.variables.get_mut(index).unwrap()
    }
}

fn parse_scalar_capture(capture: Captures) -> (String, f64) {
    (
        capture
            .name("name")
            .expect("Name not found")
            .as_str()
            .to_string(),
        f64::from_str(
            &capture
                .name("data")
                .expect("No value for scalar data.")
                .as_str()
                .replace('\n', ""),
        )
        .expect("Could not parse f64 from string."),
    )
}

fn parse_string_capture(capture: Captures) -> (String, String) {
    let name = capture
        .name("name")
        .expect("Name not found")
        .as_str()
        .to_string();
    (
        name.clone(),
        capture
            .name("data")
            .expect(&format!("No value named {name} for string data."))
            .as_str()
            .to_string(),
    )
}

fn parse_cell_array_capture(
    array_captures: regex::CaptureMatches,
    mut element_captures: regex::CaptureMatches,
) -> HashMap<String, OctaveType> {
    let mut output: HashMap<String, OctaveType> = HashMap::new();

    for cell_array in array_captures {
        let name = cell_array
            .name("name")
            .expect("Name not found")
            .as_str()
            .to_string();
        let rows = cell_array
            .name("rows")
            .expect("Name not found")
            .as_str()
            .to_string()
            .parse()
            .unwrap();
        let columns = cell_array
            .name("columns")
            .expect("Name not found")
            .as_str()
            .to_string()
            .parse()
            .unwrap();

        let mut value = vec![vec![OctaveType::Empty; columns]; rows];

        for idx in 0..rows {
            for jdx in 0..columns {
                let cell_element = element_captures.next().unwrap();
                let cell_element_name = cell_element
                    .name("type")
                    .expect("Name not found")
                    .as_str()
                    .to_string();
                value[idx][jdx] = match cell_element_name.as_str() {
                    "scalar" => OctaveType::Scalar(parse_scalar_capture(cell_element).1),
                    "matrix" => OctaveType::Matrix(parse_matrix_capture(cell_element).1),
                    "sq_string" | "string" => OctaveType::String(
                        parse_string_capture(cell_element).1.replacen("\n", "", 1),
                    ),
                    _ => OctaveType::Empty,
                }
            }
        }

        output.insert(name, OctaveType::CellArray(value));
    }

    output
}

fn parse_matrix_capture(capture: Captures) -> (String, Vec<Vec<f64>>) {
    let name = capture
        .name("name")
        .expect("Name not found")
        .as_str()
        .to_string();
    let rows = usize::from_str(capture.name("rows").expect("No key named rows.").as_str())
        .expect("Could not parse usize from string.");
    let columns = usize::from_str(
        capture
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
                s.as_str()
                    .replacen('\n', " ", rows - 1)
                    .replace('\n', "")
                    .split(' ')
                    .map(|elem| match f64::from_str(elem) {
                        Ok(val) => val,
                        Err(_) => f64::NAN,
                    })
                    .enumerate()
                    .map(|(idx, element)| matrix[idx][idx] = element)
                    .for_each(drop);
            } else {
                let data = s
                    .as_str()
                    .replacen(' ', "", 1)
                    .replace('\n', "")
                    .split(' ')
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

    (name, matrix)
}
