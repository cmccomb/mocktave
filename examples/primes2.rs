fn main() {
    let primes: Box<dyn Fn([i32; 1]) -> Vec<Vec<i32>>> = mocktave::wrap("primes".into());
    let all_primes_less_than_100: Vec<Vec<i32>> = primes([100]);
    assert_eq!(
        all_primes_less_than_100,
        vec![vec![
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97
        ]]
    );
}
