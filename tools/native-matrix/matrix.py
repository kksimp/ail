#!/usr/bin/env python3
"""The check-means-build matrix (ADR 1109): every built-in method of every std
collection and prelude type, on every element type, as small native programs.

A snippet the checker accepts must build natively and run. The script writes
one program per (container, element type) group, each snippet in its own
function, then:

  1. `check-program` on each group; snippets with checker errors are dropped
     (the checker refused them, so check and build agree) and the group is
     checked again until it is clean;
  2. `build` on the clean group: every `native.subset.1` (or other build error)
     is a HOLE, named by the snippet whose lines it falls in;
  3. the built program is run; a non-zero exit is a RUN failure.

Usage:
  matrix.py emit DIR                 write the group programs to DIR
  matrix.py run AIL_BIN [--keep DIR] [--freeze DIR]
                                     check, build and run every group with the
                                     stage-2 binary AIL_BIN (AIL_ROOT and
                                     AIL_COMPILER set by the caller); --freeze
                                     writes each group that built and ran as
                                     DIR/NAME.ail and its output as NAME.out,
                                     the gate's `native_matrix` suite
                                     (tests/native_matrix/)

Measurement only: it edits no source.
"""
import os
import re
import subprocess
import sys
import tempfile

HEADER = """struct P
    name str
    n int

struct Q
    n int
    name str

impl Ord for Q

enum Color
    Red
    Green
    Blue

enum Shape
    Circle(r int)
    Square(s int)

type Uid = int

fn oi(n int) int?
    if n < 0
        return none
    n
"""

# name: (type, a, b, c, key-expression on x, flags)
# flags: o = ordered (`<`), n = number, h = hashable (map key, set element)
TYPES = {
    "int": ("int", "1", "2", "3", "x", "onh"),
    "float": ("float", "1.5", "2.5", "3.5", "x", "on"),
    "str": ("str", '"a"', '"b"', '"c"', "x", "oh"),
    "char": ("char", "'a'", "'b'", "'c'", "x", "oh"),
    "bool": ("bool", "false", "true", "true", "x", "oh"),
    "u8": ("u8", "1.to_u8()", "2.to_u8()", "3.to_u8()", "x", "onh"),
    "uid": ("Uid", "Uid(1)", "Uid(2)", "Uid(3)", "x", "oh"),
    "tuple": ("(int, str)", '(1, "a")', '(2, "b")', '(3, "c")', "x.0", "oh"),
    "struct": ("P", 'P(name: "a", n: 1)', 'P(name: "b", n: 2)', 'P(name: "c", n: 3)', "x.n", "h"),
    "ordstruct": ("Q", 'Q(n: 1, name: "a")', 'Q(n: 2, name: "b")', 'Q(n: 3, name: "c")', "x.n", "oh"),
    "enum": ("Color", "Color.Red", "Color.Green", "Color.Blue", "1", "h"),
    "payenum": ("Shape", "Shape.Circle(r: 1)", "Shape.Square(s: 2)", "Shape.Circle(r: 3)", "1", "h"),
    "opt": ("int?", "oi(1)", "oi(2)", "oi(-1)", "(x else 0)", "h"),
    "list": ("List[int]", "[1]", "[1, 2]", "[2]", "x.len()", "oh"),
    "map": ("Map[str, int]", '{"a": 1}', '{"b": 2}', '{"c": 3}', "x.len()", ""),
    "set": ("Set[int]", "{1}", "{2}", "{3}", "x.len()", ""),
    "bytes": ("bytes", 'b"a"', 'b"b"', 'b"c"', "x.len()", "oh"),
}

