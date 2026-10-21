//! `std.tensor` (spec 17-std, std.tensor.1-9, ADR 0328): the types of
//! `tensor.from(xs, shape)`, `tensor.zeros(shape)` and the methods of a
//! `tensor.Tensor[T]`, where `T` is `int` or `float`. A tensor is a value
//! whose elements live in the bootstrap (`crate::tensor`), so it has no
//! fields and can't be built with a struct literal (std.tensor.1).

use super::{Checker, Def, Kind, Local, Sig, Ty, list_names, suggest};
use crate::ast::{Arg, Expr};
use crate::diag::{Diagnostic, Pos};
use crate::num::Num;
use crate::stdlib::{StdCall, TENSOR_METHODS};

impl<'p> Checker<'p> {
    fn tensor_ty(&self, elem: Ty) -> Ty {
        match self.types.get("tensor.Tensor") {
            Some(Def::Struct(s)) => Ty::Struct(*s, vec![elem]),
            _ => unreachable!("std/types.ail declares `Tensor`"),
        }
    }

    /// The element type of `t` when it is a `tensor.Tensor[T]`.
    pub(super) fn tensor_elem(&self, t: &Ty) -> Option<Ty> {
        match t {
            Ty::Struct(s, args) if self.prog.structs[*s].name == "tensor.Tensor" => {
                Some(args.first().cloned().unwrap_or(Ty::Err))
            }
            _ => None,
        }
    }

    /// `tensor.f(args)` (std.tensor.2).
    pub(super) fn tensor_call(&mut self, f: &str, args: &'p [Arg], pos: Pos, expect: Option<&Ty>) -> Ty {
        let shape = Ty::List(Box::new(Ty::Int));
        let wanted =
            expect.and_then(|t| self.tensor_elem(t)).filter(|e| matches!(e, Ty::Int | Ty::Float | Ty::Num(Num::F32)));
        match f {
            "zeros" => {
                let elem = wanted.unwrap_or(Ty::Float);
                self.note_elem(&elem, pos);
                let sig = Sig::plain(None, vec![(false, shape)], self.tensor_ty(elem.clone()));
                self.check_call("tensor.zeros", &sig, Vec::new(), args, None, pos, expect)
            }
            _ => {
                // `from(xs)` is 1-D; `from(xs, shape)` gives the shape.
                let elem = match (wanted, args.first()) {
                    (Some(e), _) => e,
                    (None, Some(a)) => match self.quiet_type(&a.value) {
                        Ty::List(e) if matches!(*e, Ty::Int | Ty::Float) => *e,
                        Ty::List(e) if *e == Ty::Err => Ty::Float,
                        Ty::Err => Ty::Float,
                        other => {
                            self.args_only(args);
                            let msg = format!("a tensor holds `int` or `float` elements, not {}", self.show(&other));
                            self.error_help(
                                "std.tensor.1",
                                a.value.pos(),
                                msg,
                                "pass a `List[int]` or `List[float]`, e.g. `tensor.from([1.0, 2.0])`",
                            );
                            return Ty::Err;
                        }
                    },
                    (None, None) => Ty::Float,
                };
                self.note_elem(&elem, pos);
                // `from` takes a `List[float]` even for `Tensor[f32]` (std.tensor.21).
                let list = if elem == Ty::Num(Num::F32) { Ty::Float } else { elem.clone() };
                let mut params = vec![(false, Ty::List(Box::new(list)))];
                if args.len() != 1 {
                    params.push((false, shape));
                }
                let sig = Sig::plain(None, params, self.tensor_ty(elem));
                self.check_call("tensor.from", &sig, Vec::new(), args, None, pos, expect)
            }
        }
    }

    /// Records the element type of a tensor built at `pos` (std.tensor.21).
    fn note_elem(&mut self, elem: &Ty, pos: Pos) {
        match elem {
            Ty::Int => {
                self.resolved.int_tensors.insert(pos);
            }
            Ty::Num(Num::F32) => {
                self.resolved.f32_tensors.insert(pos);
            }
            _ => {}
        }
    }

