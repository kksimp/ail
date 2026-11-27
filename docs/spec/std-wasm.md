# Standard library: `std.wasm`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1290). Reads WebAssembly binary modules
> into values, checks their structure and writes them back out. There is no execution engine, no
> text format (`.wat`) and no type check of instruction operands (`std.wasm.8`).

The module is pure (`eff.infer.1`): it has no effects and needs no capability. It is written in AIL and is in
scope without `use` (`std.use.4`); `wasm.parse(...)` names it. It reads the core binary format 1 with the
sign-extension, saturating-conversion, bulk-memory, reference-types, multi-value, tail-call and fixed-width SIMD
operations. Exception handling (the tag section), threads and atomics, 64-bit and multiple memories and
shared limits are refused as `Unsupported` (`std.wasm.10`).

## The module

**std.wasm.1** — `parse(data bytes) Module ! WasmError` reads a binary module: the 8-byte header (`\0asm`, version
1), then the sections. `empty() Module` is a module with nothing in it. A `Module` has the fields `functypes`
(`List[FuncType]`), `imports` (`List[Import]`), `funcs` (the type index of each function the module defines),
`tables`, `mems` (`List[Limits]`), `globals` (`List[GlobalDef]`), `exports`, `start` (`int?`), `elems`
(`List[ElemSegment]`), `datacount` (`int?`), `code` (`List[Body]`, one per entry of `funcs`), `datas`
(`List[DataSegment]`) and `customs`. All fields are public and a module is built by naming them, so a program
can make or change a module and `write` it. *(ADR 1290)*

```ail tool=run,FILE expect=2 rules=std.wasm.1
fn main() !
    m := wasm.parse(b"\x00asm\x01\x00\x00\x00\x01\x04\x01\x60\x00\x00\x03\x02\x01\x00\x0a\x04\x01\x02\x00\x0b")?
    print(m.funcs.len() + m.functypes.len())
```

**std.wasm.2** — `enum ValType` is `I32`, `I64`, `F32`, `F64`, `V128`, `FuncRef` and `ExternRef`. A
`FuncType` has `params` and `results`, both `List[ValType]`. `Limits` has `minimum int` and `maximum int?`
(elements for a table, 64 KiB pages for a memory). `TableType` has `elem` and `limits`; `GlobalType` has `ty`
and `mutable bool`. *(ADR 1290)*

```ail tool=run,FILE expect=I32,I64 rules=std.wasm.2
fn main() !
    m := wasm.parse(b"\x00asm\x01\x00\x00\x00\x01\x06\x01\x60\x01\x7f\x01\x7e")?
    print("{m.functypes[0].params[0].name()},{m.functypes[0].results[0].name()}")
```

**std.wasm.3** — An `Import` has `module str`, `name str` and `desc`, an `ImportDesc`: `ImpFunc(type_index)`,
`ImpTable(table)`, `ImpMemory(limits)` or `ImpGlobal(gtype)`. An `Export` has `name str`, `kind ExternKind` (`Func`,
`Table`, `Memory`, `Global`) and `index int` in that kind's index space. Imports come first in each index space:
function 0 is the first imported function, if there is one. *(ADR 1290)*

```ail tool=run,FILE expect=env.log,add rules=std.wasm.3
fn main() !
    m := wasm.parse(b"\x00asm\x01\x00\x00\x00\x01\x04\x01\x60\x00\x00\x02\x0b\x01\x03env\x03log\x00\x00\x07\x07\x01\x03add\x00\x00")?
    print("{m.imports[0].module}.{m.imports[0].name},{m.exports[0].name}")
```

