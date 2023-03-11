use mocktave;

fn main() {
    let y = mocktave::eval(
        "\
        z = 5.24;               \
        m = z*inv(eye(5, 5));   \
        m(1, 2) = 5;            \
        a = 5;                  \
        ",
    );

    println!("{y:#?}");
}
