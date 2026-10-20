//! DEFLATE compression (RFC 1951), written from the RFC: LZ77 matching with
//! hash chains and one step of lazy matching, then per block the smallest
//! of stored, fixed-Huffman and dynamic-Huffman encodings (std.compress.1).
//! The output is deterministic: the same input and level give the same bytes.

use super::inflate::{CL_ORDER, DIST_BASE_T, DIST_EXTRA_T, LEN_BASE_T, LEN_EXTRA_T};

const WINDOW: usize = 32768;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
const HASH_BITS: u32 = 15;
/// Symbols per block before it is flushed.
const BLOCK_TOKENS: usize = 1 << 15;

/// A token: a literal (`len == 0`, `dist` is the byte) or a match.
#[derive(Clone, Copy)]
struct Tok {
    len: u16,
    dist: u16,
}

struct BitWriter {
    out: Vec<u8>,
    buf: u64,
    n: u32,
}

impl BitWriter {
    fn put(&mut self, v: u32, k: u32) {
        self.buf |= (v as u64) << self.n;
        self.n += k;
        while self.n >= 8 {
            self.out.push(self.buf as u8);
            self.buf >>= 8;
            self.n -= 8;
        }
    }
    fn align(&mut self) {
        if self.n > 0 {
            self.out.push(self.buf as u8);
            self.buf = 0;
            self.n = 0;
        }
    }
}

fn len_sym(len: usize) -> usize {
    LEN_BASE_T.iter().rposition(|&b| b as usize <= len).unwrap()
}

fn dist_sym(dist: usize) -> usize {
    DIST_BASE_T.iter().rposition(|&b| b as usize <= dist).unwrap()
}

/// Huffman code lengths for `freq`, no longer than `max` bits, forming a
/// complete code (so every decoder accepts it). A single used symbol gets
/// length 1.
fn code_lengths(freq: &[u32], max: u8) -> Vec<u8> {
    let used: Vec<usize> = (0..freq.len()).filter(|&i| freq[i] > 0).collect();
    let mut lens = vec![0u8; freq.len()];
    match used.len() {
        0 => return lens,
        1 => {
            lens[used[0]] = 1;
            return lens;
        }
        _ => {}
    }
    // Two-queue Huffman construction over the sorted leaves.
    let mut leaves: Vec<(u64, usize)> = used.iter().map(|&i| (freq[i] as u64, i)).collect();
    leaves.sort();
    let n = leaves.len();
    let mut weight: Vec<u64> = leaves.iter().map(|l| l.0).collect();
    let mut parent = vec![usize::MAX; 2 * n - 1];
    let (mut leaf, mut node) = (0, n);
    for k in n..2 * n - 1 {
        let mut pick = || {
            if leaf < n && (node >= k || weight[leaf] <= weight[node]) {
                leaf += 1;
                leaf - 1
            } else {
                node += 1;
                node - 1
            }
        };
        let a = pick();
        let b = pick();
        weight.push(weight[a] + weight[b]);
        parent[a] = k;
        parent[b] = k;
    }
    let mut depth = vec![0u8; 2 * n - 1];
    for i in (0..2 * n - 2).rev() {
        depth[i] = depth[parent[i]].saturating_add(1);
    }
    for (rank, &(_, sym)) in leaves.iter().enumerate() {
        lens[sym] = depth[rank].min(max);
    }
    // Restore the Kraft equality after clamping.
    let full: u64 = 1 << max;
    let mut kraft: u64 = used.iter().map(|&s| 1u64 << (max - lens[s])).sum();
    let mut by_len: Vec<usize> = used.clone();
    by_len.sort_by_key(|&s| (std::cmp::Reverse(lens[s]), freq[s]));
    while kraft > full {
        // Lengthen the rarest symbol of the longest length below max.
        let s = *by_len
            .iter()
            .filter(|&&s| lens[s] < max)
            .max_by_key(|&&s| (lens[s], std::cmp::Reverse(freq[s])))
            .expect("a code of at most 2^max symbols fits");
        kraft -= 1u64 << (max - lens[s] - 1);
        lens[s] += 1;
    }
    while kraft < full {
        let s = *by_len
            .iter()
            .filter(|&&s| lens[s] > 1 && kraft + (1u64 << (max - lens[s])) <= full)
            .max_by_key(|&&s| (lens[s], freq[s]))
            .expect("a gap is a multiple of the longest code's step");
        kraft += 1u64 << (max - lens[s]);
        lens[s] -= 1;
    }
    lens
}

