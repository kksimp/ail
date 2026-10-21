//! DEFLATE decompression (RFC 1951), written from the RFC. Canonical
//! Huffman codes decode through a 9-bit lookup table, with the RFC's
//! count-and-offset walk for longer codes (std.compress.1).

use super::CodecError;

const LEN_BASE: [u16; 29] =
    [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
const LEN_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145,
    8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] =
    [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
/// The order of the code-length code lengths in a dynamic block header.
pub(super) const CL_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
pub(super) const LEN_BASE_T: [u16; 29] = LEN_BASE;
pub(super) const LEN_EXTRA_T: [u8; 29] = LEN_EXTRA;
pub(super) const DIST_BASE_T: [u16; 30] = DIST_BASE;
pub(super) const DIST_EXTRA_T: [u8; 30] = DIST_EXTRA;

// std.compress.9: the closed list of corrupt messages.
fn bad(msg: &str) -> CodecError {
    CodecError::new("corrupt", msg)
}

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    buf: u64,
    n: u32,
}

impl<'a> Bits<'a> {
    fn refill(&mut self) {
        while self.n <= 56 && self.pos < self.data.len() {
            self.buf |= (self.data[self.pos] as u64) << self.n;
            self.pos += 1;
            self.n += 8;
        }
    }
    fn get(&mut self, k: u32) -> Result<u32, CodecError> {
        if k == 0 {
            return Ok(0);
        }
        if self.n < k {
            self.refill();
            if self.n < k {
                return Err(bad("unexpected end of compressed data"));
            }
        }
        let v = (self.buf & ((1u64 << k) - 1)) as u32;
        self.buf >>= k;
        self.n -= k;
        Ok(v)
    }
    /// Drop the bits left in the current byte.
    fn align(&mut self) {
        let drop = self.n % 8;
        self.buf >>= drop;
        self.n -= drop;
    }
    /// Bytes not yet consumed, including whole bytes still in the buffer.
    fn consumed(&self) -> usize {
        self.pos - (self.n / 8) as usize
    }
}

const FAST: u32 = 9;

struct Huff {
    count: [u16; 16],
    symbol: Vec<u16>,
    /// `len << 12 | symbol`, or 0 when the code is longer than FAST bits.
    fast: Vec<u16>,
}

impl Huff {
    fn new(lengths: &[u8]) -> Result<Huff, CodecError> {
        let mut count = [0u16; 16];
        for &l in lengths {
            count[l as usize] += 1;
        }
        count[0] = 0;
        let mut left: i32 = 1;
        for len in 1..16 {
            left = (left << 1) - count[len] as i32;
            if left < 0 {
                return Err(bad("over-subscribed Huffman code"));
            }
        }
        let mut offs = [0u16; 16];
        for len in 1..15 {
            offs[len + 1] = offs[len] + count[len];
        }
        let mut symbol = vec![0u16; lengths.len()];
        for (s, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbol[offs[l as usize] as usize] = s as u16;
                offs[l as usize] += 1;
            }
        }
        // Canonical codes, MSB first, for the fast table (stored bit-reversed).
        let mut fast = vec![0u16; 1 << FAST];
        let mut code = 0u32;
        let mut index = 0usize;
        for len in 1..=FAST {
            for _ in 0..count[len as usize] {
                let sym = symbol[index];
                index += 1;
                let rev = code.reverse_bits() >> (32 - len);
                let mut i = rev as usize;
                while i < fast.len() {
                    fast[i] = ((len as u16) << 12) | sym;
                    i += 1 << len;
                }
                code += 1;
            }
            code <<= 1;
        }
        Ok(Huff { count, symbol, fast })
    }

    fn decode(&self, b: &mut Bits) -> Result<usize, CodecError> {
        if b.n < 15 {
            b.refill();
        }
        let e = self.fast[(b.buf & ((1 << FAST) - 1)) as usize];
        if e != 0 && (e >> 12) as u32 <= b.n {
            let l = (e >> 12) as u32;
            b.buf >>= l;
            b.n -= l;
            return Ok((e & 0xfff) as usize);
        }
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            if b.n == 0 {
                return Err(bad("unexpected end of compressed data"));
            }
            code |= (b.buf & 1) as i32;
            b.buf >>= 1;
            b.n -= 1;
            let c = self.count[len] as i32;
            if code - c < first {
                return Ok(self.symbol[(index + (code - first)) as usize] as usize);
            }
            index += c;
            first = (first + c) << 1;
            code <<= 1;
        }
        Err(bad("invalid Huffman code"))
    }
}