# Each template is a function body. @T type, @A @B @C values, @K a key of `x`.
# A line `!o` (or `!n`, `!h`) first means the snippet only makes sense with that flag;
# the generator still emits the others (the checker should refuse them), but they are
# marked so the report can tell an expected refusal from a surprise.
LIST = {
    "literal": "xs := [@A, @B, @C]\nprint(xs.len())",
    "empty": "xs := List[@T]()\nprint(xs.len())",
    "typed_by_push": "mut xs := []\nxs.push(@A)\nprint(xs.len())",
    "len": "xs := [@A, @B]\nprint(xs.len())",
    "is_empty": "xs := [@A]\nprint(xs.is_empty())",
    "index": "xs := [@A, @B]\nx := xs[1]\nprint(@K == @K)",
    "index_assign": "mut xs := [@A, @B]\nxs[0] = @C\nprint(xs.len())",
    "slice": "xs := [@A, @B, @C]\nprint(xs[1..].len() + xs[..1].len() + xs[0..2].len())",
    "slice_method": "xs := [@A, @B, @C]\nprint(xs.slice(1).len() + xs.slice(0, 2).len())",
    "push": "mut xs := [@A]\nxs.push(@B)\nprint(xs.len())",
    "pop": "mut xs := [@A, @B]\nif x := xs.pop()\n    print(xs.len())",
    "swap": "mut xs := [@A, @B]\nxs.swap(0, 1)\nprint(xs.len())",
    "first_last": "xs := [@A, @B]\nif x := xs.first()\n    print(1)\nif x := xs.last()\n    print(2)",
    "get": "xs := [@A, @B]\nif x := xs.get(1)\n    print(1)\nif x := xs.get(2)\n    print(2)",
    "concat": "xs := [@A]\nys := [@B, @C]\nprint(xs.concat(ys).len() + xs.len())",
    "insert": "mut xs := [@A]\nxs.insert(0, @B)\nprint(xs.len())",
    "remove_at": "mut xs := [@A, @B]\nx := xs.remove_at(0)\nprint(xs.len())",
    "extend": "mut xs := [@A]\nxs.extend([@B, @C])\nprint(xs.len())",
    "clear": "mut xs := [@A]\nxs.clear()\nprint(xs.len())",
    "reverse": "mut xs := [@A, @B]\nxs.reverse()\nprint(xs.len())",
    "reversed": "xs := [@A, @B]\nprint(xs.reversed().len())",
    "contains": "xs := [@A, @B]\nprint(xs.contains(@B))",
    "index_of": "xs := [@A, @B]\nprint(xs.index_of(@B) else -1)",
    "join": "xs := [@A, @B]\nprint(xs.join(\",\").len() > 0)",
    "sum": "xs := [@A, @B]\ns := xs.sum()\nprint(1)",
    "min_max": "xs := [@B, @A, @C]\nif x := xs.min()\n    print(1)\nif x := xs.max()\n    print(2)",
    "sorted": "xs := [@C, @A, @B]\nprint(xs.sorted().len())",
    "sort": "mut xs := [@C, @A, @B]\nxs.sort()\nprint(xs.len())",
    "sorted_by": "xs := [@C, @A, @B]\nprint(xs.sorted(by: |x| @K).len())",
    "sort_by": "mut xs := [@C, @A, @B]\nxs.sort(by: |x| @K)\nprint(xs.len())",
    "min_by": "xs := [@C, @A, @B]\nif x := xs.min(by: |x| @K)\n    print(1)\nif x := xs.max(by: |x| @K)\n    print(2)",
    "take_skip": "xs := [@A, @B, @C]\nprint(xs.take(2).len() + xs.skip(1).len())",
    "set_ops": "xs := [@A, @B]\nys := [@B, @C]\nprint(xs.union(ys).len() + xs.intersect(ys).len() + xs.difference(ys).len())",
    "chunks_windows": "xs := [@A, @B, @C]\nprint(xs.chunks(2).len() + xs.windows(2).len())",
    "counts": "xs := [@A, @B, @A]\nc := xs.counts()\nprint(c.len())",
    "flatten": "xs := [[@A], [@B, @C]]\nprint(xs.flatten().len())",
    "enumerate": "xs := [@A, @B]\nfor i, x in xs.enumerate()\n    print(i)",
    "any_all": "xs := [@A, @B]\nprint(xs.any(|x| x == @A))\nprint(xs.all(|x| x == @A))",
    "find": "xs := [@A, @B]\nif x := xs.find(|x| x == @B)\n    print(1)",
    "filter": "xs := [@A, @B]\nprint(xs.filter(|x| x == @A).len())",
    "map": "xs := [@A, @B]\nprint(xs.map(|x| @K).len())",
    "map_to_self": "xs := [@A, @B]\nprint(xs.map(|x| x).len())",
    "group_by": "xs := [@A, @B, @A]\ng := xs.group_by(|x| @K)\nprint(g.len())",
    "count": "xs := [@A, @B, @A]\nprint(xs.count(|x| x == @A))",
    "fold": "xs := [@A, @B]\nprint(xs.fold(0, |acc, x| acc + 1))",
    "flat_map": "xs := [@A, @B]\nprint(xs.flat_map(|x| [x, x]).len())",
    "take_skip_while": "xs := [@A, @B]\nprint(xs.take_while(|x| x == @A).len() + xs.skip_while(|x| x == @A).len())",
    "zip": "xs := [@A, @B]\nfor x, y in xs.zip([1, 2])\n    print(y)",
    "zip_into": "xs := [@A, @B]\nfor y, x in [1, 2].zip(xs)\n    print(y)",
    "for": "xs := [@A, @B]\nmut n := 0\nfor x in xs\n    n += 1\nprint(n)",
    "for_mut": "mut xs := [@A, @B]\nfor mut x in xs\n    x = @C\nprint(xs.len())",
    "comprehension": "xs := [@A, @B]\nys := [x for x in xs if x == @A]\nprint(ys.len())",
    "set_comprehension": "xs := [@A, @B, @A]\ns := {x for x in xs}\nprint(s.len())",
    "map_comprehension": "xs := [@A, @B]\nm := {x: 1 for x in xs}\nprint(m.len())",
    "eq": "xs := [@A, @B]\nys := [@A, @B]\nprint(xs == ys)\nprint(xs != [@C])",
    "lt": "xs := [@A]\nys := [@B]\nprint(xs < ys)",
    "print": "xs := [@A, @B]\nprint(xs)",
    "interp": "xs := [@A, @B]\nprint(\"{xs}\".len() > 0)",
    "nested_push": "mut xs := [[@A]]\nxs[0].push(@B)\nprint(xs[0].len())",
    "in_struct": "mut b := Box_@N(items: [@A])\nb.items.push(@B)\nprint(b.items.len())",
    "pass_mut": "mut xs := [@A]\nadd_@N(mut xs, @B)\nprint(xs.len())",
    "generic_first": "xs := [@A, @B]\nif x := first_of(xs)\n    print(1)",
    "generic_ord": "xs := [@A, @B]\nif x := biggest(xs)\n    print(1)",
    "contains_list": "xs := [[@A], [@B]]\nprint(xs.contains([@B]))",
    "sorted_nested": "xs := [[@B], [@A]]\nprint(xs.sorted().len())",
}

