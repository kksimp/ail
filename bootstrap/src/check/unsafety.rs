//! `unsafe` blocks and functions (unsafe.block.1-3, ffi.call.1, ADRs
//! 0102, 0104, 0430): an unsafe operation is allowed only inside an `unsafe`
//! block, and only a package whose manifest allows it may contain one.

use super::{Checker, Ty};
use crate::diag::Pos;

impl<'p> Checker<'p> {
    /// unsafe.block.3: this position's package may contain `unsafe` code.
    pub(super) fn unsafe_opt_in(&mut self, pos: Pos) {
        if self.prog.unsafe_ok_at(pos) {
            return;
        }
        let who = match self.prog.package_at(pos) {
            "" => "this package".to_string(),
            p => format!("dependency `{p}`"),
        };
        self.error_help(
            "unsafe.block.3",
            pos,
            format!("{who} contains `unsafe` code, which its manifest doesn't allow"),
            "add `unsafe allow` to the `package` line (or the `dep` line) of `ail.pkg`, or remove the `unsafe` code",
        );
    }

    /// unsafe.block.2, ffi.call.1: calling an `unsafe fn` or a C function is
    /// an unsafe operation (unsafe.op.1).
    pub(super) fn note_unsafe_call(&mut self, f: usize, pos: Pos) {
        let decl = &self.prog.fns[f];
        if self.unsafe_depth > 0 || !(decl.unsafe_fn || decl.ext.is_some()) {
            return;
        }
        let (rule, what) = if decl.ext.is_some() {
            ("ffi.call.1", format!("`{}` is a C function, and calling one", decl.name))
        } else {
            ("unsafe.block.2", format!("`{}` is an `unsafe fn`, and calling one", decl.name))
        };
        self.error_help(
            rule,
            pos,
            format!("{what} is an unsafe operation, allowed only inside an `unsafe` block"),
            "wrap the call in `unsafe`, ideally inside a safe AIL function that checks the preconditions",
        );
    }
}

impl<'p> Checker<'p> {
    /// unsafe.ptr.1: a raw pointer points to a type with a fixed C layout:
    /// a number, a `bool` or another pointer.
    pub(super) fn ptr_target(&mut self, inner: &Ty, pos: Pos) {
        let ok = matches!(inner, Ty::Int | Ty::Float | Ty::Bool | Ty::Num(_) | Ty::Ptr(_) | Ty::Err);
        if !ok {
            let shown = self.show(inner);
            self.error_help(
                "unsafe.ptr.1",
                pos,
                format!("a raw pointer can't point to {shown}, which has no fixed C layout"),
                "point to `int`, `float`, `bool`, a sized number such as `u8`, or another pointer",
            );
        }
    }
}

/// The widest C call the bootstrap's trampolines make (ffi.decl.3): six
/// arguments in integer registers and eight in floating-point registers.
pub const MAX_INT_ARGS: usize = 6;
pub const MAX_FLOAT_ARGS: usize = 8;

impl<'p> Checker<'p> {
    /// ffi.decl.3: an `extern c fn` has only types with a C layout
    /// (`int`, `float`, `bool`, sized numbers, raw pointers; no result, or one of
    /// these), no `mut` parameters, and fits the registers of a C call.
    pub(super) fn check_extern(&mut self, idx: usize, f: &'p crate::ast::FnDecl) {
        let sig = self.sigs[idx].clone();
        let c_type = |t: &Ty| matches!(t, Ty::Int | Ty::Float | Ty::Bool | Ty::Num(_) | Ty::Ptr(_) | Ty::Err);
        let (mut ints, mut floats) = (0, 0);
        for (p, (_, t)) in f.params.iter().zip(&sig.params) {
            if p.mutable {
                self.error_help(
                    "ffi.decl.3",
                    p.pos,
                    "an `extern c fn` parameter is passed by value, not `mut`",
                    "pass a raw pointer `*T` to let C write to memory",
                );
            }
            if !c_type(t) {
                let shown = self.show(t);
                self.error_help(
                    "ffi.decl.3",
                    p.ty.pos,
                    format!("{shown} has no C layout, so it can't be a parameter of an `extern c fn`"),
                    "use `int`, `float`, `bool`, a sized number such as `i32` or `u8`, or a raw pointer `*T`",
                );
            }
            if matches!(t, Ty::Float | Ty::Num(crate::num::Num::F32)) { floats += 1 } else { ints += 1 }
        }
        if ints > MAX_INT_ARGS || floats > MAX_FLOAT_ARGS {
            self.error_help(
                "ffi.decl.3",
                f.pos,
                format!("`{}` takes {ints} integer-like and {floats} float arguments; a C call takes at most {MAX_INT_ARGS} and {MAX_FLOAT_ARGS}", f.name),
                "declare a smaller wrapper, or pass a pointer to a struct-like buffer",
            );
        }
        if !c_type(&sig.ret) && sig.ret != Ty::Unit {
            let shown = self.show(&sig.ret);
            let at = f.ret.as_ref().map_or(f.pos, |t| t.pos);
            self.error_help(
                "ffi.decl.3",
                at,
                format!("{shown} has no C layout, so it can't be the result of an `extern c fn`"),
                "return `int`, `float`, `bool`, a sized number or a raw pointer, or nothing",
            );
        }
    }
}

