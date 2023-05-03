use mocktave::{wrap, OctaveType};
fn main() {
    let norm = wrap("norm".into());
    let x: [OctaveType; 2] = [vec![vec![0.0; 2]; 2].into(), 2.0_f64.into()];

    let should_be_zero: f64 = norm(x);
    assert_eq!(should_be_zero, 0.0_f64);
}
