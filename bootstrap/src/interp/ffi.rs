//! C calls from the interpreter (ffi.call.2, ffi.load.1-3, ADRs 0431-0432).
//!
//! Toolchain code, not Runtime Code: it stays in `ail` and is never linked
//! into a user's program. What a native build links in for `extern c fn`
//! (a dynamic-link stub per symbol) is specified in
//! `docs/design/native-abi.md` and falls under the licence's Runtime
//! Library Exception (ADR 0510).
//!
//! A libffi-free design: on x86-64 System V and on ARM64 (AAPCS64, Apple's
//! variant included) integer-class and float-class arguments travel in two
//! separate register files. One function pointer type that takes six
//! `i64`s and then eight `f64`s therefore calls any C function with up to
//! six integer-class and eight float-class arguments, whatever their
//! order: the integers fill the integer registers in order, the floats the
//! float registers in order, and the registers a callee doesn't read are
//! ignored. The result comes back in the integer or the float register, so
//! two function pointer types cover every signature class. Variadic
//! functions, structs by value, and more arguments than registers are
//! refused by the checker (ffi.decl.3).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use super::{Env, Exit, Flow, Interp, Trap, Value, trap};
use crate::ast::{FnDecl, TypeKind, TypeRef};
use crate::diag::Pos;
use crate::num::Num;

/// How a parameter or result travels (ffi.map.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Class {
    /// `int`, `i64`, a pointer: the 64 bits as they are.
    Word,
    Float,
    Bool,
    /// A sized number other than `f32`.
    Num(Num),
    F32,
    Unit,
}

fn class(t: &TypeRef) -> Class {
    match &t.kind {
        TypeKind::Named { name, .. } => match name.as_str() {
            "float" | "f64" => Class::Float,
            "bool" => Class::Bool,
            n => match Num::from_name(n) {
                Some(Num::F32) => Class::F32,
                Some(k) => Class::Num(k),
                None => Class::Word,
            },
        },
        _ => Class::Word,
    }
}

/// A result, normalised to the declared type: C leaves the upper bits of a
/// narrow integer result unspecified (ffi.map.2).
fn word_result(bits: i64, c: Class) -> Value {
    match c {
        Class::Bool => Value::Bool(bits & 0xff != 0),
        Class::Num(k) => {
            let v = match k {
                Num::I8 => bits as i8 as i64,
                Num::I16 => bits as i16 as i64,
                Num::I32 => bits as i32 as i64,
                Num::U8 => bits as u8 as i64,
                Num::U16 => bits as u16 as i64,
                Num::U32 => bits as u32 as i64,
                Num::U64 | Num::F32 => bits,
            };
            Value::Num(k, v)
        }
        _ => Value::Int(bits),
    }
}

#[cfg(unix)]
mod sys {
    use std::ffi::{CStr, CString, c_char, c_int, c_void};

    #[cfg_attr(target_os = "linux", link(name = "dl"))]
    unsafe extern "C" {
        fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
        fn dlerror() -> *const c_char;
    }

    const RTLD_NOW: c_int = 2;

    fn last_error() -> String {
        // SAFETY: `dlerror` returns null or a NUL-terminated string valid until the next dl call.
        let p = unsafe { dlerror() };
        if p.is_null() { String::new() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
    }

    /// The process itself when `path` is `None`.
    pub fn open(path: Option<&str>) -> Result<usize, String> {
        let c = match path {
            Some(p) => Some(CString::new(p).map_err(|_| "a library name can't contain NUL".to_string())?),
            None => None,
        };
        // SAFETY: a valid C string or null; `dlopen` has no other preconditions.
        let h = unsafe { dlopen(c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()), RTLD_NOW) };
        if h.is_null() { Err(last_error()) } else { Ok(h as usize) }
    }

    pub fn symbol(handle: usize, name: &str) -> Result<usize, String> {
        let c = CString::new(name).map_err(|_| "a symbol name can't contain NUL".to_string())?;
        // SAFETY: `handle` came from `dlopen`; `c` is a valid C string.
        let p = unsafe { dlsym(handle as *mut c_void, c.as_ptr()) };
        if p.is_null() { Err(last_error()) } else { Ok(p as usize) }
    }

    pub type IntFn = unsafe extern "C" fn(i64, i64, i64, i64, i64, i64, f64, f64, f64, f64, f64, f64, f64, f64) -> i64;
    pub type FloatFn = unsafe extern "C" fn(i64, i64, i64, i64, i64, i64, f64, f64, f64, f64, f64, f64, f64, f64) -> f64;

    /// Call `f` with these register images; `float_result` picks the
    /// register the result is read from.
    ///
    /// # Safety
    /// `f` must be the address of a C function whose parameters fit these registers.
    pub unsafe fn call(f: usize, i: [i64; 6], d: [f64; 8], float_result: bool) -> (i64, f64) {
        // SAFETY: the caller promises `f` is a C function of a compatible shape.
        unsafe {
            if float_result {
                let g: FloatFn = std::mem::transmute(f);
                (0, g(i[0], i[1], i[2], i[3], i[4], i[5], d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]))
            } else {
                let g: IntFn = std::mem::transmute(f);
                (g(i[0], i[1], i[2], i[3], i[4], i[5], d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]), 0.0)
            }
        }
    }
}

