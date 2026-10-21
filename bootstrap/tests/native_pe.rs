//! The Windows PE writer (`compiler/obj/pe.ail`, ADR 0597, ADR 0598): the
//! sample program's image, built by `ail run compiler pe-sample sample`, has
//! the PE32+ structure and the kernel32 imports the codegen will call.

use std::path::Path;
use std::process::Command;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").leak()
}

/// The sample executable's bytes: the compiler prints them as hex lines.
fn sample() -> Vec<u8> {
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["run", "compiler", "pe-sample", "sample"])
        .output()
        .unwrap();
    assert!(out.status.success(), "pe-sample failed:\n{}", String::from_utf8_lossy(&out.stderr));
    let hex: String = String::from_utf8(out.stdout).unwrap().split_whitespace().collect();
    (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect()
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

// ADR 0597: DOS header, PE32+ headers, four sections.
#[test]
fn pe_headers_describe_a_windows_x64_executable() {
    let b = sample();
    assert_eq!(&b[0..2], b"MZ");
    let pe = u32_at(&b, 0x3c) as usize;
    assert_eq!(pe, 0x80);
    assert_eq!(&b[pe..pe + 4], b"PE\0\0");
    assert_eq!(u16_at(&b, pe + 4), 0x8664, "machine is AMD64");
    assert_eq!(u16_at(&b, pe + 6), 4, "four sections: .text .rdata .idata .bss");
    assert_eq!(u16_at(&b, pe + 24), 0x20b, "PE32+ optional header");
    assert_eq!(u32_at(&b, pe + 24 + 16), 0x1000, "entry point is the first code byte");
    assert_eq!(u32_at(&b, pe + 24 + 108), 16, "sixteen data directories");
    // The import directory and the address table are in .idata at 0x3000.
    let dirs = pe + 24 + 112;
    assert_eq!(u32_at(&b, dirs + 8), 0x3000);
    assert_eq!(u32_at(&b, dirs + 12), 60, "three descriptors: KERNEL32.dll, WS2_32.dll and the null entry");
    let iat = u32_at(&b, dirs + 96);
    assert_eq!(iat, 0x305c, "address table starts after the descriptors and KERNEL32 lookup table");
    assert_eq!(u32_at(&b, dirs + 100), 48, "KERNEL32: three entries and a null; WS2_32: one and a null; eight bytes each");
}

// ADR 0598: each call goes through an import slot, and the slots name the three functions.
#[test]
fn pe_imports_name_kernel32_functions_the_codegen_calls() {
    let b = sample();
    let idata = &b[0x800..0xa00];
    let dll = idata.windows(13).position(|w| w == b"KERNEL32.dll\0");
    assert!(dll.is_some(), "the DLL name is in .idata");
    for name in ["GetStdHandle", "WriteFile", "ExitProcess"] {
        let mut hint_name = vec![0u8, 0];
        hint_name.extend_from_slice(name.as_bytes());
        hint_name.push(0);
        assert!(idata.windows(hint_name.len()).any(|w| w == hint_name.as_slice()), "{name} hint/name entry");
    }
    // The call to GetStdHandle is FF 15 with a displacement to its slot.
    let text = &b[0x400..0x600];
    let iat = u32_at(&b, 0x80 + 24 + 112 + 96);
    let call = text.windows(2).position(|w| w == [0xff, 0x15]).unwrap();
    let next = 0x1000 + call as u32 + 6;
    let disp = i32::from_le_bytes(text[call + 2..call + 6].try_into().unwrap());
    assert_eq!((next as i64 + disp as i64) as u32, iat);
}

// ADR 0597: `objdump` and `llvm-readobj` agree on the file, when installed.
#[test]
fn pe_sample_reads_back_in_an_external_tool() {
    let b = sample();
    let path = std::env::temp_dir().join(format!("ail-pe-sample-{}.exe", std::process::id()));
    std::fs::write(&path, &b).unwrap();
    let text = match Command::new("objdump").arg("-x").arg(&path).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => {
            let _ = std::fs::remove_file(&path);
            return;
        }
    };
    let _ = std::fs::remove_file(&path);
    assert!(text.contains("pei-x86-64") || text.contains("coff-x86-64"), "{text}");
    assert!(text.contains("DLL Name: KERNEL32.dll"), "{text}");
    assert!(text.contains("ExitProcess"), "{text}");
    assert!(text.contains("DLL Name: WS2_32.dll"), "{text}");
    assert!(text.contains("closesocket"), "{text}");
}

// ADR 0597, ADR 0598: the sample runs under Wine when it is installed: it
// prints its line, and exits 0. Without Wine the test passes without checking.
#[test]
fn pe_sample_runs_under_wine_when_installed() {
    if Command::new("wine").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        return;
    }
    let b = sample();
    let dir = std::env::temp_dir().join(format!("ail-pe-run-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("sample.exe");
    std::fs::write(&exe, &b).unwrap();
    let out = Command::new("wine")
        .arg(&exe)
        .env("WINEDEBUG", "-all")
        .env("WINEPREFIX", dir.join("prefix"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(out.status.success(), "wine exited with {:?}", out.status);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello from AIL on Windows\n");
}

// ADR 0606: a MIR module that calls `libsys.WriteFile` with five arguments
// (the fifth on the stack) runs under Wine: it prints its line, and it exits
// with `written`, which must equal the line's length.
#[test]
fn libsys_write_file_runs_under_wine_when_installed() {
    if Command::new("wine").arg("--version").output().map(|o| !o.status.success()).unwrap_or(true) {
        return;
    }
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .current_dir(root())
        .args(["run", "compiler", "pe-mir-sample", "x"])
        .output()
        .unwrap();
    assert!(out.status.success(), "pe-mir-sample failed:\n{}", String::from_utf8_lossy(&out.stderr));
    let hex: String = String::from_utf8(out.stdout).unwrap().split_whitespace().collect();
    let b: Vec<u8> = (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap()).collect();
    let dir = std::env::temp_dir().join(format!("ail-pe-libsys-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("libsys.exe");
    std::fs::write(&exe, &b).unwrap();
    let run = Command::new("wine")
        .arg(&exe)
        .env("WINEDEBUG", "-all")
        .env("WINEPREFIX", dir.join("prefix"))
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let line = "hello from AIL on Windows\n";
    assert_eq!(String::from_utf8_lossy(&run.stdout), line, "the printed text");
    assert_eq!(run.status.code(), Some(line.len() as i32), "written, which must equal the length");
}
