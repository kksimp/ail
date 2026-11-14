// Rust twin of lzr.ail: LZ77 + adaptive binary range coder.
struct Coder {
    low: i64,
    rng: i64,
    cache: i64,
    cache_size: i64,
    out: Vec<i64>,
    probs: Vec<i64>,
    code: i64,
    pos: usize,
}

impl Coder {
    fn shift_low(&mut self) {
        if self.low < 4278190080 || self.low >= 4294967296 {
            let carry = self.low >> 32;
            let mut temp = self.cache;
            loop {
                self.out.push((temp + carry) & 255);
                temp = 255;
                self.cache_size -= 1;
                if self.cache_size == 0 {
                    break;
                }
            }
            self.cache = (self.low >> 24) & 255;
        }
        self.cache_size += 1;
        self.low = (self.low & 16777215) << 8;
    }

    fn ebit(&mut self, idx: i64, b: i64) {
        let p = self.probs[idx as usize];
        let bound = (self.rng >> 11) * p;
        if b == 0 {
            self.rng = bound;
            self.probs[idx as usize] = p + ((2048 - p) >> 5);
        } else {
            self.low += bound;
            self.rng -= bound;
            self.probs[idx as usize] = p - (p >> 5);
        }
        while self.rng < 16777216 {
            self.rng <<= 8;
            self.shift_low();
        }
    }

    fn edirect(&mut self, v: i64, nbits: i64) {
        let mut i = nbits - 1;
        while i >= 0 {
            self.rng >>= 1;
            if (v >> i) & 1 != 0 {
                self.low += self.rng;
            }
            while self.rng < 16777216 {
                self.rng <<= 8;
                self.shift_low();
            }
            i -= 1;
        }
    }

    fn etree(&mut self, base: i64, nbits: i64, v: i64) {
        let mut m = 1;
        let mut i = nbits - 1;
        while i >= 0 {
            let b = (v >> i) & 1;
            self.ebit(base + m, b);
            m = m * 2 + b;
            i -= 1;
        }
    }

    fn dnext(&mut self) -> i64 {
        // Past the end reads 0 (the C twin reads whatever its buffer holds there; a correct stream never gets that far).
        let v = if self.pos < self.out.len() { self.out[self.pos] } else { 0 };
        self.pos += 1;
        v
    }

    fn dbit(&mut self, idx: i64) -> i64 {
        let p = self.probs[idx as usize];
        let bound = (self.rng >> 11) * p;
        let mut r = 0;
        if self.code < bound {
            self.rng = bound;
            self.probs[idx as usize] = p + ((2048 - p) >> 5);
        } else {
            self.code -= bound;
            self.rng -= bound;
            self.probs[idx as usize] = p - (p >> 5);
            r = 1;
        }
        if self.rng < 16777216 {
            self.rng <<= 8;
            self.code = (self.code << 8) | self.dnext();
        }
        r
    }

    fn ddirect(&mut self, nbits: i64) -> i64 {
        let mut v = 0;
        for _ in 0..nbits {
            self.rng >>= 1;
            let mut b = 0;
            if self.code >= self.rng {
                self.code -= self.rng;
                b = 1;
            }
            v = (v << 1) | b;
            if self.rng < 16777216 {
                self.rng <<= 8;
                self.code = (self.code << 8) | self.dnext();
            }
        }
        v
    }

    fn dtree(&mut self, base: i64, nbits: i64) -> i64 {
        let mut m = 1;
        let mut top = 1;
        for _ in 0..nbits {
            m = m * 2 + self.dbit(base + m);
            top *= 2;
        }
        m - top
    }
}

fn hash3(a: i64, b: i64, c: i64) -> usize {
    ((((a * 65536 + b * 256 + c) * 2654435761) & 4294967295) >> 12) as usize
}

struct Matcher {
    head: Vec<i64>,
    prev: Vec<i64>,
    data: Vec<i64>,
}

impl Matcher {
    fn insert_at(&mut self, p: usize, n: usize) {
        if p + 2 < n {
            let h = hash3(self.data[p], self.data[p + 1], self.data[p + 2]);
            self.prev[p] = self.head[h];
            self.head[h] = p as i64;
        }
    }