/// ffi.load.3: Windows needs `LoadLibraryA`/`GetProcAddress` and the Win64
/// convention, whose argument registers are positional, so the two-file
/// trick above doesn't apply. Stubbed until a per-signature trampoline is
/// written and tested there (ADR 0004: Windows doesn't gate).
#[cfg(not(unix))]
mod sys {
    pub fn open(_: Option<&str>) -> Result<usize, String> {
        Err("C calls are not supported on Windows yet (ffi.load.3)".into())
    }
    pub fn symbol(_: usize, _: &str) -> Result<usize, String> {
        Err("C calls are not supported on Windows yet (ffi.load.3)".into())
    }
    pub unsafe fn call(_: usize, _: [i64; 6], _: [f64; 8], _: bool) -> (i64, f64) {
        (0, 0.0)
    }
}

/// Library names a written name may stand for (ffi.load.1): a name with a
/// `/` or a `.` is used as written; `m` and `libm` are tried as the
/// platform spells shared libraries.
fn candidates(lib: &str) -> Vec<String> {
    if lib.contains('/') || lib.contains('.') {
        return vec![lib.to_string()];
    }
    let bases: Vec<String> =
        if lib.starts_with("lib") { vec![lib.to_string()] } else { vec![lib.to_string(), format!("lib{lib}")] };
    let mut out = Vec::new();
    for b in bases {
        if cfg!(target_os = "macos") {
            out.push(format!("{b}.dylib"));
            out.push(format!("/usr/lib/{b}.dylib"));
            out.push(format!("/opt/homebrew/lib/{b}.dylib"));
        } else {
            // Development packages ship `libX.so`; run-time packages only `libX.so.N`.
            for suffix in [".so", ".so.6", ".so.1", ".so.0", ".so.2", ".so.3"] {
                out.push(format!("{b}{suffix}"));
            }
        }
    }
    out
}

type Cache = Mutex<HashMap<(Option<String>, String), usize>>;

fn cache() -> &'static Cache {
    static C: OnceLock<Cache> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The address of `name` in `lib` (the process itself without one),
/// loading the library on first use (ffi.load.1-2).
fn resolve(lib: Option<&str>, name: &str) -> Result<usize, (&'static str, String)> {
    let key = (lib.map(String::from), name.to_string());
    if let Some(&a) = cache().lock().unwrap().get(&key) {
        return Ok(a);
    }
    let handle = match lib {
        None => sys::open(None).map_err(|e| ("ffi.load.1", format!("can't open the running program's symbols: {e}")))?,
        Some(l) => {
            let tried = candidates(l);
            let mut last = String::new();
            let mut found = None;
            for c in &tried {
                match sys::open(Some(c)) {
                    Ok(h) => {
                        found = Some(h);
                        break;
                    }
                    Err(e) => last = e,
                }
            }
            found.ok_or(("ffi.load.1", format!("can't load the library `{l}` (tried {}): {last}", tried.join(", "))))?
        }
    };
    let addr = sys::symbol(handle, name)
        .map_err(|e| ("ffi.load.2", format!("the library {} has no symbol `{name}`: {e}", lib.map_or("(the program)".to_string(), |l| format!("`{l}`")))))?;
    cache().lock().unwrap().insert(key, addr);
    Ok(addr)
}

impl<'a, W: std::io::Write> Interp<'a, W> {
    /// Call the C function `f` declares, with the arguments in `env`
    /// (ffi.call.2).
    pub(super) fn call_extern(&mut self, f: &'a FnDecl, env: &Env<'a>, pos: Pos) -> Result<Value, Trap> {
        let ext = f.ext.as_ref().expect("an extern fn");
        let addr = match resolve(ext.lib.as_deref(), &f.name) {
            Ok(a) => a,
            Err((rule, msg)) => return trap(rule, pos, msg),
        };
        let (mut ints, mut floats) = ([0i64; 6], [0f64; 8]);
        let (mut ni, mut nf) = (0, 0);
        for ((_, v), p) in env.iter().zip(&f.params) {
            match class(&p.ty) {
                Class::Float => {
                    floats[nf] = v.float();
                    nf += 1;
                }
                Class::F32 => {
                    let Value::Num(_, bits) = v else { unreachable!("checked: an f32") };
                    floats[nf] = f64::from_bits(*bits as u32 as u64);
                    nf += 1;
                }
                c => {
                    ints[ni] = match (c, v) {
                        (Class::Bool, Value::Bool(b)) => i64::from(*b),
                        (_, Value::Num(_, bits)) => *bits,
                        (_, v) => v.int(),
                    };
                    ni += 1;
                }
            }
        }
        let ret = f.ret.as_ref().map_or(Class::Unit, class);
        self.flush(pos)?;
        // SAFETY: the declaration is the program's claim about the C function (ffi.call.1:
        // the call is in an `unsafe` block); the checker limited the shape to the registers.
        let (word, float) = unsafe { sys::call(addr, ints, floats, matches!(ret, Class::Float | Class::F32)) };
        Ok(match ret {
            Class::Unit => Value::Unit,
            Class::Float => Value::Float(float),
            Class::F32 => Value::Num(Num::F32, i64::from(float.to_bits() as u32)),
            c => word_result(word, c),
        })
    }
}

