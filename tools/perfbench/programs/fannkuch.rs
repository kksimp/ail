// Fannkuch-redux: the same permutation walk as fannkuch.ail.
fn main() {
    let n: usize = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(8);
    let mut perm1: Vec<usize> = (0..n).collect();
    let mut perm = vec![0usize; n];
    let mut count = vec![0usize; n];
    let mut maxflips = 0i64;
    let mut checksum = 0i64;
    let mut permcount = 0i64;
    let mut r = n;
    let mut done = false;
    while !done {
        while r != 1 {
            count[r - 1] = r;
            r -= 1;
        }
        perm.copy_from_slice(&perm1);
        let mut flips = 0i64;
        let mut k = perm[0];
        while k != 0 {
            let mut i = 0;
            let mut j = k;
            while i < j {
                perm.swap(i, j);
                i += 1;
                j -= 1;
            }
            flips += 1;
            k = perm[0];
        }
        if flips > maxflips {
            maxflips = flips;
        }
        if permcount % 2 == 0 {
            checksum += flips;
        } else {
            checksum -= flips;
        }
        let mut advanced = false;
        while !advanced {
            if r == n {
                done = true;
                break;
            }
            let first = perm1[0];
            for i in 0..r {
                perm1[i] = perm1[i + 1];
            }
            perm1[r] = first;
            count[r] -= 1;
            if count[r] > 0 {
                advanced = true;
            } else {
                r += 1;
            }
        }
        permcount += 1;
    }
    println!("{}", checksum);
    println!("Pfannkuchen({}) = {}", n, maxflips);
}