    /// `t.name(args)` where `t` is a tensor (std.tensor.3-8); `None` for
    /// any other receiver.
    pub(super) fn tensor_method(
        &mut self,
        recv: &'p Expr,
        rt: &Ty,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let elem = self.tensor_elem(rt)?;
        let Some(&m) = TENSOR_METHODS.iter().find(|m| **m == name) else {
            self.args_only(args);
            let mut d = Diagnostic::new("std.tensor.3", pos, format!("`tensor.Tensor` has no method `{name}`"));
            d = match suggest(name, TENSOR_METHODS) {
                Some(s) => d.help(format!("did you mean `{s}`?")),
                None => d.help(format!("a tensor's methods are {}", list_names(TENSOR_METHODS))),
            };
            self.diags.push(d);
            return Some(Ty::Err);
        };
        self.resolved.std_calls.insert(pos, StdCall::Tensor(m));
        if m == "map" {
            return Some(self.tensor_map(rt, &elem, args, pos));
        }
        if matches!(m, "to_f32" | "to_float") {
            return Some(self.tensor_convert(m, &elem, args, pos));
        }
        if m == "mean" && elem == Ty::Int {
            self.args_only(args);
            self.error_help(
                "std.tensor.11",
                pos,
                "`mean` is on `float` tensors, and this one holds `int`",
                "convert first: `t.map(|x| x.to_float()).mean()`",
            );
            return Some(Ty::Err);
        }
        if m == "round" && elem == Ty::Int {
            self.args_only(args);
            self.error_help(
                "std.tensor.19",
                pos,
                "`round` is on `float` tensors, and this one holds `int`",
                "a tensor of `int` already has whole numbers",
            );
            return Some(Ty::Err);
        }
        if m == "clamp" && elem == Ty::Int {
            self.args_only(args);
            self.error_help(
                "std.tensor.18",
                pos,
                "`clamp` is on `float` tensors, and this one holds `int`",
                "convert first: `t.map(|x| x.to_float()).clamp(lo, hi)`",
            );
            return Some(Ty::Err);
        }
        if m == "conv2d" && elem == Ty::Int {
            self.args_only(args);
            self.error_help(
                "std.tensor.16",
                pos,
                "`conv2d` is on `float` tensors, and this one holds `int`",
                "convert first: `t.map(|x| x.to_float()).conv2d(k)`",
            );
            return Some(Ty::Err);
        }
        if matches!(m, "relu" | "sigmoid" | "tanh") && elem == Ty::Int {
            self.args_only(args);
            let msg = format!("`{m}` is on `float` tensors, and this one holds `int`");
            self.error_help("std.tensor.13", pos, msg, format!("convert first: `t.map(|x| x.to_float()).{m}()`"));
            return Some(Ty::Err);
        }
        let t = rt.clone();
        // std.tensor.21: an `f32` tensor's bounds and list are `float`.
        let (bound, list) =
            if elem == Ty::Num(Num::F32) { (Ty::Float, Ty::Float) } else { (elem.clone(), elem.clone()) };
        let (params, ret) = match m {
            "shape" => (vec![], Ty::List(Box::new(Ty::Int))),
            "len" => (vec![], Ty::Int),
            "to_list" => (vec![], Ty::List(Box::new(list))),
            "item" => (vec![], elem),
            // std.tensor.11: over every element, or over one axis.
            "sum" | "max" | "mean" if args.is_empty() => (vec![], elem),
            "sum" | "max" | "mean" => (vec![Ty::Int], t),
            "transpose" => (vec![], t),
            "round" => (vec![], self.tensor_ty(Ty::Int)),
            "argmax" | "argmin" if args.is_empty() => (vec![], Ty::Int),
            "argmax" | "argmin" => (vec![Ty::Int], self.tensor_ty(Ty::Int)),
            "clamp" => (vec![bound.clone(), bound], t),
            "dot" => (vec![t], elem),
            "add" | "mul" | "matmul" | "conv2d" => (vec![t.clone()], t),
            "relu" | "sigmoid" | "tanh" => (vec![], t),
            "reshape" => (vec![Ty::List(Box::new(Ty::Int))], t),
            _ => unreachable!("TENSOR_METHODS lists `{m}`"),
        };
        let sig = Sig::plain(Some(false), params.into_iter().map(|t| (false, t)).collect(), ret);
        Some(self.check_call(&format!("tensor.Tensor.{m}"), &sig, Vec::new(), args, Some(recv), pos, expect))
    }

    /// std.tensor.22: `t.to_f32()` on a `float` tensor and `t.to_float()` on
    /// an `f32` one; every other conversion is an error.
    fn tensor_convert(&mut self, m: &str, elem: &Ty, args: &'p [Arg], pos: Pos) -> Ty {
        self.args_only(args);
        let (from, to) = if m == "to_f32" { (Ty::Float, Ty::Num(Num::F32)) } else { (Ty::Num(Num::F32), Ty::Float) };
        if *elem != from {
            let msg = format!("`{m}` converts a {} tensor, but this one holds {}", self.show(&from), self.show(elem));
            self.error_help(
                "std.tensor.22",
                pos,
                msg,
                "`to_f32()` takes `float` tensors and `to_float()` takes `f32` ones",
            );
            return Ty::Err;
        }
        self.tensor_ty(to)
    }

