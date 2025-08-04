pub fn is_prime(n: f64) -> bool {
    if n.fract() != 0.0 {
        return false;
    }

    let n = n as i64;
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }

    let sqrt_n = (n as f64).sqrt() as i64;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }

    true
}