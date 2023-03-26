fn main() {
    let script = "
        a = 25.2;
        b = rand(5, 5);
        c = 'asdf1';
        d = \"asdf2\";
        e = eye(5);
        f = [1, 2, 3, 4];
        g = {'a', 'b'};
    ";

    let results = mocktave::eval(&script);

    let a = results.get_scalar_named("a").unwrap();
    let b = results.get_matrix_named("b").unwrap();
    let c = results.get_string_named("c").unwrap();
    let d = results.get_string_named("d").unwrap();
    let e = results.get_matrix_named("e").unwrap();
    let f = results.get_matrix_named("f").unwrap();
    // let g = results.get_matrix_named("g").unwrap();

    println!("{results:#?}");
}