**std.wasm.4** — The other sections read into plain fields. `GlobalDef` has `gtype` and `init`. `ElemSegment` has
`flags` (the file's form, 0 to 7), `table`, `offset`, `reftype`, `funcs` (forms 0 to 3) and `exprs` (forms 4 to
7); `table` and `offset` mean something for the active forms 0, 2, 4 and 6. `DataSegment` has `flags` (0 active
in memory 0, 1 passive, 2 active in `memory`), `memory`, `offset` and `bytes`. `Body` has `locals`
(`List[LocalDecl]`, each a `count` and a `ty`) and `code`. A `Custom` section has `name`, `data` and `after`, the
id of the section it followed (0 at the start); `m.custom(name)` finds one (`Custom?`). *(ADR 1290)*

```ail tool=run,FILE expect=hi,2,0 rules=std.wasm.4
fn main() !
    m := wasm.parse(b"\x00asm\x01\x00\x00\x00\x00\x04\x02hi!\x05\x03\x01\x00\x01\x0b\x08\x01\x00\x41\x00\x0b\x02ab")?
    c := m.custom("hi") else return
    print("{c.name},{m.datas[0].bytes.len()},{c.after}")
```

## Instructions

**std.wasm.5** — A function body, global initialiser, element offset or data offset is a `List[Instr]` in file
order, the closing `end` included; blocks are not nested, their `block`, `loop`, `if`, `else` and `end`
instructions are in the list as they are in the file. An `Instr` has `op int` (the opcode; an operation of the
0xFC group is `0xFC0000 + n` and of the 0xFD group `0xFD0000 + n`), `imm List[int]` and `data bytes`. The
immediates, in file order:

| Operations | `imm` |
|---|---|
| `block`, `loop`, `if` | the block type: -64 for none, -1 to -5 and -16, -17 for a value type, 0 or more for a type index |
| `br`, `br_if`, `call`, `return_call`, `ref.func`, `local.*`, `global.*`, `table.get`, `table.set`, `table.grow`, `table.size`, `table.fill`, `data.drop`, `elem.drop`, `memory.size`, `memory.grow`, `memory.fill` | one index |
| `br_table` | every label, then the default label |
| `call_indirect`, `return_call_indirect`, `memory.init`, `memory.copy`, `table.init`, `table.copy` | two indexes, in file order |
| loads and stores | alignment, offset |
| SIMD lane loads and stores | alignment, offset, lane |
| SIMD lane extract and replace | the lane |
| `select` with types | the value type bytes (`0x7f` for `i32`) |
| `i32.const`, `i64.const` | the value |
| `f32.const`, `f64.const` | the bits as an `int` (`f32` in 0 to 2^32-1) |
| `ref.null` | the reference type byte |
| `v128.const`, `i8x16.shuffle` | none; the 16 bytes are `data` |

`ins.name()` is the operation's name as the text format spells it (`i32.add`, `local.get`), `simd.N` for a SIMD
operation without a name here; `print(ins)` shows the name then the immediates. *(ADR 1290)*

```ail tool=run,FILE expect=local.get|i32.const|end rules=std.wasm.5
fn main() !
    m := wasm.parse(b"\x00asm\x01\x00\x00\x00\x03\x02\x01\x00\x0a\x08\x01\x06\x00\x20\x03\x41\x7e\x0b")?
    mut shown := List[str]()
    for ins in m.code[0].code
        shown.push(ins.name())
    print(shown.join("|"))
```

## Reading

**std.wasm.6** — `parse` refuses what is not a well-formed file and says where, in the text (`(byte N)`): a
missing header or other version (`BadHeader`); a count or section that runs past the end of its container
(`Truncated`); an unknown or repeated or out-of-order section id, a section that does not use all its bytes, or
a different number of function entries and code bodies (`BadSection`); an unknown value type, block type,
form byte or a name that is not UTF-8 (`BadValue`); an unknown opcode (`BadOpcode`); a feature in the lead
paragraph's list of refused ones (`Unsupported`). The sections are in the order of the format; custom
sections may sit anywhere. Numbers are read as the format's LEB128; a number with more bytes than its width allows
is `BadValue`. *(ADR 1290)*