MAPK = {  # Map[@T, int]: the element type as the key
    "literal": "m := {@A: 1, @B: 2}\nprint(m.len())",
    "empty": "m := Map[@T, int]()\nprint(m.len())",
    "typed_by_use": "mut m := {}\nm[@A] += 1\nprint(m.len())",
    "index": "m := {@A: 1}\nprint(m[@A])",
    "assign": "mut m := Map[@T, int]()\nm[@A] = 1\nm[@B] = 2\nprint(m.len())",
    "op_assign": "mut m := Map[@T, int]()\nm[@A] += 3\nprint(m[@A])",
    "get": "m := {@A: 1}\nprint(m.get(@A) else 0)",
    "get_or": "m := {@A: 1}\nprint(m.get_or(@B, 7))",
    "contains": "m := {@A: 1}\nprint(m.contains(@A))",
    "remove": "mut m := {@A: 1, @B: 2}\nprint(m.remove(@A) else 0)",
    "keys_values": "m := {@A: 1, @B: 2}\nprint(m.keys().len() + m.values().len())",
    "keys_sorted": "m := {@B: 1, @A: 2}\nprint(m.keys().sorted().len())",
    "top": "m := {@A: 1, @B: 2}\nprint(m.top(1).len())",
    "for": "m := {@A: 1, @B: 2}\nmut n := 0\nfor k, v in m\n    n += v\nprint(n)",
    "any_all_count": "m := {@A: 1, @B: 2}\nprint(m.any(|k, v| v > 1))\nprint(m.all(|k, v| v > 1))\nprint(m.count(|k, v| v > 1))",
    "fold": "m := {@A: 1, @B: 2}\nprint(m.fold(0, |acc, k, v| acc + v))",
    "filter": "m := {@A: 1, @B: 2}\nprint(m.filter(|k, v| v > 1).len())",
    "map": "m := {@A: 1, @B: 2}\nprint(m.map(|k, v| v).len())",
    "flat_map": "m := {@A: 1, @B: 2}\nprint(m.flat_map(|k, v| [v, v]).len())",
    "map_values": "m := {@A: 1, @B: 2}\nprint(m.map_values(|k, v| v * 2).len())",
    "push": "mut m := Map[@T, List[int]]()\nm.push(@A, 1)\nm.push(@A, 2)\nprint(m[@A].len())",
    "eq": "m := {@A: 1}\nprint(m == {@A: 1})",
    "print": "m := {@A: 1}\nprint(m)",
    "is_empty": "m := {@A: 1}\nprint(m.is_empty())",
}

MAPV = {  # Map[str, @T]: the element type as the value
    "literal": "m := {\"a\": @A, \"b\": @B}\nprint(m.len())",
    "empty": "m := Map[str, @T]()\nprint(m.len())",
    "index": "m := {\"a\": @A}\nx := m[\"a\"]\nprint(@K == @K)",
    "assign": "mut m := Map[str, @T]()\nm[\"a\"] = @A\nprint(m.len())",
    "get": "m := {\"a\": @A}\nif x := m.get(\"a\")\n    print(1)",
    "get_else": "m := {\"a\": @A}\nx := m.get(\"z\") else @B\nprint(1)",
    "get_or": "m := {\"a\": @A}\nx := m.get_or(\"b\", @C)\nprint(1)",
    "remove": "mut m := {\"a\": @A}\nif x := m.remove(\"a\")\n    print(m.len())",
    "values": "m := {\"a\": @A, \"b\": @B}\nprint(m.values().len())",
    "values_sorted": "m := {\"a\": @B, \"b\": @A}\nprint(m.values().sorted().len())",
    "for": "m := {\"a\": @A, \"b\": @B}\nmut n := 0\nfor k, v in m\n    n += 1\nprint(n)",
    "map_values": "m := {\"a\": @A}\nprint(m.map_values(|k, v| v).len())",
    "filter": "m := {\"a\": @A, \"b\": @B}\nprint(m.filter(|k, v| v == @A).len())",
    "nested_update": "mut m := Map[str, List[@T]]()\nm[\"a\"] = [@A]\nm[\"a\"].push(@B)\nprint(m[\"a\"].len())",
    "eq": "m := {\"a\": @A}\nprint(m == {\"a\": @A})",
    "print": "m := {\"a\": @A}\nprint(m)",
}

SET = {
    "literal": "s := {@A, @B}\nprint(s.len())",
    "empty": "s := Set[@T]()\nprint(s.len())",
    "from_list": "s := {x for x in [@A, @B, @A]}\nprint(s.len())",
    "add_remove": "mut s := Set[@T]()\nprint(s.add(@A))\nprint(s.remove(@A))",
    "contains": "s := {@A, @B}\nprint(s.contains(@A))",
    "is_empty": "s := {@A}\nprint(s.is_empty())",
    "sorted": "s := {@B, @A}\nprint(s.sorted().len())",
    "to_list": "s := {@A, @B}\nprint(s.to_list().len())",
    "is_subset": "s := {@A}\nt := {@A, @B}\nprint(s.is_subset(t))",
    "set_ops": "s := {@A, @B}\nt := {@B, @C}\nprint(s.union(t).len() + s.intersect(t).len() + s.difference(t).len())",
    "any_all_find": "s := {@A, @B}\nprint(s.any(|x| x == @A))\nprint(s.all(|x| x == @A))\nif x := s.find(|x| x == @B)\n    print(1)",
    "filter_map": "s := {@A, @B}\nprint(s.filter(|x| x == @A).len() + s.map(|x| @K).len())",
    "group_by": "s := {@A, @B}\nprint(s.group_by(|x| @K).len())",
    "count_fold": "s := {@A, @B}\nprint(s.count(|x| x == @A))\nprint(s.fold(0, |acc, x| acc + 1))",
    "flat_map": "s := {@A, @B}\nprint(s.flat_map(|x| [x]).len())",
    "take_skip": "s := {@A, @B, @C}\nprint(s.take(1).len() + s.skip(1).len() + s.take_while(|x| x == @A).len() + s.skip_while(|x| x == @A).len())",
    "zip_enumerate": "s := {@A, @B}\nprint(s.zip([1, 2]).len() + s.enumerate().len())",
    "sum": "s := {@A, @B}\nx := s.sum()\nprint(1)",
    "min_max": "s := {@A, @B}\nif x := s.min()\n    print(1)\nif x := s.max()\n    print(2)",
    "for": "s := {@A, @B}\nmut n := 0\nfor x in s\n    n += 1\nprint(n)",
    "eq": "s := {@A, @B}\nprint(s == {@B, @A})",
    "print": "s := {@A}\nprint(s)",
    "in_map": "mut m := Map[str, Set[@T]]()\nm[\"a\"] = Set[@T]()\nm[\"a\"].add(@A)\nprint(m[\"a\"].len())",
    "in_list": "mut xs := [Set[@T]()]\nxs[0].add(@A)\nprint(xs[0].len())",
}

