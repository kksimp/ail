//! Character classes of `std.regex` (std.regex.10): the ASCII Perl and POSIX
//! classes, and the subset of Unicode general categories and scripts the
//! bootstrap carries. The Unicode classes are approximations built on the
//! standard library's character properties and fixed ranges (ADR 0439).

/// A named set of characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Digit,
    Word,
    Space,
    Alpha,
    Alnum,
    Ascii,
    Blank,
    Cntrl,
    Graph,
    Lower,
    Print,
    Punct,
    Upper,
    XDigit,
    /// `\p{L}`.
    Letter,
    /// `\p{Lu}`.
    UpperLetter,
    /// `\p{Ll}`.
    LowerLetter,
    /// `\p{N}`.
    Number,
    /// `\p{Nd}`.
    DecimalNumber,
    /// `\p{P}`.
    Punctuation,
    /// `\p{S}`.
    Symbol,
    /// `\p{Z}`.
    Separator,
    /// `\p{Zs}`.
    SpaceSeparator,
    /// `\p{Cc}`.
    Control,
    /// `\p{Alphabetic}`.
    Alphabetic,
    /// `\p{White_Space}`.
    WhiteSpace,
    /// `\p{Any}`.
    Any,
    Script(&'static [(u32, u32)]),
}

/// The Unicode class names `\p{...}` accepts, for the error message.
pub const UNICODE_NAMES: &str = "L, Lu, Ll, N, Nd, P, S, Z, Zs, Cc, Letter, Uppercase_Letter, Lowercase_Letter, \
Number, Decimal_Number, Punctuation, Symbol, Separator, Alphabetic, White_Space, Any, Latin, Greek, Cyrillic, \
Han, Hiragana, Katakana, Hangul, Arabic, Hebrew, Thai, Devanagari";

const LATIN: &[(u32, u32)] = &[
    (0x41, 0x5A),
    (0x61, 0x7A),
    (0xAA, 0xAA),
    (0xBA, 0xBA),
    (0xC0, 0xD6),
    (0xD8, 0xF6),
    (0xF8, 0x2B8),
    (0x1E00, 0x1EFF),
    (0x2C60, 0x2C7F),
    (0xA720, 0xA7FF),
    (0xFF21, 0xFF3A),
    (0xFF41, 0xFF5A),
];
const GREEK: &[(u32, u32)] =
    &[(0x370, 0x373), (0x375, 0x377), (0x37A, 0x37D), (0x37F, 0x37F), (0x384, 0x3E1), (0x3F0, 0x3FF), (0x1F00, 0x1FFE)];
const CYRILLIC: &[(u32, u32)] = &[(0x400, 0x52F), (0x1C80, 0x1C88), (0x2DE0, 0x2DFF), (0xA640, 0xA69F)];
const HAN: &[(u32, u32)] = &[
    (0x2E80, 0x2FD5),
    (0x3005, 0x3005),
    (0x3007, 0x3007),
    (0x3021, 0x3029),
    (0x3038, 0x303B),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xF900, 0xFAD9),
    (0x20000, 0x323AF),
];
const HIRAGANA: &[(u32, u32)] = &[(0x3041, 0x3096), (0x309D, 0x309F), (0x1B001, 0x1B11F)];
const KATAKANA: &[(u32, u32)] =
    &[(0x30A1, 0x30FA), (0x30FD, 0x30FF), (0x31F0, 0x31FF), (0xFF66, 0xFF6F), (0xFF71, 0xFF9D)];
const HANGUL: &[(u32, u32)] =
    &[(0x1100, 0x11FF), (0x3131, 0x318E), (0xA960, 0xA97C), (0xAC00, 0xD7A3), (0xD7B0, 0xD7FB)];
const ARABIC: &[(u32, u32)] =
    &[(0x600, 0x604), (0x606, 0x6FF), (0x750, 0x77F), (0x8A0, 0x8FF), (0xFB50, 0xFDFF), (0xFE70, 0xFEFC)];
