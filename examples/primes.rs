fn main() {
    // A function to compute all primes less than a given number
    fn primes(less_than_n: usize) -> Vec<Vec<f64>> {
        mocktave::eval(&format!("x = primes({});", less_than_n))
            .get_matrix_named("x")
            .unwrap()
    }

    let all_primes_less_than_100 = primes(100);

    println!("{:#?}", all_primes_less_than_100);
}