HEAP = {
    "empty_push_pop": "mut h := Heap[@T]()\nh.push(@B)\nh.push(@A)\nif x := h.pop()\n    print(h.len())",
    "max_first": "mut h := Heap[@T](max_first: true)\nh.push(@A)\nh.push(@B)\nif x := h.peek()\n    print(h.len())",
    "from": "h := Heap.from([@C, @A, @B])\nprint(h.len())",
    "from_max": "h := Heap.from_max([@C, @A, @B])\nprint(h.to_sorted().len())",
    "is_empty": "h := Heap[@T]()\nprint(h.is_empty())",
    "drain": "mut h := Heap.from([@C, @A, @B])\nmut n := 0\nwhile x := h.pop()\n    n += 1\nprint(n)",
}

DEQUE = {
    "empty_push": "mut d := Deque[@T]()\nd.push_back(@A)\nd.push_front(@B)\nprint(d.len())",
    "from": "d := Deque.from([@A, @B])\nprint(d.len())",
    "pop": "mut d := Deque.from([@A, @B, @C])\nif x := d.pop_front()\n    print(1)\nif x := d.pop_back()\n    print(2)",
    "first_last": "d := Deque.from([@A, @B])\nif x := d.first()\n    print(1)\nif x := d.last()\n    print(2)",
    "index": "d := Deque.from([@A, @B])\nx := d[1]\nprint(@K == @K)",
    "at_set": "mut d := Deque.from([@A, @B])\nd.set(0, @C)\nx := d.at(0)\nprint(d.len())",
    "to_list": "d := Deque.from([@A, @B])\nprint(d.to_list().len())",
    "for": "d := Deque.from([@A, @B])\nmut n := 0\nfor x in d\n    n += 1\nprint(n)",
    "vocab": "d := Deque.from([@A, @B])\nprint(d.filter(|x| x == @A).len() + d.map(|x| @K).len() + d.flat_map(|x| [x]).len())\nprint(d.any(|x| x == @A))\nprint(d.all(|x| x == @A))\nprint(d.count(|x| x == @A))\nprint(d.fold(0, |acc, x| acc + 1))\nif x := d.find(|x| x == @A)\n    print(1)",
    "take_skip": "d := Deque.from([@A, @B, @C])\nprint(d.take(1).len() + d.skip(1).len() + d.take_while(|x| x == @A).len() + d.skip_while(|x| x == @A).len())",
    "enumerate_zip": "d := Deque.from([@A, @B])\nprint(d.enumerate().len() + d.zip([1, 2]).len())",
    "is_empty": "d := Deque[@T]()\nprint(d.is_empty())",
}

GRID = {
    "fill": "g := Grid.fill(2, 3, @A)\nprint(g.rows * g.cols)",
    "from": "g := Grid.from([[@A, @B], [@C, @A]])\nprint(g.cells.len())",
    "index": "g := Grid.fill(2, 2, @A)\nx := g[1, 1]\nprint(@K == @K)",
    "index_assign": "mut g := Grid.fill(2, 2, @A)\ng[0, 1] = @B\nprint(g.rows)",
    "row": "g := Grid.from([[@A, @B], [@C, @A]])\nprint(g.row(1).len())",
    "to_list": "g := Grid.fill(2, 2, @A)\nprint(g.to_list().len())",
    "cells_for": "g := Grid.fill(2, 2, @A)\nmut n := 0\nfor x in g.cells\n    n += 1\nprint(n)",
}

SEQ = {
    "of_collect": "s := Seq.of([@A, @B])\nprint(s.collect().len())",
    "map_filter": "s := Seq.of([@A, @B])\nprint(s.filter(|x| x == @A).map(|x| @K).collect().len())",
    "take_skip": "s := Seq.of([@A, @B, @C])\nprint(s.skip(1).take(1).collect().len())",
    "enumerate": "s := Seq.of([@A, @B])\nprint(s.enumerate().collect().len())",
    "zip_chain": "s := Seq.of([@A, @B])\nt := Seq.of([1, 2])\nprint(s.zip(move t).collect().len())",
    "chain": "s := Seq.of([@A])\nt := Seq.of([@B])\nprint(s.chain(move t).collect().len())",
    "next": "mut s := Seq.of([@A, @B])\nif x := s.next()\n    print(1)",
    "for": "s := Seq.of([@A, @B])\nmut n := 0\nfor x in s\n    n += 1\nprint(n)",
    "count_any": "s := Seq.of([@A, @B])\nprint(s.count(|x| x == @A))",
    "fold": "s := Seq.of([@A, @B])\nprint(s.fold(0, |acc, x| acc + 1))",
    "find": "s := Seq.of([@A, @B])\nif x := s.find(|x| x == @B)\n    print(1)",
    "flat_map": "s := Seq.of([@A, @B])\nprint(s.flat_map(|x| [x, x]).collect().len())",
    "while": "s := Seq.of([@A, @B])\nprint(s.take_while(|x| x == @A).collect().len())",
    "skip_while": "s := Seq.of([@A, @B])\nprint(s.skip_while(|x| x == @A).collect().len())",
}

