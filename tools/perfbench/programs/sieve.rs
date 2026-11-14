// Sieve of Eratosthenes: the same count, sum and largest as sieve.ail.
fn main() {
    let n: usize = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(1000000);
    let mut is_prime = vec![true; n];
    is_prime[0] = false;
    is_prime[1] = false;
    let mut i = 2;
    while i * i < n {
        if is_prime[i] {
            let mut j = i * i;
            while j < n {
                is_prime[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    let mut count = 0i64;
    let mut total = 0i64;
    let mut largest = 0i64;
    for k in 0..n {
        if is_prime[k] {
            count += 1;
            total += k as i64;
            largest = k as i64;
        }
    }
    println!("primes below {}: {}", n, count);
    println!("sum: {}", total);
    println!("largest: {}", largest);
}
