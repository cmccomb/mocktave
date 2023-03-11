use mocktave;

fn main() {
    let script = "        \
        z = 5.24;               \
        m = z*inv(eye(5, 5));   \
        m(1, 2) = 5;            \
        a = 5;                  \
        ";

    let y = mocktave::eval(script);

    println!("{y:#?}");
}