VALUE = {  # the element type on its own
    "eq": "print(@A == @A)\nprint(@A != @B)",
    "lt": "print(@A < @B)\nprint(@B >= @A)",
    "print": "print(@A)",
    "interp": "x := @A\nprint(\"<{x}>\")",
    "to_str_len": "x := @A\nprint(\"{x}\".len() > 0)",
    "optional_else": "o := wrap_@N(@A)\nx := o else @B\nprint(1)",
    "optional_if": "o := wrap_@N(@A)\nif x := o\n    print(1)",
    "optional_none": "o := nothing_@N()\nprint(o == none)",
    "optional_print": "o := wrap_@N(@A)\nprint(o)",
    "tuple": "t := (@A, 1)\nprint(t.1)\na, b := t\nprint(b)",
    "tuple_eq": "t := (@A, 1)\nprint(t == (@A, 1))",
    "tuple_print": "t := (@A, 1)\nprint(t)",
    "tuple_list_sorted": "xs := [(@B, 1), (@A, 2)]\nprint(xs.sorted().len())",
    "tuple_key": "mut m := Map[(@T, int), int]()\nm[(@A, 1)] = 2\nprint(m[(@A, 1)])",
    "struct_field": "b := Box_@N(items: [@A])\nc := b\nprint(c == b)",
    "struct_print": "b := Box_@N(items: [@A])\nprint(b)",
    "ref": "r := ref Box_@N(items: [@A])\ns := r\ns.items.push(@B)\nprint(r.items.len())",
    "generic_id": "x := ident(@A)\nprint(1)",
    "generic_box": "g := GBox(v: @A)\nprint(g.v == @A)",
    "fn_value": "f := |x @T| x\ny := f(@A)\nprint(1)",
    "closure_capture": "a := @A\nf := || a\ny := f()\nprint(1)",
    "match_eq": "x := @A\nprint(if x == @A then 1 else 0)",
    "const_list": "print(K_@N.len())",
}

# Per element type declarations, each dropped (with the snippets naming it) if the checker refuses it.
PER_TYPE_DECLS = [
    ("Box_@N", "struct Box_@N\n    items List[@T]"),
    ("add_@N", "fn add_@N(mut xs List[@T], x @T)\n    xs.push(x)"),
    ("wrap_@N", "fn wrap_@N(x @T) @T?\n    x"),
    ("nothing_@N", "fn nothing_@N() @T?\n    none"),
    ("K_@N", "K_@N := [@A, @B]"),
]

GENERIC_DECLS = """fn first_of[T](xs List[T]) T?
    xs.first()

fn biggest[T Ord](xs List[T]) T?
    mut best := xs.first() else return none
    for x in xs
        if x > best
            best = x
    best

fn ident[T](x T) T
    x

struct GBox[T]
    v T
"""

