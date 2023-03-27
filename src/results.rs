use std::{collections::HashMap, str::FromStr};

use regex::Captures;

use human_regex::{
    any, beginning, digit, end, exactly, multi_line_mode, named_capture, one_or_more, or,
    printable, text, word, zero_or_more,
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
    cell_arrays: HashMap<String, OctaveTypes>,
}

#[derive(Debug, Clone)]
pub enum OctaveTypes {
    Scalar(f64),
    Matrix(Vec<Vec<f64>>),
    String(String),
    CellArray(Vec<Vec<OctaveTypes>>),
}

impl InterpreterResults {
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
    pub fn get_cell_array_named(&self, name: &str) -> Option<OctaveTypes> {
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

        for capture in cell_array_match.to_regex().captures_iter(&output) {
            let (name, value) = parse_cell_array_capture(capture);
            results.cell_arrays.insert(name, value);
        }

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

fn parse_cell_array_capture(capture: Captures) -> (String, OctaveTypes) {
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

    (
        name,
        OctaveTypes::CellArray(vec![
            vec![OctaveTypes::String("".to_string()); rows];
            columns
        ]),
    )
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
