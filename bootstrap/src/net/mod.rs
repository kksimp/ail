//! Networking internals for `std.net` (spec 17-std, ADR 0283, ADR 0360).
//! The HTTP client itself lives in `interp/net.rs`; this module holds the
//! in-house TLS 1.3 client it will use for `https` (ADR 0360).

pub mod crypto;