# Methods of the scalar types: one snippet each.
SCALAR = {
    "str": [
        'print("a,b".split(","))', 'print("12".to_int() else 0)', 'print("1.5".to_float() else 0.0)',
        'print("ab".chars())', 'print(" a ".trim())', 'print("abc".slice(1, 2))', 'print("Ab".to_lower())',
        'print("ab".to_upper())', 'print("abc".find("b") else -1)', 'print("abc".replace("b", "x"))',
        'print("abc".char_at(1) else \'z\')', 'print("abc".starts_with("a"))', 'print("abc".ends_with("c"))',
        'print("abc".strip_prefix("a") else "")', 'print("abc".strip_suffix("c") else "")',
        'print("abc".contains("b"))', 'print("ab".repeat(2))', 'print("a".pad_left(3, \'.\'))',
        'print("a".pad_right(3, \'.\'))', 'print("a".center(3, \'.\'))', 'print("a b".words())',
        'print(" a".trim_start())', 'print("a ".trim_end())', 'print("aaa".count("a"))',
        'print("aba".rfind("a") else -1)', 'print("a\\nb".lines())', 'print("ab".len())',
        'print("".is_empty())', 'print("é".byte_len())', 'print("ab".to_bytes().len())',
        'print("b" < "c")', 'print("a" + "b")', 'print("abc"[1..])', 'print("ab".chars().sorted())',
        'for c in "ab"\n    print(c)', 'print(["b", "a"].sorted())',
    ],
    "bytes": [
        'print(b"ab".len())', 'print(b"ab".to_str() else "")', 'print(b"ab".text())', 'print(b"ab".to_list())',
        'print(b"a".concat(b"b").len())', 'print(b"ab".find(b"b") else -1)', 'print(b"ab".starts_with(b"a"))',
        'print(b"ab".ends_with(b"b"))', 'print(b"ab".contains(b"b"))', 'print(b"ab".hex())', 'print(b"ab".base64())',
        'print(b"ab"[0])', 'print(b"abc"[1..].len())', 'print((b"a" + b"b").len())', 'for x in b"ab"\n    print(x)',
        'print(bytes.from_list([1.to_u8()]).len())', 'print((bytes.from_hex("6162") else b"").len())',
        'print((bytes.from_base64("YWI=") else b"").len())', 'print(bytes.zeros(3).len())', 'print(b"a" == b"a")',
        'print(b"a")', 'print(b"a" < b"b")',
    ],
    "char": [
        "print('1'.is_digit())", "print('a'.is_alpha())", "print('a'.is_alnum())", "print(' '.is_space())",
        "print('A'.is_upper())", "print('a'.is_lower())", "print('a'.to_upper())", "print('A'.to_lower())",
        "print('a'.code())", "print('f'.to_digit(16) else -1)", "print(char.from_code(97) else 'z')",
        "print('a' < 'b')", "print(\"{'a'}\")",
    ],
    "int": [
        "print(6.bit_and(3))", "print(6.bit_or(3))", "print(6.bit_xor(3))", "print(6.bit_not())", "print(1.shl(3))",
        "print(8.shr(2))", "print(1.checked_add(2) else 0)", "print(1.checked_sub(2) else 0)",
        "print(2.checked_mul(3) else 0)", "print(6.checked_div(0) else -1)", "print(5.checked_neg() else 0)",
        "print(7.checked_rem(3) else 0)", "print(5.wrapping_neg())", "print(1.wrapping_add(2))",
        "print(1.wrapping_sub(2))", "print(2.wrapping_mul(3))", "print(1.saturating_add(2))",
        "print(1.saturating_sub(2))", "print(2.saturating_mul(3))", "print((-3).abs())", "print(3.to_float())",
        "print(255.to_hex())", "print(3.to_str())", "print(3.to_i8())", "print(3.to_i16())", "print(3.to_i32())",
        "print(3.to_u8())", "print(3.to_u16())", "print(3.to_u32())", "print(3.to_u64())", "print(3.to_f32())",
        "print(7 / 2)", "print(7 % 3)",
    ],
    "float": [
        "print((-1.5).abs())", "print(1.5.floor())", "print(1.5.ceil())", "print(1.5.round())", "print(4.0.sqrt())",
        "print(0.0.sin())", "print(0.0.cos())", "print(0.0.tan())", "print(0.0.atan())", "print(0.0.atan2(1.0))",
        "print(0.0.exp())", "print(1.0.ln())", "print(10.0.log10())", "print(2.0.pow(3.0))",
        "print(3.0.hypot(4.0))", "print(8.0.cbrt())", "print(1.5.to_int())", "print(1.5.to_f32())",
        "print([1.5, 0.5].sorted())", "print([1.5, 0.5].sum())", "print([1.5, 0.5].max() else 0.0)",
    ],
    "sized": [
        "x := 3.to_u8()\nprint(x.checked_add(1.to_u8()) else 0.to_u8())", "x := 3.to_i32()\nprint(x.wrapping_neg())",
        "x := 3.to_i32()\nprint(x.abs())", "x := 3.to_u16()\nprint(x.to_int())", "x := 3.to_u32()\nprint(x.to_float())",
        "x := 3.to_u64()\nprint(x.to_hex())", "x := 3.to_i16()\nprint(x.to_str())",
        "x := 3.to_u8()\nprint(x.saturating_sub(5.to_u8()))", "x := 1.5.to_f32()\nprint(x.sqrt())",
        "x := 1.5.to_f32()\nprint(x.to_float())", "xs := [3.to_u8(), 1.to_u8()]\nprint(xs.sorted())",
        "xs := [3.to_i32(), 1.to_i32()]\nprint(xs.sum())",
    ],
}

# Constructs beyond the method tables, each its own snippet group (one function per snippet, shared decls).
EXTRA_DECLS = [
    ("Method", "trait Method\n    fn winner(self, xs List[str]) str ! VoteErr"),
    ("VoteErr", "error struct VoteErr\n    why str"),
    ("Plur", "struct Plur\n    tag str\n\nimpl Method for Plur\n    fn winner(self, xs List[str]) str ! VoteErr\n        if xs.is_empty()\n            return VoteErr(why: \"empty\")\n        xs[0]"),
    ("Named", "trait Named\n    fn name(self) str"),
    ("Tone", "enum Tone\n    Low\n    High(n int)\n\nimpl Named for Tone\n    fn name(self) str\n        match self\n            Low => \"low\"\n            High(n) => \"high {n}\""),
    ("bump_shard", "fn bump_shard(shards List[Mutex[Map[str, int]]], w str)\n    s := shards[0]\n    with s as mut m\n        m[w] += 1"),
]

EXTRA = {
    # corpus voting_systems: `else` on a fallible method called through `any Trait`.
    "dyn_fallible_else": "mut ms := List[any Method]()\nms.push(Plur(tag: \"p\"))\nfor m in ms\n    w := m.winner([\"a\"]) else e\n        print(e.why)\n        continue\n    print(w)",
    "dyn_fallible_else_value": "m := Plur(tag: \"p\")\nw := m.winner([]) else \"none\"\nprint(w)",
    # corpus sharded_counter_tasks: a list of mutexes, passed to a task.
    "list_of_mutex": "mut shards := List[Mutex[Map[str, int]]]()\nshards.push(Mutex(Map[str, int]()))\nbump_shard(shards, \"a\")\nprint([s.get() for s in shards].len())",
    # An enum converted to `any Trait` (trait.any.1).
    "enum_to_any": "xs := List[any Named]()\nmut ys := xs\nys.push(Tone.Low)\nys.push(Tone.High(n: 2))\nfor x in ys\n    print(x.name())",
    # corpus channel_broadcast_sim: a list of channels.
    "list_of_channels": "mut chans := List[Channel[int]]()\nfor _ in 0..2\n    chans.push(Channel[int](4))\nfor c in chans\n    c.send(1)\nprint(chans.len())",
    "list_of_atomics": "xs := [Atomic(0), Atomic(0)]\nxs[1].add(2)\nprint(xs[1].load())",
    "list_of_mutex_tasks": "mut shards := List[Mutex[Map[str, int]]]()\nfor _ in 0..2\n    shards.push(Mutex(Map[str, int]()))\ng := TaskGroup()\nfor k in 0..2\n    g.add(spawn bump_shard(shards, \"a\"))\ng.wait_all() else return\nprint(shards[0].get().len())",
}