/// Canonical codes (bit-reversed for LSB-first output) for `lens`.
fn codes(lens: &[u8]) -> Vec<u32> {
    let mut count = [0u32; 16];
    for &l in lens {
        count[l as usize] += 1;
    }
    count[0] = 0;
    let mut next = [0u32; 16];
    let mut c = 0;
    for l in 1..16 {
        c = (c + count[l - 1]) << 1;
        next[l] = c;
    }
    lens.iter()
        .map(|&l| {
            if l == 0 {
                return 0;
            }
            let code = next[l as usize];
            next[l as usize] += 1;
            code.reverse_bits() >> (32 - l as u32)
        })
        .collect()
}

fn fixed_lengths() -> (Vec<u8>, Vec<u8>) {
    let mut l = vec![8u8; 288];
    l[144..256].fill(9);
    l[256..280].fill(7);
    (l, vec![5u8; 30])
}

/// The run-length coding of code lengths (symbols 16-18), as
/// `(symbol, extra bits value)`.
fn rle(lens: &[u8]) -> Vec<(u8, u8)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lens.len() {
        let v = lens[i];
        let mut run = 1;
        while i + run < lens.len() && lens[i + run] == v {
            run += 1;
        }
        i += run;
        if v == 0 {
            while run >= 11 {
                let r = run.min(138);
                out.push((18, (r - 11) as u8));
                run -= r;
            }
            if run >= 3 {
                out.push((17, (run - 3) as u8));
                run = 0;
            }
        } else {
            out.push((v, 0));
            run -= 1;
            while run >= 3 {
                let r = run.min(6);
                out.push((16, (r - 3) as u8));
                run -= r;
            }
        }
        for _ in 0..run {
            out.push((v, 0));
        }
    }
    out
}

fn emit_tokens(w: &mut BitWriter, toks: &[Tok], ll: &[u8], lc: &[u32], dl: &[u8], dc: &[u32]) {
    for t in toks {
        if t.len == 0 {
            w.put(lc[t.dist as usize], ll[t.dist as usize] as u32);
        } else {
            let s = len_sym(t.len as usize);
            w.put(lc[257 + s], ll[257 + s] as u32);
            w.put(t.len as u32 - LEN_BASE_T[s] as u32, LEN_EXTRA_T[s] as u32);
            let d = dist_sym(t.dist as usize);
            w.put(dc[d], dl[d] as u32);
            w.put(t.dist as u32 - DIST_BASE_T[d] as u32, DIST_EXTRA_T[d] as u32);
        }
    }
    w.put(lc[256], ll[256] as u32);
}

fn token_bits(toks: &[Tok], ll: &[u8], dl: &[u8]) -> u64 {
    let mut bits = ll[256] as u64;
    for t in toks {
        if t.len == 0 {
            bits += ll[t.dist as usize] as u64;
        } else {
            let s = len_sym(t.len as usize);
            let d = dist_sym(t.dist as usize);
            bits += (ll[257 + s] + LEN_EXTRA_T[s] + dl[d] + DIST_EXTRA_T[d]) as u64;
        }
    }
    bits
}