```ail tool=run,FILE expect=BadSection rules=std.wasm.6
fn main()
    m := wasm.parse(b"\x00asm\x01\x00\x00\x00\x03\x02\x01\x00") else err
        print(err.kind.name())
        return
    print(m.funcs.len())
```

## Writing

**std.wasm.7** — `write(m Module) bytes ! WasmError` writes the header and each section that has content (a
non-empty list, or `start` or `datacount` set) in the format's order, with each custom section after the
section it followed. Numbers are written in their shortest form, so `write(parse(b))` has the content of `b`
but the same bytes only when `b` used shortest forms and no empty sections. `parse(write(m))` equals `m` for
every module `parse` can make. `write` refuses, as `Invalid`, an instruction that is not an operation, one with
the wrong number of immediates, an index or count outside 0 to 2^32-1, an `i32.const` outside 32 bits and a
`v128.const` without 16 bytes of `data`. *(ADR 1290)*

```ail tool=run,FILE expect=0061736d01000000050401010104 rules=std.wasm.7
fn main() !
    mut m := wasm.empty()
    m.mems.push(wasm.Limits(minimum: 1, maximum: 4))
    print(wasm.write(m)?.hex())
```

## Checking

**std.wasm.8** — `validate(m Module) ! WasmError` checks the structure and fails with `Invalid` and a text that
names the place: every type, function, table, memory, global, data and element index is in range (imports
count); `funcs` and `code` have the same length; a limit's maximum is not below its minimum and a memory has
at most 65,536 pages; the module has at most one memory; export names are unique and each index is in range;
the start function takes and gives nothing; the data count equals the number of data segments; global
initialisers and segment offsets are constant expressions (constants, `global.get`, `ref.null`, `ref.func`,
the integer `add`, `sub`, `mul`); in a body every `block`, `loop` and `if` is closed, `else` is inside an `if` and
used once, every `br`, `br_if` and `br_table` names an open label, every local index is a parameter or local, a
memory operation needs a memory, and the body ends with `end`. It does not type-check: the operand stack and
the types of operands are not followed, so a module that passes can still be refused by an engine. *(ADR 1290)*

```ail tool=run,FILE expect=Invalid rules=std.wasm.8
fn main()
    mut m := wasm.empty()
    m.funcs.push(0)
    m.code.push(wasm.Body(locals: List[wasm.LocalDecl](), code: [wasm.Instr(op: 11, imm: List[int](), data: b"")]))
    wasm.validate(m) else err
        print(err.kind.name())
        return
    print("valid")
```

## Looking things up

**std.wasm.9** — `m.func_count()` is the size of the function index space (imported functions, then defined
ones). `m.func_type(i)` is the `FuncType?` of function `i` in that space, `none` when there is none.
`m.export(name)` is the `Export?` called `name` and `m.custom(name)` the `Custom?`. *(ADR 1290)*

```ail tool=run,FILE expect=1,true rules=std.wasm.9
fn main() !
    m := wasm.parse(b"\x00asm\x01\x00\x00\x00\x01\x04\x01\x60\x00\x00\x03\x02\x01\x00\x07\x05\x01\x01f\x00\x00\x0a\x04\x01\x02\x00\x0b")?
    print("{m.func_count()},{m.export("f") != none}")
```

## Errors

**std.wasm.10** — `WasmError` is an error with `kind WasmErrorKind` and `why str`; its text is `why`. The
kinds are `BadHeader`, `Truncated`, `BadSection`, `BadValue`, `BadOpcode`, `Unsupported` (`std.wasm.6`) and
`Invalid` (`std.wasm.7`, `std.wasm.8`). Match on `err.kind`, never on `why`. *(ADR 1290)*

```ail tool=run,FILE expect=BadHeader rules=std.wasm.10
fn main()
    m := wasm.parse(b"not wasm") else err
        print(err.kind.name())
        return
    print(m.funcs.len())
```