    /// std.tensor.8: `t.map(|x| e)`: `x` is each element, and `e` is an
    /// `int`, `float` or `f32`, the new tensor's element type.
    fn tensor_map(&mut self, rt: &Ty, elem: &Ty, args: &'p [Arg], pos: Pos) -> Ty {
        let [a] = args else {
            self.args_only(args);
            self.error_help(
                "std.tensor.8",
                pos,
                format!("`map` takes one function, but {} given", super::was_were(args.len())),
                "write `t.map(|x| x * 2.0)`",
            );
            return rt.clone();
        };
        let Some((var, vpos)) = &a.param else {
            self.expr(&a.value, None, true);
            self.error_help(
                "std.tensor.8",
                a.value.pos(),
                "`map` takes a function of each element",
                "write `t.map(|x| x * 2.0)`",
            );
            return rt.clone();
        };
        let mark = self.locals.len();
        if self.declare(var, *vpos) {
            self.locals.push(Local {
                name: var.clone(),
                ty: elem.clone(),
                mutable: false,
                kind: Kind::LoopVar,
                decl: None,
            });
        }
        let out = self.expr(&a.value, Some(elem), true);
        self.end_scope(mark);
        match out {
            Ty::Int | Ty::Float | Ty::Num(Num::F32) => self.tensor_ty(out),
            Ty::Err => Ty::Err,
            other => {
                let msg = format!("`map`'s function gives {}, but a tensor holds `int` or `float`", self.show(&other));
                self.error_help("std.tensor.8", a.value.pos(), msg, "return a number from the function");
                Ty::Err
            }
        }
    }

    /// type.num.10 for std.tensor.10: an `int` literal `scalar` beside a
    /// `Tensor[float]` `t` is a `float`.
    pub(super) fn float_tensor_scalar(&mut self, t: &Ty, st: &Ty, scalar: &Expr) -> bool {
        *st == Ty::Int && self.tensor_elem(t) == Some(Ty::Float) && self.float_literal(scalar)
    }

    /// std.tensor.10: `+`, `-` and `*` with a tensor operand: two tensors
    /// of one element type, or a tensor and a scalar of its element type,
    /// either side. `None` when neither operand is a tensor.
    pub(super) fn tensor_operator(&mut self, sym: &str, arith: bool, lt: &Ty, rt: &Ty, pos: Pos) -> Option<Ty> {
        let (le, re) = (self.tensor_elem(lt), self.tensor_elem(rt));
        if le.is_none() && re.is_none() {
            return None;
        }
        if !arith || !matches!(sym, "+" | "-" | "*" | "/") {
            if arith {
                self.error_help(
                    "std.tensor.10",
                    pos,
                    format!("`{sym}` is not defined for tensors"),
                    "use `t.map(|x| ...)`",
                );
                return Some(Ty::Err);
            }
            return None;
        }
        // std.tensor.21: `f32` and `float` tensors don't mix.
        if let (Some(a), Some(b)) = (&le, &re)
            && (*a == Ty::Num(Num::F32)) != (*b == Ty::Num(Num::F32))
            && matches!(a, Ty::Float | Ty::Num(Num::F32))
            && matches!(b, Ty::Float | Ty::Num(Num::F32))
        {
            let msg =
                format!("`{sym}` needs tensors of one element type, but found {} and {}", self.show(lt), self.show(rt));
            self.error_help("std.tensor.21", pos, msg, "convert one with `t.to_f32()` or `t.to_float()`");
            return Some(Ty::Err);
        }
        // A `float` scalar takes an `f32` tensor's element type (std.tensor.21).
        let scalar_is = |elem: &Ty, s: &Ty| elem == s || (*elem == Ty::Num(Num::F32) && *s == Ty::Float);
        let ok = match (&le, &re) {
            (Some(a), Some(b)) => a == b,
            (Some(e), None) => scalar_is(e, rt),
            (None, Some(e)) => scalar_is(e, lt),
            (None, None) => false,
        };
        if ok && sym == "/" && le.as_ref().or(re.as_ref()) == Some(&Ty::Int) {
            self.error_help(
                "std.tensor.10",
                pos,
                "`/` is not defined for `int` tensors",
                "convert first: `t.map(|x| x.to_float())`",
            );
            return Some(Ty::Err);
        }
        if ok {
            return Some(if le.is_some() { lt.clone() } else { rt.clone() });
        }
        let msg = format!(
            "`{sym}` needs two tensors of one element type, or a tensor and a scalar of its element type, found {} and {}",
            self.show(lt),
            self.show(rt)
        );
        let other = if le.is_some() { rt } else { lt };
        let help = match (&le, &re) {
            (Some(_), Some(_)) => "convert one with `t.map(|x| x.to_float())`",
            _ if matches!(other, Ty::Int | Ty::Float) => {
                "convert the number to the element type: `x.to_float()` or `x.to_int()`"
            }
            _ => "a tensor combines with a tensor or a number of its element type",
        };
        self.error_help("std.tensor.10", pos, msg, help);
        Some(Ty::Err)
    }

    /// std.tensor.1: a tensor has no fields; `None` for other receivers.
    pub(super) fn tensor_field(&mut self, rt: &Ty, name: &str, pos: Pos) -> Option<Ty> {
        self.tensor_elem(rt)?;
        self.error_help(
            "std.tensor.1",
            pos,
            format!("a tensor has no field `{name}`; its elements live in the runtime"),
            format!("read it with a method: {}", list_names(TENSOR_METHODS)),
        );
        Some(Ty::Err)
    }
}