/// Write one block holding `toks`, which cover `raw` (the bytes they expand to).
fn write_block(w: &mut BitWriter, toks: &[Tok], raw: &[u8], last: bool) {
    let mut lf = vec![0u32; 286];
    let mut df = vec![0u32; 30];
    lf[256] = 1;
    for t in toks {
        if t.len == 0 {
            lf[t.dist as usize] += 1;
        } else {
            lf[257 + len_sym(t.len as usize)] += 1;
            df[dist_sym(t.dist as usize)] += 1;
        }
    }
    let (fl, fd) = fixed_lengths();
    let fixed_bits = 3 + token_bits(toks, &fl, &fd);
    let ll = code_lengths(&lf, 15);
    let mut dl = code_lengths(&df, 15);
    if dl.iter().all(|&l| l == 0) {
        dl[0] = 1;
    }
    let nlen = (257..=286).rev().find(|&n| n == 257 || ll[n - 1] != 0).unwrap();
    let ndist = (1..=30).rev().find(|&n| n == 1 || dl[n - 1] != 0).unwrap();
    let mut all = ll[..nlen].to_vec();
    all.extend_from_slice(&dl[..ndist]);
    let syms = rle(&all);
    let mut cf = [0u32; 19];
    for &(s, _) in &syms {
        cf[s as usize] += 1;
    }
    let cl = code_lengths(&cf, 7);
    let ncode = (4..=19).rev().find(|&n| n == 4 || cl[CL_ORDER[n - 1]] != 0).unwrap();
    let mut dyn_bits = 3 + 14 + 3 * ncode as u64 + token_bits(toks, &ll, &dl);
    for &(s, _) in &syms {
        dyn_bits += cl[s as usize] as u64 + [0, 0, 2, 3, 7][if s >= 16 { s as usize - 14 } else { 0 }];
    }
    let stored_bits = raw.len().div_ceil(65535).max(1) as u64 * 40 + raw.len() as u64 * 8 + 7;
    if stored_bits <= fixed_bits.min(dyn_bits) {
        let mut chunks: Vec<&[u8]> = raw.chunks(65535).collect();
        if chunks.is_empty() {
            chunks.push(&[]);
        }
        let n = chunks.len();
        for (i, c) in chunks.into_iter().enumerate() {
            w.put((last && i + 1 == n) as u32, 1);
            w.put(0, 2);
            w.align();
            w.put(c.len() as u32, 16);
            w.put(!(c.len() as u32) & 0xffff, 16);
            w.out.extend_from_slice(c);
        }
    } else if fixed_bits <= dyn_bits {
        w.put(last as u32, 1);
        w.put(1, 2);
        emit_tokens(w, toks, &fl, &codes(&fl), &fd, &codes(&fd));
    } else {
        w.put(last as u32, 1);
        w.put(2, 2);
        w.put(nlen as u32 - 257, 5);
        w.put(ndist as u32 - 1, 5);
        w.put(ncode as u32 - 4, 4);
        for &i in CL_ORDER.iter().take(ncode) {
            w.put(cl[i] as u32, 3);
        }
        let cc = codes(&cl);
        for &(s, extra) in &syms {
            w.put(cc[s as usize], cl[s as usize] as u32);
            match s {
                16 => w.put(extra as u32, 2),
                17 => w.put(extra as u32, 3),
                18 => w.put(extra as u32, 7),
                _ => {}
            }
        }
        emit_tokens(w, toks, &ll, &codes(&ll), &dl, &codes(&dl));
    }
}

/// How hard the matcher looks: `(chain length, good enough length, lazy)`.
fn effort(level: u32) -> (usize, usize, bool) {
    match level {
        0..=1 => (4, 16, false),
        2..=3 => (16, 32, false),
        4..=5 => (32, 64, true),
        6 => (128, 128, true),
        7..=8 => (512, 258, true),
        _ => (4096, 258, true),
    }
}