/// Memory the program got from `std.mem` and has not freed, by address and
/// size; and the addresses it freed since (unsafe.mem.2).
#[derive(Default)]
struct Blocks {
    live: std::collections::BTreeMap<usize, usize>,
    freed: std::collections::HashSet<usize>,
}

fn blocks() -> &'static Mutex<Blocks> {
    static B: OnceLock<Mutex<Blocks>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(Blocks::default()))
}

#[cfg(unix)]
mod libc {
    use std::ffi::c_void;
    unsafe extern "C" {
        pub fn malloc(n: usize) -> *mut c_void;
        pub fn free(p: *mut c_void);
        pub fn strlen(p: *const std::ffi::c_char) -> usize;
    }
}

impl<'a, W: std::io::Write> Interp<'a, W> {
    /// `std.mem` (unsafe.mem.1-2): C strings in memory from C's own `malloc`,
    /// so a block can be handed to C and freed there, or the other way round.
    pub(super) fn mem_call(&mut self, f: &str, values: &[Value], pos: Pos) -> Flow<Value> {
        #[cfg(not(unix))]
        {
            let _ = (f, values);
            return stop("ffi.load.3", pos, "std.mem is not supported on Windows yet (ffi.load.3)".into());
        }
        #[cfg(unix)]
        {
            let addr = |i: usize| values[i].int() as usize;
            match f {
                "cstr" => {
                    let Value::Str(s) = &values[0] else { unreachable!("checked: a str") };
                    let bytes = s.as_bytes();
                    // SAFETY: `n + 1` bytes are allocated and written before use.
                    let p = unsafe { libc::malloc(bytes.len() + 1) } as *mut u8;
                    if p.is_null() {
                        return stop("unsafe.mem.1", pos, "out of memory".into());
                    }
                    unsafe {
                        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
                        *p.add(bytes.len()) = 0;
                    }
                    let mut b = blocks().lock().unwrap();
                    b.freed.remove(&(p as usize));
                    b.live.insert(p as usize, bytes.len() + 1);
                    Ok(Value::Int(p as i64))
                }
                "from_cstr" => {
                    let a = addr(0);
                    if a == 0 {
                        return stop("unsafe.mem.1", pos, "`from_cstr` of a null pointer".into());
                    }
                    let b = blocks().lock().unwrap();
                    // A block of ours is scanned only up to its end; a pointer from C trusts its NUL.
                    let known = b.live.range(..=a).next_back().filter(|(s, n)| a < **s + **n);
                    let len = match known {
                        Some((s, n)) => {
                            // SAFETY: `a..s+n` lies inside a live block of ours.
                            let bytes = unsafe { std::slice::from_raw_parts(a as *const u8, s + n - a) };
                            match bytes.iter().position(|&c| c == 0) {
                                Some(l) => l,
                                None => {
                                    return stop("unsafe.mem.1", pos, "`from_cstr` found no NUL before the end of the block".into());
                                }
                            }
                        }
                        // SAFETY: the program claims `a` is a C string (it is inside `unsafe`).
                        None => unsafe { libc::strlen(a as *const std::ffi::c_char) },
                    };
                    // SAFETY: `len` bytes were just seen to be readable.
                    let bytes = unsafe { std::slice::from_raw_parts(a as *const u8, len) };
                    Ok(Value::Str(String::from_utf8_lossy(bytes).into_owned().into()))
                }
                "free" => {
                    let a = addr(0);
                    let mut b = blocks().lock().unwrap();
                    if b.live.remove(&a).is_some() {
                        b.freed.insert(a);
                    } else if a == 0 || b.freed.contains(&a) {
                        let why = if a == 0 { "a null pointer" } else { "a pointer that was already freed" };
                        return stop("unsafe.mem.2", pos, format!("`free` of {why}"));
                    }
                    // SAFETY: a block from `malloc` (ours, or the program's claim for one from C).
                    unsafe { libc::free(a as *mut std::ffi::c_void) };
                    Ok(Value::Unit)
                }
                _ => unreachable!("stdlib::MODULES lists `mem.{f}`"),
            }
        }
    }
}

fn stop<T>(rule: &'static str, pos: Pos, message: String) -> Flow<T> {
    Err(Exit::Trap(Box::new(Trap { rule, pos, message })))
}
