fn main() {
    let tolerance_min: usize = 0;
    let tolerance_max: usize = 0;
    let script = format!("
         ## Demonstrate convergence order for ode45
         tol = 1e-5 ./ 10.^[{tolerance_min}:{tolerance_max}];
         for i = 1 : numel (tol)
           opt = odeset (\"RelTol\", tol(i), \"AbsTol\", realmin);
           [t, y] = ode45 (@(t, y) -y, [0, 1], 1, opt);
           h(i) = 1 / (numel (t) - 1);
           err(i) = norm (y .* exp (t) - 1, Inf);
         endfor

         ## Estimate order numerically
         p = diff (log (err)) ./ diff (log (h))
    ");

    let y = mocktave::eval(&script);

    println!("{y:#?}");
}
