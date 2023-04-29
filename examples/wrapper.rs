use mocktave::{wrap, OctaveType};
fn main() {
    let norm = wrap("norm".into());
    let x = [
        OctaveType::Matrix(vec![vec![0.0; 2]; 2]),
        OctaveType::Scalar(2.0),
    ];
    let should_be_zero = norm(x).try_into_f64().unwrap();
    assert_eq!(should_be_zero, 0.0_f64);
}