# Whole packages (path -> text), for what crosses files: checked as `main.ail`, then built.
PKGS = {
    # Two files of one module with the same function name (mod.unit.1): one public, one private.
    "dup_fn_in_module": {
        "main.ail": "use util\n\nfn main()\n    print(util.rename_all(1, \"a\", \"b\"))\n",
        "util/edit.ail": "pub fn rename_all(pos int, old str, text str) str\n    \"{pos}{old}{text}\"\n",
        "util/resolve.ail": "fn rename_all(e int, lines List[str]) int\n    e + lines.len()\n\npub fn use_it() int\n    rename_all(1, [])\n",
    },
    # A private generic function of another module, called unqualified (trial-b10's `absent[ast.Pos]()`).
    "private_generic_other_module": {
        "main.ail": "use util\nuse other\n\nfn main()\n    print(other.run())\n    print(util.size())\n",
        "util/util.ail": "fn nothing[T]() T?\n    none\n\npub fn size() int\n    x := nothing[int]() else 3\n    x\n",
        "other/other.ail": "pub fn run() int\n    y := nothing[int]() else 4\n    y\n",
    },
    # A program's own `Heap` beside the prelude's `Grid` (ai-review p2_route; dd083278 fixed it in the tree,
    # the seed before it still refused `Grid.fill`).
    "own_heap_with_grid": {
        "main.ail": "struct Heap[T Ord]\n    items List[T]\n\nfn Heap[T].new() Heap[T]\n    Heap[T](items: [])\n\nfn main()\n    mut g := Grid.fill(2, 3, -1)\n    g[1, 2] = 4\n    h := Heap[int].new()\n    print(g[1, 2] + h.items.len())\n",
    },
    # The same in the root module, across two files.
    "dup_fn_in_root": {
        "main.ail": "fn twice(x int) int\n    x * 2\n\nfn main()\n    print(twice(2))\n",
        "other.ail": "fn twice(s str) str\n    s\n",
    },
}

GROUPS = [
    ("list", LIST),
    ("mapk", MAPK),
    ("mapv", MAPV),
    ("set", SET),
    ("heap", HEAP),
    ("deque", DEQUE),
    ("grid", GRID),
    ("seq", SEQ),
    ("value", VALUE),
]


def indent(body, n=1):
    pad = "    " * n
    return "\n".join(pad + line if line else line for line in body.split("\n"))


def fill(body, tname, t):
    ty, a, b, c, key, _ = t
    key2 = "x" if "x" not in key else key  # unused
    out = body.replace("@T", ty).replace("@A", a).replace("@B", b).replace("@C", c)
    out = out.replace("@K2", "v").replace("@K", key).replace("@N", tname)
    return out


def groups():
    """(file stem, [(snippet name, body)], extra decls) for every group."""
    out = []
    for gname, table in GROUPS:
        for tname, t in TYPES.items():
            snippets = [("{}.{}[{}]".format(gname, m, tname), fill(body, tname, t)) for m, body in table.items()]
            decls = [(fill(n, tname, t), fill(d, tname, t)) for n, d in PER_TYPE_DECLS]
            out.append(("{}_{}".format(gname, tname), snippets, decls))
    out.append(("extra", list(EXTRA.items()), [(n, d) for n, d in EXTRA_DECLS]))
    for sname, lines in SCALAR.items():
        snippets = [("{}.{}".format(sname, i), body) for i, body in enumerate(lines)]
        out.append(("scalar_{}".format(sname), snippets, []))
    return out


def program(snippets, decls):
    """The program text and, per snippet or declaration, its first and last line (1-based)."""
    lines = (HEADER + "\n" + GENERIC_DECLS).split("\n")
    spans = []
    for name, text in decls:
        lines.append("")
        start = len(lines) + 1
        lines.extend(text.split("\n"))
        spans.append(("decl " + name, start, len(lines)))
    for i, (name, body) in enumerate(snippets):
        lines.append("")
        lines.append("## {}".format(name))
        start = len(lines) + 1
        lines.append("fn s{}()".format(i))
        lines.extend(indent(body).split("\n"))
        spans.append((name, start, len(lines)))
    lines.append("")
    lines.append("fn main()")
    if not snippets:
        lines.append('    print("-- none")')
    for i, (name, _) in enumerate(snippets):
        lines.append('    print("-- {}")'.format(name))
        lines.append("    s{}()".format(i))
    return "\n".join(lines) + "\n", spans


ERR = re.compile(r":(\d+):\d+: error\[([^\]]+)\]: (.*)")


def errors(text):
    for line in text.splitlines():
        m = ERR.search(line)
        if m:
            yield int(m.group(1)), m.group(2), m.group(3)


def owner(spans, line):
    for name, a, b in spans:
        if a <= line <= b:
            return name
    return None


