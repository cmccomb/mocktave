fn main() {
    let script = "
        a = 25.2;
        b = rand(5, 5);
        c = 'asdf1';
        d = \"asdf2\";
        e = eye(5);
        f = [1, 2, 3, 4];
        g = {'a', 1, [1; 1]};
        h = 1+1i;
    ";

    let results = mocktave::eval(&script);

    let _a: f64 = results.get_scalar("a").unwrap();
    let _b: Vec<Vec<f64>> = results.get_matrix("b").unwrap();
    let _c: String = results.get_string("c").unwrap();
    let _d: String = results.get_string("d").unwrap();
    let _e: Vec<Vec<f64>> = results.get_matrix("e").unwrap();
    let _f: Vec<Vec<f64>> = results.get_matrix("f").unwrap();
    let _g: Vec<Vec<mocktave::OctaveType>> = results.get_cell_array("g").unwrap();

    let _a2: f64 = results.get_unchecked("a").into();
    let _b2: Vec<Vec<f64>> = results.get_unchecked("b").into();
    let _c2: String = results.get_unchecked("c").into();
    let _d2: String = results.get_unchecked("d").into();
    let _e2: Vec<Vec<f32>> = results.get_unchecked("e").into();
    let _f2: Vec<Vec<f32>> = results.get_unchecked("f").into();
    let _g2: Vec<Vec<mocktave::OctaveType>> = results.get_unchecked("g").into();

    let _a3: &mocktave::OctaveType = &results["a"];
    let _c3: &mocktave::OctaveType = &results["b"];
    let _d3: &mocktave::OctaveType = &results["c"];
    let _e3: &mocktave::OctaveType = &results["d"];
    let _e3: &mocktave::OctaveType = &results["e"];
    let _f3: &mocktave::OctaveType = &results["f"];
    let _g3: &mocktave::OctaveType = &results["g"];

    println!("{results}");
}
