//! The sized numeric types other than `int` (`i64`) and `float` (`f64`)
//! (type.num.1, type.num.13-15).
//!
//! A sized value is stored as an `i64` bit pattern: integers hold their value
//! (a `u64` above `i64::MAX` is its two's-complement reinterpretation), and an
//! `f32` holds its IEEE bits.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Num {
    I8,
    I16,
    I32,
    U8,
    U16,
    U32,
    U64,
    F32,
}

/// Every conversion method name, `to_i8` ... `to_f32`, with its target.
pub const CONVERSIONS: &[(&str, Num)] = &[
    ("to_i8", Num::I8),
    ("to_i16", Num::I16),
    ("to_i32", Num::I32),
    ("to_u8", Num::U8),
    ("to_u16", Num::U16),
    ("to_u32", Num::U32),
    ("to_u64", Num::U64),
    ("to_f32", Num::F32),
];

impl Num {
    pub fn from_name(name: &str) -> Option<Num> {
        Some(match name {
            "i8" => Num::I8,
            "i16" => Num::I16,
            "i32" => Num::I32,
            "u8" => Num::U8,
            "u16" => Num::U16,
            "u32" => Num::U32,
            "u64" => Num::U64,
            "f32" => Num::F32,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Num::I8 => "i8",
            Num::I16 => "i16",
            Num::I32 => "i32",
            Num::U8 => "u8",
            Num::U16 => "u16",
            Num::U32 => "u32",
            Num::U64 => "u64",
            Num::F32 => "f32",
        }
    }

    pub fn is_float(self) -> bool {
        self == Num::F32
    }

    pub fn is_signed(self) -> bool {
        matches!(self, Num::I8 | Num::I16 | Num::I32)
    }

    /// The smallest and largest value of an integer type.
    pub fn range(self) -> (i128, i128) {
        match self {
            Num::I8 => (i8::MIN.into(), i8::MAX.into()),
            Num::I16 => (i16::MIN.into(), i16::MAX.into()),
            Num::I32 => (i32::MIN.into(), i32::MAX.into()),
            Num::U8 => (0, u8::MAX.into()),
            Num::U16 => (0, u16::MAX.into()),
            Num::U32 => (0, u32::MAX.into()),
            Num::U64 => (0, u64::MAX.into()),
            Num::F32 => unreachable!("f32 has no integer range"),
        }
    }

    pub fn fits(self, v: i128) -> bool {
        let (lo, hi) = self.range();
        (lo..=hi).contains(&v)
    }

    /// The mathematical value of an integer's stored bits.
    pub fn decode(self, bits: i64) -> i128 {
        if self == Num::U64 { i128::from(bits as u64) } else { i128::from(bits) }
    }

    /// The stored bits of an in-range integer value.
    pub fn encode(self, v: i128) -> i64 {
        debug_assert!(self.fits(v));
        v as i64
    }

    pub fn f32_of(bits: i64) -> f32 {
        f32::from_bits(bits as u32)
    }

    pub fn f32_bits(x: f32) -> i64 {
        i64::from(x.to_bits())
    }
}
