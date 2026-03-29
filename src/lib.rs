pub fn perm(n: i64, k: i64) -> i64 {
    if k > n {
        return 0;
    }
    (n - k + 1..=n).product()
}

pub fn comb(n: i64, k: i64) -> i64 {
    if k > n {
        return 0;
    }
    perm(n, k) / perm(k, k)
}