impl<'p> Checker<'p> {
    /// unsafe.op.1: `what` is an unsafe operation (raw memory, a pointer
    /// access), allowed only inside an `unsafe` block.
    pub(super) fn note_unsafe_op(&mut self, what: &str, pos: Pos) {
        if self.unsafe_depth > 0 {
            return;
        }
        self.error_help(
            "unsafe.op.1",
            pos,
            format!("{what} is an unsafe operation, allowed only inside an `unsafe` block"),
            "wrap it in `unsafe`, ideally inside a safe AIL function that checks the preconditions",
        );
    }
}

/// The methods of a raw pointer `*T` (unsafe.ptr.2-3), sorted.
pub const PTR_METHODS: &[&str] = &["is_null", "offset", "read", "write"];

impl<'p> Checker<'p> {
    /// The memory layout of what `t` points to (unsafe.ptr.1).
    fn ptr_elem(t: &Ty) -> Option<crate::stdlib::PtrElem> {
        use crate::stdlib::PtrElem;
        Some(match t {
            Ty::Int => PtrElem::Int,
            Ty::Float => PtrElem::Float,
            Ty::Bool => PtrElem::Bool,
            Ty::Num(k) => PtrElem::Num(*k),
            Ty::Ptr(_) => PtrElem::Ptr,
            _ => return None,
        })
    }

    /// `p.read()`, `p.write(v)`, `p.offset(n)` and `p.is_null()` on a raw pointer
    /// `*T` (unsafe.ptr.2-3); `None` for any other receiver.
    pub(super) fn ptr_method(
        &mut self,
        rt: &Ty,
        name: &str,
        pos: Pos,
        args: &'p [crate::ast::Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let Ty::Ptr(target) = rt else { return None };
        let Some(&m) = PTR_METHODS.iter().find(|m| **m == name) else {
            self.args_only(args);
            self.error_help(
                "unsafe.ptr.3",
                pos,
                format!("a raw pointer has no method `{name}`"),
                format!("its methods are {}", super::list_names(PTR_METHODS)),
            );
            return Some(Ty::Err);
        };
        let Some(elem) = Self::ptr_elem(target) else {
            self.args_only(args);
            return Some(Ty::Err);
        };
        self.resolved.std_calls.insert(pos, crate::stdlib::StdCall::Ptr(m, elem));
        if m != "is_null" {
            self.note_unsafe_op(&format!("`{m}` on a raw pointer"), pos);
        }
        let (params, ret) = match m {
            "read" => (vec![], (**target).clone()),
            "write" => (vec![(**target).clone()], Ty::Unit),
            "offset" => (vec![Ty::Int], rt.clone()),
            _ => (vec![], Ty::Bool),
        };
        let sig = super::Sig::plain(None, params.into_iter().map(|t| (false, t)).collect(), ret);
        Some(self.check_call(&format!("ptr.{m}"), &sig, Vec::new(), args, None, pos, expect))
    }

    /// `mem.alloc(n, example)`, `mem.cast(p, example)` and `mem.free(p)`: the
    /// pointee type comes from an example value or from the pointer
    /// (unsafe.mem.3). `None` for the other `std.mem` functions.
    pub(super) fn mem_generic(
        &mut self,
        f: &str,
        args: &'p [crate::ast::Arg],
        pos: Pos,
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        if !matches!(f, "alloc" | "cast" | "free") {
            return None;
        }
        let want = if f == "free" { 1 } else { 2 };
        let ptr_first = f != "alloc";
        let mut tys = Vec::new();
        for a in args {
            tys.push(self.quiet_type(&a.value));
        }
        if args.len() != want {
            let sig = super::Sig::plain(None, vec![(false, Ty::Err); want], Ty::Err);
            return Some(self.check_call(&format!("mem.{f}"), &sig, Vec::new(), args, None, pos, expect));
        }
        let mut params = Vec::new();
        if ptr_first {
            if !matches!(tys[0], Ty::Ptr(_) | Ty::Err) {
                let shown = self.show(&tys[0]);
                self.error_help(
                    "unsafe.ptr.1",
                    args[0].value.pos(),
                    format!("`mem.{f}` takes a raw pointer, but this is {shown}"),
                    "a raw pointer is `*T`, such as the result of `mem.alloc` or `mem.cstr`",
                );
                tys[0] = Ty::Err;
            }
            params.push(tys[0].clone());
        } else {
            params.push(Ty::Int);
        }
        let ret = if f == "free" {
            Ty::Unit
        } else {
            let example = tys[1].clone();
            self.ptr_target(&example, args[1].value.pos());
            params.push(example.clone());
            if example == Ty::Err { Ty::Err } else { Ty::Ptr(Box::new(example)) }
        };
        let sig = super::Sig::plain(None, params.into_iter().map(|t| (false, t)).collect(), ret);
        Some(self.check_call(&format!("mem.{f}"), &sig, Vec::new(), args, None, pos, expect))
    }
}
