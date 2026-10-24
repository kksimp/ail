//! Helpers shared by the end-to-end test files.

use std::path::Path;

/// A path as tool output spells it: `/` separators on every host (ADR 0799).
#[allow(dead_code)]
pub fn shown(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

/// Failures that are known gaps on Windows, not regressions: the example calls
/// a feature that is not there yet, and the reason is printed as a skip.
/// `None` on other hosts, and for any failure that is not one of these gaps.
#[allow(dead_code)]
pub fn windows_gap(failure: &str) -> Option<&'static str> {
    if !cfg!(windows) {
        return None;
    }
    if failure.contains("trap[ffi.load.") {
        Some("C calls are not on Windows yet (ffi.load.N)")
    } else if failure.contains("cmd: \"tar\", kind: \"not_found\"") {
        Some("needs `tar`, which this host does not have")
    } else {
        None
    }
}
