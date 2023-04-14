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

    let a: f64 = results.get_scalar_named("a").unwrap();
    let b: Vec<Vec<f64>> = results.get_matrix_named("b").unwrap();
    let c: String = results.get_string_named("c").unwrap();
    let d: String = results.get_string_named("d").unwrap();
    let e: Vec<Vec<f64>> = results.get_matrix_named("e").unwrap();
    let f: Vec<Vec<f64>> = results.get_matrix_named("f").unwrap();
    let g: mocktave::OctaveType = results.get_cell_array_named("g").unwrap();

    println!("{results:#?}");
}
