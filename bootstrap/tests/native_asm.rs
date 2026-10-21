//! The instruction encoders of the native backend (`compiler/x64/asm.ail`,
//! ADR 0385, and `compiler/a64/asm.ail`, ADR 0388) against an external
//! disassembler. ADR 0010 allows objdump only in tests, never in the build.
//!
//! Each driver in `tests/native/asm/` encodes a list of instructions and
//! prints `intent|bytes` per case. The module under test is copied next to
//! the driver at run time, so the tests use the compiler's own files. Then
//! x86-64 bytes go through GNU objdump, and ARM64 words through llvm-mc, and
//! the disassembly must match the intent. Missing tools skip with a message.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn have(tool: &str) -> bool {
    match Command::new(tool).arg("--version").output() {
        // macOS ships LLVM's objdump under the GNU name; it has no `-b binary`.
        Ok(out) if tool == "objdump" => String::from_utf8_lossy(&out.stdout).contains("GNU"),
        Ok(_) => true,
        Err(_) => false,
    }
}

/// One encoded case: the intent and the bytes the encoder produced.
struct Case {
    intent: String,
    bytes: Vec<u8>,
}

/// Copies `module` to `<tmp>/<dir>/asm/asm.ail` and `driver` to `<tmp>/main.ail`, runs
/// it with `ail run`, and parses the `intent|bytes` lines.
fn run_driver(name: &str, dir: &str, module: &str, driver: &str) -> Vec<Case> {
    let tmp = std::env::temp_dir().join(format!("ail-asm-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(tmp.join(dir).join("asm")).unwrap();
    fs::copy(root().join(module), tmp.join(dir).join("asm").join("asm.ail")).unwrap();
    fs::copy(root().join(driver), tmp.join("main.ail")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["run"])
        .arg(tmp.join("main.ail"))
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&tmp);
    assert!(out.status.success(), "{driver} failed:\n{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let (intent, bytes) = line.split_once('|').unwrap_or_else(|| panic!("bad line {line:?}"));
            Case { intent: intent.to_string(), bytes: bytes.split_whitespace().map(|b| b.parse().unwrap()).collect() }
        })
        .collect()
}

/// Lower-cases and splits an instruction into comparable tokens: punctuation
/// and size words are dropped, immediates become decimal, and zero is dropped
/// (objdump omits a zero displacement that the intent writes out).
fn norm(text: &str) -> Vec<String> {
    let text = text.to_ascii_lowercase();
    let text = text.replace(['[', ']', ',', '*', '+', '!'], " ").replace('-', " -");
    let text = text.replace('#', " ");
    text.split_whitespace()
        .filter(|w| !matches!(*w, "qword" | "dword" | "word" | "byte" | "ptr" | "xmmword"))
        .filter_map(|w| match parse_num(w) {
            Some(0) => None,
            Some(v) => Some(v.to_string()),
            None => Some(w.to_string()),
        })
        .collect()
}

/// Reads `0x..` (as unsigned 64-bit, so `0xffff..ff` is -1) or decimal.
fn parse_num(w: &str) -> Option<i128> {
    let (neg, digits) = match w.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, w),
    };
    let v = if let Some(hex) = digits.strip_prefix("0x") {
        let u = u64::from_str_radix(hex, 16).ok()?;
        if u >= 1 << 63 { u as i128 - (1i128 << 64) } else { u as i128 }
    } else {
        digits.parse::<i128>().ok()?
    };
    Some(if neg { -v } else { v })
}

/// Splits an intent into instructions (`; `) and normalizes each.
fn expected(intent: &str) -> Vec<Vec<String>> {
    intent.split("; ").map(norm).collect()
}

/// The x86-64 instructions objdump prints for `bytes`, Intel syntax.
fn objdump_x86(bytes: &[u8], file: &Path) -> Result<Vec<String>, String> {
    fs::write(file, bytes).unwrap();
    let out = Command::new("objdump")
        .args(["-D", "-b", "binary", "-m", "i386:x86-64", "-M", "intel"])
        .arg(file)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    let mut insns = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        // `   0:\t48 89 c1 \tmov    rcx,rax`; long instructions continue
        // their bytes on lines with no text.
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() >= 3 && fields[0].trim_end().ends_with(':') && !fields[2].trim().is_empty() {
            // A RIP-relative operand is followed by objdump's `# 0xADDR` comment.
            let text = fields[2].split(" #").next().unwrap_or("");
            insns.push(text.trim().to_string());
        }
    }
    Ok(insns)
}

