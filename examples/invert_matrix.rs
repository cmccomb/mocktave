use mocktave;

fn main() {
    mocktave::eval(
        "\
        z = 5.24; \
        m = z*inv(eye(5, 5));\
        m(1, 2) = 5; \
        ",
    );
}