fn fixed() -> (Huff, Huff) {
    let mut l = [8u8; 288];
    l[144..256].fill(9);
    l[256..280].fill(7);
    (Huff::new(&l).unwrap(), Huff::new(&[5u8; 30]).unwrap())
}

/// Inflate the DEFLATE stream at the start of `data`; return the output and
/// how many input bytes the stream used. `limit` caps the output size
/// (a zip bomb guard, std.compress.4).
pub fn inflate_prefix(data: &[u8], limit: usize) -> Result<(Vec<u8>, usize), CodecError> {
    let mut b = Bits { data, pos: 0, buf: 0, n: 0 };
    let mut out: Vec<u8> = Vec::with_capacity(data.len().saturating_mul(3).min(limit));
    let mut fixed_tables: Option<(Huff, Huff)> = None;
    loop {
        let last = b.get(1)?;
        match b.get(2)? {
            0 => {
                b.align();
                let len = b.get(16)?;
                let nlen = b.get(16)?;
                if len != !nlen & 0xffff {
                    return Err(bad("stored block length check failed"));
                }
                if out.len() + len as usize > limit {
                    return Err(CodecError::new("too_big", "output exceeds the size limit"));
                }
                for _ in 0..len {
                    out.push(b.get(8)? as u8);
                }
            }
            t @ (1 | 2) => {
                let owned;
                let (lit, dist) = if t == 1 {
                    &*fixed_tables.get_or_insert_with(fixed)
                } else {
                    owned = dynamic(&mut b)?;
                    &owned
                };
                loop {
                    let sym = lit.decode(&mut b)?;
                    if sym < 256 {
                        out.push(sym as u8);
                    } else if sym == 256 {
                        break;
                    } else {
                        let s = sym - 257;
                        if s >= 29 {
                            return Err(bad("invalid length symbol"));
                        }
                        let len = LEN_BASE[s] as usize + b.get(LEN_EXTRA[s] as u32)? as usize;
                        let d = dist.decode(&mut b)?;
                        if d >= 30 {
                            return Err(bad("invalid distance symbol"));
                        }
                        let dist = DIST_BASE[d] as usize + b.get(DIST_EXTRA[d] as u32)? as usize;
                        if dist > out.len() {
                            return Err(bad("distance reaches before the start of the output"));
                        }
                        let start = out.len() - dist;
                        if dist >= len {
                            out.extend_from_within(start..start + len);
                        } else {
                            for i in 0..len {
                                out.push(out[start + i]);
                            }
                        }
                    }
                    if out.len() > limit {
                        return Err(CodecError::new("too_big", "output exceeds the size limit"));
                    }
                }
            }
            _ => return Err(bad("invalid block type")),
        }
        if last == 1 {
            break;
        }
    }
    b.align();
    Ok((out, b.consumed()))
}

fn dynamic(b: &mut Bits) -> Result<(Huff, Huff), CodecError> {
    let nlen = b.get(5)? as usize + 257;
    let ndist = b.get(5)? as usize + 1;
    let ncode = b.get(4)? as usize + 4;
    if nlen > 286 || ndist > 30 {
        return Err(bad("too many length or distance codes"));
    }
    let mut cl = [0u8; 19];
    for &i in CL_ORDER.iter().take(ncode) {
        cl[i] = b.get(3)? as u8;
    }
    let clh = Huff::new(&cl)?;
    let mut lengths = [0u8; 316];
    let mut i = 0;
    while i < nlen + ndist {
        let sym = clh.decode(b)?;
        if sym < 16 {
            lengths[i] = sym as u8;
            i += 1;
        } else {
            let (val, rep) = match sym {
                16 => {
                    if i == 0 {
                        return Err(bad("repeat with no previous length"));
                    }
                    (lengths[i - 1], 3 + b.get(2)? as usize)
                }
                17 => (0, 3 + b.get(3)? as usize),
                _ => (0, 11 + b.get(7)? as usize),
            };
            if i + rep > nlen + ndist {
                return Err(bad("code length repeat overruns the table"));
            }
            lengths[i..i + rep].fill(val);
            i += rep;
        }
    }
    if lengths[256] == 0 {
        return Err(bad("block has no end-of-block code"));
    }
    Ok((Huff::new(&lengths[..nlen])?, Huff::new(&lengths[nlen..nlen + ndist])?))
}

/// Inflate a whole raw DEFLATE stream; trailing bytes are an error.
pub fn inflate(data: &[u8], limit: usize) -> Result<Vec<u8>, CodecError> {
    let (out, used) = inflate_prefix(data, limit)?;
    if used != data.len() {
        return Err(bad("extra bytes after the end of the compressed data"));
    }
    Ok(out)
}
