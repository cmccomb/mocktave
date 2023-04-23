fn main() {
    let script = "
        a = 25.2;
        b = rand(5, 5);
        c = 'asdf1';
        d = \"asdf2\";
        e = eye(5);
        f = [1, 2, 3, 4];
        g = {'a', 1, [1; 1]};
    ";

    let results = mocktave::eval(&script);

    let _a: f64 = results.get_scalar_named("a").unwrap();
    let _b: Vec<Vec<f64>> = results.get_matrix_named("b").unwrap();
    let _c: String = results.get_string_named("c").unwrap();
    let _d: String = results.get_string_named("d").unwrap();
    let _e: Vec<Vec<f64>> = results.get_matrix_named("e").unwrap();
    let _f: Vec<Vec<f64>> = results.get_matrix_named("f").unwrap();
    let _g: mocktave::OctaveType = results.get_cell_array_named("g").unwrap();

    println!("{results:#?}");
}
