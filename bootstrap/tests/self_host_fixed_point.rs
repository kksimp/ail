//! The native compiler builds itself to a fixed point (ADR 0022, ADR 0610):
//! stage 1 (`ail build compiler/main.ail`) builds stage 2, stage 2 builds
//! stage 3, and stages 2 and 3 are byte-identical. A few native tests are then
//! built by the self-hosted compiler and must behave as under `ail run`.
//! `tools/self-host/run.sh` does the work and prints where it stops; it
//! stops short (exit 0) when the target can't run on this host.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

#[test]
fn compiler_builds_itself_to_a_fixed_point() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("self_host_fixed_point");
    let _ = std::fs::remove_dir_all(&out);
    let run = Command::new(root().join("tools/self-host/run.sh"))
        .arg(&out)
        .env("ONLY", "arith,args,addr_fold")
        .current_dir(root())
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&run.stdout);
    let _ = std::fs::remove_dir_all(&out);
    assert!(run.status.success(), "self-host bootstrap did not reach the fixed point:\n{report}");
}
