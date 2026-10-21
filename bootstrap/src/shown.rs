//! Path text in tool output (ADR 0799): diagnostics, JSON and `ail api`
//! always spell a path with `/`, on every host. The OS form (`\` on Windows)
//! appears only where a path is handed to the operating system.

use std::path::Path;

/// `p` as tool output shows it: `/` separators on every host.
pub fn shown(p: &Path) -> String {
    let s = p.display().to_string();
    if cfg!(windows) {
        s.replace('\\', "/")
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::shown;
    use std::path::Path;

    // ADR 0799: one spelling in tool output.
    #[test]
    fn tool_output_uses_slashes() {
        assert_eq!(shown(Path::new("tests/cases/x.ail")), "tests/cases/x.ail");
        let joined = Path::new("bootstrap").join("tests").join("cases").join("x.ail");
        assert_eq!(shown(&joined), "bootstrap/tests/cases/x.ail");
    }
}