const HEBREW: &[(u32, u32)] = &[(0x591, 0x5C7), (0x5D0, 0x5F4), (0xFB1D, 0xFB4F)];
const THAI: &[(u32, u32)] = &[(0xE01, 0xE3A), (0xE40, 0xE5B)];
const DEVANAGARI: &[(u32, u32)] = &[(0x900, 0x950), (0x955, 0x963), (0x966, 0x97F), (0xA8E0, 0xA8FF)];

const DECIMAL: &[(u32, u32)] = &[
    (0x30, 0x39),
    (0x660, 0x669),
    (0x6F0, 0x6F9),
    (0x7C0, 0x7C9),
    (0x966, 0x96F),
    (0x9E6, 0x9EF),
    (0xA66, 0xA6F),
    (0xAE6, 0xAEF),
    (0xB66, 0xB6F),
    (0xBE6, 0xBEF),
    (0xC66, 0xC6F),
    (0xCE6, 0xCEF),
    (0xD66, 0xD6F),
    (0xE50, 0xE59),
    (0xED0, 0xED9),
    (0xF20, 0xF29),
    (0x1040, 0x1049),
    (0x17E0, 0x17E9),
    (0x1810, 0x1819),
    (0xFF10, 0xFF19),
];
const PUNCT: &[(u32, u32)] = &[
    (0xA1, 0xA1),
    (0xA7, 0xA7),
    (0xAB, 0xAB),
    (0xB6, 0xB7),
    (0xBB, 0xBB),
    (0xBF, 0xBF),
    (0x2010, 0x2027),
    (0x2030, 0x2043),
    (0x2045, 0x2051),
    (0x2053, 0x205E),
    (0x3001, 0x3003),
    (0x3008, 0x3011),
    (0x3014, 0x301F),
    (0xFF01, 0xFF03),
    (0xFF05, 0xFF0A),
    (0xFF0C, 0xFF0F),
    (0xFF1A, 0xFF1B),
    (0xFF1F, 0xFF20),
    (0xFF3B, 0xFF3D),
    (0xFF3F, 0xFF3F),
    (0xFF5B, 0xFF5B),
    (0xFF5D, 0xFF5D),
    (0xFF5F, 0xFF65),
];
const SYMBOL: &[(u32, u32)] = &[
    (0xA2, 0xA6),
    (0xA8, 0xA9),
    (0xAC, 0xAC),
    (0xAE, 0xB1),
    (0xB4, 0xB4),
    (0xB8, 0xB8),
    (0xD7, 0xD7),
    (0xF7, 0xF7),
    (0x20A0, 0x20C0),
    (0x2190, 0x23FF),
    (0x2500, 0x27BF),
    (0x2900, 0x2BFF),
    (0x1F300, 0x1FAFF),
];
const SEPARATOR: &[(u32, u32)] = &[
    (0x20, 0x20),
    (0xA0, 0xA0),
    (0x1680, 0x1680),
    (0x2000, 0x200A),
    (0x2028, 0x2029),
    (0x202F, 0x202F),
    (0x205F, 0x205F),
    (0x3000, 0x3000),
];

fn in_ranges(r: &[(u32, u32)], c: char) -> bool {
    let c = c as u32;
    r.iter().any(|&(lo, hi)| lo <= c && c <= hi)
}

impl Prop {
    /// The POSIX class `[:name:]` (std.regex.10).
    pub fn posix(name: &str) -> Option<Prop> {
        Some(match name {
            "alpha" => Prop::Alpha,
            "alnum" => Prop::Alnum,
            "ascii" => Prop::Ascii,
            "blank" => Prop::Blank,
            "cntrl" => Prop::Cntrl,
            "digit" => Prop::Digit,
            "graph" => Prop::Graph,
            "lower" => Prop::Lower,
            "print" => Prop::Print,
            "punct" => Prop::Punct,
            "space" => Prop::Space,
            "upper" => Prop::Upper,
            "word" => Prop::Word,
            "xdigit" => Prop::XDigit,
            _ => return None,
        })
    }