#[test]
fn x86_64_encodings_match_objdump() {
    if !have("objdump") {
        eprintln!("skipping: objdump is not installed");
        return;
    }
    let cases = run_driver("x64", "x64", "compiler/x64/asm.ail", "bootstrap/tests/native/asm/x64_encoder.ail");
    assert!(cases.len() > 300, "the driver printed only {} cases", cases.len());
    let file = std::env::temp_dir().join(format!("ail-asm-x64-{}.bin", std::process::id()));
    let mut failures = Vec::new();
    for case in &cases {
        let want = expected(&case.intent);
        let got = match objdump_x86(&case.bytes, &file) {
            Ok(insns) => insns,
            Err(e) => {
                failures.push(format!("{}: objdump failed: {e}", case.intent));
                continue;
            }
        };
        let got: Vec<Vec<String>> = got.iter().map(|i| norm(i)).collect();
        if want != got {
            failures.push(format!(
                "{} | {:02x?}: objdump says {:?}",
                case.intent,
                case.bytes,
                got.iter().map(|g| g.join(" ")).collect::<Vec<_>>()
            ));
        }
    }
    let _ = fs::remove_file(&file);
    assert!(failures.is_empty(), "{} of {} encodings differ:\n{}", failures.len(), cases.len(), failures.join("\n"));
}

/// The ARM64 instructions llvm-mc prints for `words`, one per 32-bit word.
fn llvm_mc_a64(bytes: &[u8]) -> Result<Vec<String>, String> {
    let hex: Vec<String> = bytes.iter().map(|b| format!("0x{b:02x}")).collect();
    let mut child = Command::new("llvm-mc")
        .args(["--disassemble", "-triple=aarch64"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    child.stdin.take().unwrap().write_all(hex.join(" ").as_bytes()).unwrap();
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        // Aliases end in a `// =0x..` comment with the value; `.text` is the
        // section directive llvm-mc prints first, not an instruction.
        .map(|l| l.split(" //").next().unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty() && !l.starts_with('.'))
        .collect())
}

fn parse_imm(w: &str) -> Option<i128> {
    parse_num(w.trim_start_matches('#'))
}

/// Runs the instructions for a `const xD=V` case and returns the value of xD
/// they compute: movz, movk and movn by 16-bit part, or llvm's `mov` alias.
fn simulate_const(lines: &[String], d: usize) -> Result<u64, String> {
    let mut reg: u64 = 0;
    for line in lines {
        let (mnemonic, rest) = line.split_once(' ').ok_or(format!("no operands: {line}"))?;
        let ops: Vec<&str> = rest.split(',').map(str::trim).collect();
        if ops[0] != format!("x{d}") {
            return Err(format!("{line} does not write x{d}"));
        }
        let imm = parse_imm(ops[1]).ok_or(format!("bad immediate in {line}"))?;
        let shift = match ops.get(2) {
            Some(s) => parse_imm(s.trim_start_matches("lsl ")).ok_or(format!("bad shift in {line}"))?,
            None => 0,
        };
        let part = (imm as u64) << shift;
        reg = match mnemonic {
            "movz" => part,
            "movk" => (reg & !(0xffffu64 << shift)) | part,
            "movn" => !part,
            "mov" => imm as u64,
            _ => return Err(format!("unexpected instruction {line}")),
        };
    }
    Ok(reg)
}

#[test]
fn aarch64_encodings_match_llvm_mc() {
    if !have("llvm-mc") {
        eprintln!("skipping: llvm-mc is not installed");
        return;
    }
    let cases = run_driver("a64", "a64", "compiler/a64/asm.ail", "bootstrap/tests/native/asm/a64_encoder.ail");
    assert!(cases.len() > 100, "the driver printed only {} cases", cases.len());
    let mut failures = Vec::new();
    for case in &cases {
        let lines = match llvm_mc_a64(&case.bytes) {
            Ok(lines) => lines,
            Err(e) => {
                failures.push(format!("{}: llvm-mc failed: {e}", case.intent));
                continue;
            }
        };
        if let Some(rest) = case.intent.strip_prefix("const x") {
            // A constant is checked by the value the instructions compute.
            let (d, v) = rest.split_once('=').unwrap();
            let want: i64 = v.parse().unwrap();
            match simulate_const(&lines, d.parse().unwrap()) {
                Ok(got) if got == want as u64 => {}
                Ok(got) => failures.push(format!("{}: computes {got:#x} from {lines:?}", case.intent)),
                Err(e) => failures.push(format!("{}: {e}", case.intent)),
            }
            continue;
        }
        let got: Vec<Vec<String>> = lines.iter().map(|l| norm(l)).collect();
        if expected(&case.intent) != got {
            failures.push(format!("{} | {:02x?}: llvm-mc says {lines:?}", case.intent, case.bytes));
        }
    }
    assert!(failures.is_empty(), "{} of {} encodings differ:\n{}", failures.len(), cases.len(), failures.join("\n"));
}
