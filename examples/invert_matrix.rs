use mocktave;

fn main() {
    mocktave::eval(
        "\
        z = 5.24; \
        m = inv(eye(5, 5)); \
        save(\"-\", \"*\");\
        ",
    );
}