    /// The Unicode class `\p{name}` (std.regex.10).
    pub fn unicode(name: &str) -> Option<Prop> {
        Some(match name {
            "L" | "Letter" => Prop::Letter,
            "Lu" | "Uppercase_Letter" => Prop::UpperLetter,
            "Ll" | "Lowercase_Letter" => Prop::LowerLetter,
            "N" | "Number" => Prop::Number,
            "Nd" | "Decimal_Number" => Prop::DecimalNumber,
            "P" | "Punctuation" => Prop::Punctuation,
            "S" | "Symbol" => Prop::Symbol,
            "Z" | "Separator" => Prop::Separator,
            "Zs" => Prop::SpaceSeparator,
            "Cc" => Prop::Control,
            "Alphabetic" => Prop::Alphabetic,
            "White_Space" => Prop::WhiteSpace,
            "Any" => Prop::Any,
            "Latin" => Prop::Script(LATIN),
            "Greek" => Prop::Script(GREEK),
            "Cyrillic" => Prop::Script(CYRILLIC),
            "Han" => Prop::Script(HAN),
            "Hiragana" => Prop::Script(HIRAGANA),
            "Katakana" => Prop::Script(KATAKANA),
            "Hangul" => Prop::Script(HANGUL),
            "Arabic" => Prop::Script(ARABIC),
            "Hebrew" => Prop::Script(HEBREW),
            "Thai" => Prop::Script(THAI),
            "Devanagari" => Prop::Script(DEVANAGARI),
            _ => return None,
        })
    }

    /// Is `c` in the class?
    pub fn test(self, c: char) -> bool {
        match self {
            Prop::Digit => c.is_ascii_digit(),
            Prop::Word => c.is_ascii_alphanumeric() || c == '_',
            Prop::Space => matches!(c, '\t' | '\n' | '\x0B' | '\x0C' | '\r' | ' '),
            Prop::Alpha => c.is_ascii_alphabetic(),
            Prop::Alnum => c.is_ascii_alphanumeric(),
            Prop::Ascii => c.is_ascii(),
            Prop::Blank => c == ' ' || c == '\t',
            Prop::Cntrl => c.is_ascii_control(),
            Prop::Graph => c.is_ascii_graphic(),
            Prop::Lower => c.is_ascii_lowercase(),
            Prop::Print => c.is_ascii_graphic() || c == ' ',
            Prop::Punct => c.is_ascii_punctuation(),
            Prop::Upper => c.is_ascii_uppercase(),
            Prop::XDigit => c.is_ascii_hexdigit(),
            Prop::Letter => c.is_alphabetic() && !c.is_numeric(),
            Prop::UpperLetter => c.is_uppercase(),
            Prop::LowerLetter => c.is_lowercase(),
            Prop::Number => c.is_numeric(),
            Prop::DecimalNumber => in_ranges(DECIMAL, c),
            Prop::Punctuation => {
                if c.is_ascii() {
                    c.is_ascii_punctuation() && !"$+<=>^`|~".contains(c)
                } else {
                    in_ranges(PUNCT, c)
                }
            }
            Prop::Symbol => {
                if c.is_ascii() {
                    "$+<=>^`|~".contains(c)
                } else {
                    in_ranges(SYMBOL, c)
                }
            }
            Prop::Separator => in_ranges(SEPARATOR, c),
            Prop::SpaceSeparator => in_ranges(SEPARATOR, c) && c != '\u{2028}' && c != '\u{2029}',
            Prop::Control => c.is_control(),
            Prop::Alphabetic => c.is_alphabetic(),
            Prop::WhiteSpace => c.is_whitespace(),
            Prop::Any => true,
            Prop::Script(r) => in_ranges(r, c),
        }
    }
}