def run(binary, cmd, path, out=None, timeout=300):
    args = [binary, cmd, path] + (["-o", out] if out else [])
    try:
        r = subprocess.run(args, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return 124, "timeout"
    return r.returncode, r.stdout + r.stderr


def drive(binary, keep, freeze=None):
    work = keep or tempfile.mkdtemp()
    if freeze:
        os.makedirs(freeze, exist_ok=True)
    os.makedirs(work, exist_ok=True)
    holes, refused, runs, built = [], 0, [], 0
    for stem, snippets, decls in groups():
        live = list(snippets)
        dropped = {}
        for _ in range(8):
            text, spans = program(live, decls)
            path = os.path.join(work, stem + ".ail")
            with open(path, "w") as f:
                f.write(text)
            code, log = run(binary, "check-program", path)
            bad = {}
            stray = []
            for line, rule, msg in errors(log):
                o = owner(spans, line)
                if o is None:
                    stray.append("{}:{} {}".format(line, rule, msg))
                else:
                    bad.setdefault(o, "{} {}".format(rule, msg))
            if code == 0 and not bad:
                break
            if not bad:
                print("GROUP-RED {} check failed outside the snippets: {}".format(stem, "; ".join(stray[:3]) or log.strip()[-300:]))
                live = []
                break
            gone = [n[5:] for n in bad if n.startswith("decl ")]
            decls = [d for d in decls if d[0] not in gone]
            for s in live:
                if s[0] not in bad and any(g in s[1] for g in gone):
                    bad[s[0]] = "uses a refused declaration"
            dropped.update(bad)
            live = [s for s in live if s[0] not in bad]
        refused += len(dropped)
        for name, why in sorted(dropped.items()):
            print("refused {}: {}".format(name, why[:160]))
        if not live:
            continue
        text, spans = program(live, decls)
        path = os.path.join(work, stem + ".ail")
        with open(path, "w") as f:
            f.write(text)
        exe = os.path.join(work, stem)
        code, log = run(binary, "build", path, exe, 900)
        if code != 0:
            found = False
            for line, rule, msg in errors(log):
                o = owner(spans, line) or "{}:{}".format(stem, line)
                holes.append((o, rule, msg))
                print("HOLE {}: error[{}] {}".format(o, rule, msg[:200]))
                found = True
            if not found:
                print("HOLE {}: build failed: {}".format(stem, log.strip()[-400:]))
                holes.append((stem, "build", log.strip()[-200:]))
            continue
        built += len(live)
        try:
            r = subprocess.run([exe], capture_output=True, text=True, timeout=60, cwd=work)
            code, rout = r.returncode, r.stdout + r.stderr
        except subprocess.TimeoutExpired:
            code, rout = 124, "timeout"
        with open(os.path.join(work, stem + ".run"), "w") as f:
            f.write(rout)
        if freeze and code == 0:
            with open(os.path.join(freeze, stem + ".ail"), "w") as f:
                f.write(text)
            with open(os.path.join(freeze, stem + ".out"), "w") as f:
                f.write(rout)
        if code != 0:
            last = [l for l in rout.splitlines() if l.startswith("-- ")]
            at = last[-1][3:] if last else stem
            runs.append(at)
            print("RUN-RED {} (exit {}): {}".format(at, code, " | ".join(rout.strip().splitlines()[-3:])[:300]))
    for pname, files in PKGS.items():
        d = os.path.join(work, "pkg_" + pname)
        for rel, text in files.items():
            os.makedirs(os.path.dirname(os.path.join(d, rel)), exist_ok=True)
            with open(os.path.join(d, rel), "w") as f:
                f.write(text)
        code, log = run(binary, "check-program", d if "other.ail" in files else os.path.join(d, "main.ail"))
        if code != 0:
            first = next(errors(log), None)
            print("refused pkg.{}: {}".format(pname, "{} {}".format(first[1], first[2]) if first else log.strip()[-160:]))
            refused += 1
            continue
        code, log = run(binary, "build", os.path.join(d, "main.ail"), os.path.join(d, "x"), 900)
        if code != 0:
            first = next(errors(log), None)
            holes.append(("pkg." + pname, "build", log))
            print("HOLE pkg.{}: {}".format(pname, "error[{}] {}".format(first[1], first[2]) if first else log.strip()[-300:]))
            continue
        try:
            r = subprocess.run([os.path.join(d, "x")], capture_output=True, text=True, timeout=60, cwd=d)
            if r.returncode != 0:
                runs.append("pkg." + pname)
                print("RUN-RED pkg.{} (exit {}): {}".format(pname, r.returncode, (r.stdout + r.stderr).strip()[-300:]))
            else:
                built += 1
        except subprocess.TimeoutExpired:
            runs.append("pkg." + pname)
            print("RUN-RED pkg.{} timeout".format(pname))
    print("SUMMARY built-and-ran {} snippets, {} refused by check, {} holes, {} run failures".format(built, refused, len(holes), len(runs)))
    return 1 if holes or runs else 0


def emit(d):
    os.makedirs(d, exist_ok=True)
    for stem, snippets, decls in groups():
        text, _ = program(snippets, decls)
        with open(os.path.join(d, stem + ".ail"), "w") as f:
            f.write(text)


def main():
    if len(sys.argv) >= 3 and sys.argv[1] == "emit":
        emit(sys.argv[2])
        return 0
    if len(sys.argv) >= 3 and sys.argv[1] == "run":
        opts = sys.argv[3:]
        keep = opts[opts.index("--keep") + 1] if "--keep" in opts else None
        freeze = opts[opts.index("--freeze") + 1] if "--freeze" in opts else None
        return drive(sys.argv[2], keep, freeze)
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main())