/// Compress `data` into a raw DEFLATE stream at `level` 0 to 9 (0 stores).
pub fn deflate(data: &[u8], level: u32) -> Vec<u8> {
    let mut w = BitWriter { out: Vec::with_capacity(data.len() / 2 + 16), buf: 0, n: 0 };
    if data.is_empty() {
        write_block(&mut w, &[], data, true);
        w.align();
        return w.out;
    }
    if level == 0 {
        let chunks: Vec<&[u8]> = data.chunks(65535).collect();
        let n = chunks.len();
        for (i, c) in chunks.into_iter().enumerate() {
            w.put((i + 1 == n) as u32, 1);
            w.put(0, 2);
            w.align();
            w.put(c.len() as u32, 16);
            w.put(!(c.len() as u32) & 0xffff, 16);
            w.out.extend_from_slice(c);
        }
        return w.out;
    }
    let (max_chain, good, lazy) = effort(level);
    let n = data.len();
    let mut head = vec![u32::MAX; 1 << HASH_BITS];
    let mut prev = vec![u32::MAX; n];
    let hash = |i: usize| -> usize {
        let v = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], 0]);
        (v.wrapping_mul(0x9E37_79B1) >> (32 - HASH_BITS)) as usize
    };
    let insert = |head: &mut Vec<u32>, prev: &mut Vec<u32>, i: usize| {
        if i + MIN_MATCH <= n {
            let h = hash(i);
            prev[i] = head[h];
            head[h] = i as u32;
        }
    };
    let find = |head: &Vec<u32>, prev: &Vec<u32>, i: usize| -> (usize, usize) {
        if i + MIN_MATCH > n {
            return (0, 0);
        }
        let max_len = MAX_MATCH.min(n - i);
        let (mut best_len, mut best_dist) = (MIN_MATCH - 1, 0);
        let mut cand = head[hash(i)];
        let mut chain = max_chain;
        while cand != u32::MAX && chain > 0 {
            let c = cand as usize;
            let dist = i - c;
            if dist > WINDOW {
                break;
            }
            if data[c + best_len.min(max_len - 1)] == data[i + best_len.min(max_len - 1)] {
                let mut l = 0;
                while l < max_len && data[c + l] == data[i + l] {
                    l += 1;
                }
                if l > best_len {
                    best_len = l;
                    best_dist = dist;
                    if l >= good || l == max_len {
                        break;
                    }
                }
            }
            cand = prev[c];
            chain -= 1;
        }
        if best_len >= MIN_MATCH { (best_len, best_dist) } else { (0, 0) }
    };

    let mut toks: Vec<Tok> = Vec::with_capacity(BLOCK_TOKENS);
    let mut block_start = 0;
    let mut i = 0;
    while i < n {
        let (len, dist) = find(&head, &prev, i);
        if len >= MIN_MATCH && lazy && len < good && i + 1 < n {
            insert(&mut head, &mut prev, i);
            let (l2, _) = find(&head, &prev, i + 1);
            if l2 > len {
                // Emit a literal now; the better match is found next round.
                toks.push(Tok { len: 0, dist: data[i] as u16 });
                i += 1;
                if toks.len() >= BLOCK_TOKENS {
                    write_block(&mut w, &toks, &data[block_start..i], false);
                    toks.clear();
                    block_start = i;
                }
                continue;
            }
            // `i` is already inserted; the loop below skips it.
            toks.push(Tok { len: len as u16, dist: dist as u16 });
            for k in i + 1..i + len {
                insert(&mut head, &mut prev, k);
            }
            i += len;
        } else if len >= MIN_MATCH {
            toks.push(Tok { len: len as u16, dist: dist as u16 });
            for k in i..i + len {
                insert(&mut head, &mut prev, k);
            }
            i += len;
        } else {
            insert(&mut head, &mut prev, i);
            toks.push(Tok { len: 0, dist: data[i] as u16 });
            i += 1;
        }
        if toks.len() >= BLOCK_TOKENS {
            write_block(&mut w, &toks, &data[block_start..i], false);
            toks.clear();
            block_start = i;
        }
    }
    write_block(&mut w, &toks, &data[block_start..n], true);
    w.align();
    w.out
}
