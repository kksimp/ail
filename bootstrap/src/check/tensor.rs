//! `std.tensor` (spec 17-std, std.tensor.1-9, ADR 0328): the types of
//! `tensor.from(xs, shape)`, `tensor.zeros(shape)` and the methods of a
//! `tensor.Tensor[T]`, where `T` is `int` or `float`. A tensor is a value
//! whose elements live in the bootstrap (`crate::tensor`), so it has no
//! fields and can't be built with a struct literal (std.tensor.1).

use super::{Checker, Def, Kind, Local, Sig, Ty, list_names, suggest};
use crate::ast::{Arg, Expr};
use crate::diag::{Diagnostic, Pos};
use crate::stdlib::{StdCall, TENSOR_METHODS};

impl<'p> Checker<'p> {
    fn tensor_ty(&self, elem: Ty) -> Ty {
        match self.types.get("tensor.Tensor") {
            Some(Def::Struct(s)) => Ty::Struct(*s, vec![elem]),
            _ => unreachable!("std/types.ail declares `Tensor`"),
        }
    }

    /// The element type of `t` when it is a `tensor.Tensor[T]`.
    fn tensor_elem(&self, t: &Ty) -> Option<Ty> {
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
        let wanted = expect.and_then(|t| self.tensor_elem(t)).filter(|e| matches!(e, Ty::Int | Ty::Float));
        match f {
            "zeros" => {
                let elem = wanted.unwrap_or(Ty::Float);
                if elem == Ty::Int {
                    self.resolved.int_tensors.insert(pos);
                }
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
                if elem == Ty::Int {
                    self.resolved.int_tensors.insert(pos);
                }
                let mut params = vec![(false, Ty::List(Box::new(elem.clone())))];
                if args.len() != 1 {
                    params.push((false, shape));
                }
                let sig = Sig::plain(None, params, self.tensor_ty(elem));
                self.check_call("tensor.from", &sig, Vec::new(), args, None, pos, expect)
            }
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
        let t = rt.clone();
        let (params, ret) = match m {
            "shape" => (vec![], Ty::List(Box::new(Ty::Int))),
            "len" => (vec![], Ty::Int),
            "to_list" => (vec![], Ty::List(Box::new(elem))),
            "sum" => (vec![], elem),
            "dot" => (vec![t], elem),
            "add" | "mul" | "matmul" => (vec![t.clone()], t),
            "reshape" => (vec![Ty::List(Box::new(Ty::Int))], t),
            _ => unreachable!("TENSOR_METHODS lists `{m}`"),
        };
        let sig = Sig::plain(Some(false), params.into_iter().map(|t| (false, t)).collect(), ret);
        Some(self.check_call(&format!("tensor.Tensor.{m}"), &sig, Vec::new(), args, Some(recv), pos, expect))
    }

    /// std.tensor.8: `t.map(|x| e)`: `x` is each element, and `e` is an
    /// `int` or `float`, the new tensor's element type.
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
            self.error_help("std.tensor.8", a.value.pos(), "`map` takes a function of each element", "write `t.map(|x| x * 2.0)`");
            return rt.clone();
        };
        let mark = self.locals.len();
        if self.declare(var, *vpos) {
            self.locals.push(Local { name: var.clone(), ty: elem.clone(), mutable: false, kind: Kind::LoopVar, decl: None });
        }
        let out = self.expr(&a.value, Some(elem), true);
        self.end_scope(mark);
        match out {
            Ty::Int | Ty::Float => self.tensor_ty(out),
            Ty::Err => Ty::Err,
            other => {
                let msg = format!("`map`'s function gives {}, but a tensor holds `int` or `float`", self.show(&other));
                self.error_help("std.tensor.8", a.value.pos(), msg, "return a number from the function");
                Ty::Err
            }
        }
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
