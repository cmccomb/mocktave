use std::{collections::HashMap, str::FromStr};

use regex::{Captures, Match};

use crate::OctaveType::CellArray;
use human_regex::{
    any, beginning, digit, end, exactly, multi_line_mode, named_capture, one_or_more, or,
    printable, text, whitespace, word, zero_or_more, zero_or_one,
};

/// Contains the workspace that resulted from running the octave command in `eval`
#[derive(Debug, Clone)]
pub struct InterpreterResults {
    /// Raw output
    pub raw: String,
    /// Scalar variables
    scalars: HashMap<String, f64>,
    /// Matrix variables
    matrices: HashMap<String, Vec<Vec<f64>>>,
    /// String variables
    strings: HashMap<String, String>,
    /// String variables
    cell_arrays: HashMap<String, OctaveType>,
}

/// ```
/// use mocktave::OctaveType;
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum OctaveType {
    Scalar(f64),
    Matrix(Vec<Vec<f64>>),
    String(String),
    CellArray(Vec<Vec<OctaveType>>),
    Empty,
}

impl Default for OctaveType {
    fn default() -> Self {
        OctaveType::Empty
    }
}

impl InterpreterResults {
    /// Get unchecked
    pub fn get_unchecked(&self, name: &str) -> OctaveType {
        match self.get_scalar_named(name) {
            None => match self.get_matrix_named(name) {
                None => match self.get_string_named(name) {
                    None => match self.get_cell_array_named(name) {
                        None => OctaveType::Empty,
                        Some(cell_array) => cell_array,
                    },
                    Some(string) => OctaveType::String(string),
                },
                Some(matrix) => OctaveType::Matrix(matrix),
            },
            Some(scalar) => OctaveType::Scalar(scalar),
        }
    }
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
    /// Get a string by name
    pub fn get_cell_array_named(&self, name: &str) -> Option<OctaveType> {
        self.cell_arrays.get(name).cloned()
    }
}

impl Default for InterpreterResults {
    fn default() -> Self {
        InterpreterResults {
            raw: "".to_string(),
            scalars: Default::default(),
            matrices: Default::default(),
            strings: Default::default(),
            cell_arrays: Default::default(),
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
            results.scalars.insert(name, value);
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
            results.strings.insert(name, value);
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

        for capture in matrix_match.to_regex().captures_iter(&output) {
            let (name, value) = parse_matrix_capture(capture);
            results.matrices.insert(name, value);
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
                + text("\n# rows: ")
                + zero_or_more(named_capture(one_or_more(digit()), "rows"))
                + text("\n# columns: ")
                + zero_or_more(named_capture(one_or_more(digit()), "columns"))
                + text("\n# elements: ")
                + zero_or_more(named_capture(one_or_more(digit()), "elements"))
                + text("\n# length: ")
                + zero_or_more(named_capture(one_or_more(digit()), "length"))
                + zero_or_one(named_capture(
                    one_or_more(whitespace() + beginning() + one_or_more(any())),
                    "data",
                )),
        );

        for capture in cell_array_match.to_regex().captures_iter(&output) {
            let (name, value) = parse_cell_array_capture(
                capture,
                cell_element_match
                    .to_regex()
                    .captures_iter(&output)
                    .map(|x| x)
                    .collect::<Vec<Captures>>(),
            );
            results.cell_arrays.insert(name, value);
        }

        results
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
    capture: Captures,
    mut elements: Vec<Captures>,
) -> (String, OctaveType) {
    println!("{elements:?}");

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

    let mut cell_array = vec![vec![OctaveType::Empty; columns]; rows];

    //     for i in 0..rows {
    //         for j in 0..columns {
    //             cell_array[i][j] = match element.name("type").unwrap().as_str() {
    //                 "sq_string" | "string" => {
    //                     OctaveType::String(parse_string_capture(element).1.replacen("\n", "", 1))
    //                 }
    //                 "scalar" => OctaveType::Scalar(parse_scalar_capture(element).1),
    //                 "matrix" => OctaveType::Matrix(parse_matrix_capture(element).1),
    //                 &_ => OctaveType::Empty,
    //             };
    //         }
    //     }

    (name, CellArray(cell_array))
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