    fn find(&self, i: usize, n: usize) -> i64 {
        let mut best_len = 0i64;
        let mut best_dist = 0i64;
        if i + 2 < n {
            let mut cand = self.head[hash3(self.data[i], self.data[i + 1], self.data[i + 2])];
            let mut chain = 0;
            while cand >= 0 && i as i64 - cand <= 1048576 && chain < 32 {
                let c = cand as usize;
                let mut l = 0usize;
                while l < 66 && i + l < n && self.data[c + l] == self.data[i + l] {
                    l += 1;
                }
                if l as i64 > best_len {
                    best_len = l as i64;
                    best_dist = i as i64 - cand;
                }
                cand = self.prev[c];
                chain += 1;
            }
        }
        if best_len < 3 {
            return 0;
        }
        best_len * 2097152 + best_dist
    }
}

fn dist_slot(d: i64) -> i64 {
    if d < 4 {
        return d;
    }
    let mut n = 0;
    let mut t = d;
    while t > 1 {
        t >>= 1;
        n += 1;
    }
    2 * n + ((d >> (n - 1)) & 1)
}

fn main() {
    let mut n: usize = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(12000000);
    let mut data: Vec<i64> = Vec::with_capacity(n + 16);
    let mut seed: i64 = 7;
    while data.len() < n {
        seed = (seed * 3877 + 29573) % 139968;
        let k = seed % 32768;
        for j in 0..3 + k % 7 {
            data.push(97 + (k * 7 + j * 3 + k / 64) % 26);
        }
        data.push(32);
    }
    n = data.len();
    let mut mt = Matcher { head: vec![-1; 1048576], prev: vec![-1; n], data };
    let mut co = Coder {
        low: 0,
        rng: 4294967295,
        cache: 0,
        cache_size: 1,
        out: Vec::with_capacity(n + 1024),
        probs: vec![1024; 2400],
        code: 0,
        pos: 0,
    };
    let mut ins = 0usize;
    let mut st = 0i64;
    let mut i = 0usize;
    while i < n {
        while ins < i {
            mt.insert_at(ins, n);
            ins += 1;
        }
        let mut m = mt.find(i, n);
        if m > 0 && i + 1 < n {
            while ins < i + 1 {
                mt.insert_at(ins, n);
                ins += 1;
            }
            if mt.find(i + 1, n) / 2097152 > m / 2097152 + 1 {
                m = 0;
            }
        }
        if m > 0 {
            let l = m / 2097152 - 3;
            let d = m % 2097152 - 1;
            co.ebit(st, 1);
            co.etree(2, 6, l);
            let slot = dist_slot(d);
            co.etree(66 + l.min(3) * 64, 6, slot);
            if slot >= 4 {
                let nb = (slot >> 1) - 1;
                co.edirect(d - ((2 + (slot & 1)) << nb), nb);
            }
            st = 1;
            i += (l + 3) as usize;
        } else {
            let ctx = if i > 0 { mt.data[i - 1] >> 5 } else { 0 };
            co.ebit(st, 0);
            co.etree(322 + ctx * 256, 8, mt.data[i]);
            st = 0;
            i += 1;
        }
    }
    for _ in 0..5 {
        co.shift_low();
    }
    let clen = co.out.len();
    // decode
    co.pos = 1;
    co.code = 0;
    co.rng = 4294967295;
    for _ in 0..4 {
        co.code = (co.code << 8) | co.dnext();
    }
    for p in co.probs.iter_mut() {
        *p = 1024;
    }
    let mut back = vec![0i64; n + 80];
    let mut nb2 = 0usize;
    st = 0;
    while nb2 < n {
        if co.dbit(st) == 0 {
            let ctx = if nb2 > 0 { back[nb2 - 1] >> 5 } else { 0 };
            back[nb2] = co.dtree(322 + ctx * 256, 8);
            nb2 += 1;
            st = 0;
        } else {
            let l = co.dtree(2, 6);
            let slot = co.dtree(66 + l.min(3) * 64, 6);
            let mut dist = slot;
            if slot >= 4 {
                let nb = (slot >> 1) - 1;
                dist = ((2 + (slot & 1)) << nb) + co.ddirect(nb);
            }
            let start = nb2 - (dist as usize + 1);
            for k in 0..(l + 3) as usize {
                back[nb2] = back[start + k];
                nb2 += 1;
            }
            st = 1;
        }
    }
    let mut cks = 0i64;
    for k in 0..clen {
        cks = (cks * 31 + co.out[k]) % 1000000007;
    }
    let ok = nb2 == n && (0..n).all(|k| back[k] == mt.data[k]);
    println!(
        "input {}\ncompressed {}\npermille {}\nchecksum {}\nroundtrip {}",
        n,
        clen,
        clen as i64 * 1000 / n as i64,
        cks,
        if ok { "true" } else { "false" }
    );
}
