//! Secure random bytes from the operating system, for TLS randoms and
//! X25519 private keys (ADR 0367). `std.random` is a seeded SplitMix64 and
//! must never be used for keys.

use std::io;

/// Fills `buf` from the OS generator, or fails; it never falls back to a
/// weaker source.
pub fn fill(buf: &mut [u8]) -> io::Result<()> {
    imp::fill(buf)
}

#[cfg(unix)]
mod imp {
    use std::io::{self, Read};

    pub fn fill(buf: &mut [u8]) -> io::Result<()> {
        std::fs::File::open("/dev/urandom")?.read_exact(buf)
    }
}

#[cfg(windows)]
mod imp {
    use std::io;

    // BCryptGenRandom with BCRYPT_USE_SYSTEM_PREFERRED_RNG (bcrypt.dll,
    // Windows Vista and later).
    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(alg: *mut core::ffi::c_void, buf: *mut u8, len: u32, flags: u32) -> i32;
    }

    pub fn fill(buf: &mut [u8]) -> io::Result<()> {
        for chunk in buf.chunks_mut(u32::MAX as usize) {
            // SAFETY: the pointer and length describe a live, writable slice.
            let status = unsafe { BCryptGenRandom(core::ptr::null_mut(), chunk.as_mut_ptr(), chunk.len() as u32, 2) };
            if status != 0 {
                return Err(io::Error::other(format!("BCryptGenRandom failed with status {status:#x}")));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fills_and_differs() {
        let (mut a, mut b) = ([0u8; 32], [0u8; 32]);
        super::fill(&mut a).unwrap();
        super::fill(&mut b).unwrap();
        assert_ne!(a, b);
        assert_ne!(a, [0; 32]);
    }
}
