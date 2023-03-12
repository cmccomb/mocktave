use mocktave;

fn main() {
    let script = "m = inv(eye(5, 5))";

    let y = mocktave::eval(script);

    println!("{y:#?}");
}
