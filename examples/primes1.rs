fn main() {
    // Define a function to compute all primes less than a given number
    fn primes(less_than_n: usize) -> Vec<i32> {
        mocktave::eval(&format!("x = primes({});", less_than_n))
            .get_unchecked("x")
            .into()
    }

    // Calculate all primes less than 100
    let all_primes_less_than_100 = primes(100);

    // Check the outcome
    assert_eq!(
        all_primes_less_than_100,
        vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97
        ]
    );
}
