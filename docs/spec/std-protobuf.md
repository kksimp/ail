# Standard library: `std.protobuf`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/protobuf/protobuf.ail`, ADR 1293).

The Protocol Buffers wire format, proto3. `decode` reads bytes into a `Message`,
a list of numbered fields that keeps every field it saw, and `encode` writes it
back. A `Message` is built and read with `add_*` and `get_*` calls by field
number. `parse_schema` reads a `.proto` file, and `to_json` and `from_json` turn
a typed message into a `Json` object and back. Every function is pure
(`eff.infer.1`). Groups, services and extensions are not supported.

## The module

**std.protobuf.1** — `use std.protobuf` provides `new_message()`, `decode`,
`encode`, `zigzag`, `unzigzag`, `parse_schema`, `to_json`, `from_json`, the types
`Message`, `Field`, `Value`, `Schema`, `ProtoError` and `ProtoKind`, and the
methods of `Message` and `Schema` below. *(ADR 1293)*

**std.protobuf.2** — A `Message` has `fields List[Field]`, in wire order. A
`Field` has `num int` (1 to 536870911) and `value Value`. A `Value` is
`Varint(n int)` (wire type 0, the 64-bit pattern of the number, so a negative
`int32` is a 10-byte varint), `Fixed64(n int)` (wire type 1, the pattern),
`Bytes(b bytes)` (wire type 2: a string, bytes, a nested message or a packed
list) or `Fixed32(n int)` (wire type 5, 0 to 4294967295). *(ADR 1293)*

**std.protobuf.3** — `zigzag(n)` maps `0, -1, 1, -2` to `0, 1, 2, 3` over 64
bits, and `unzigzag` is its inverse. A varint on the wire is 1 to 10 bytes,
seven bits a byte, low bits first. *(ADR 1293)*

```ail rules=std.protobuf.1,std.protobuf.2,std.protobuf.3
use std.protobuf

fn main()
    print(protobuf.zigzag(-1))                # 1
    print(protobuf.unzigzag(4294967295))      # -2147483648
    m := protobuf.new_message()
    print(m.fields.len())                     # 0
```

**std.protobuf.4** — `decode(b)` gives the fields of `b` in wire order, or a
`ProtoError`. A packed list stays one `Bytes` value until it is read as a list
(`std.protobuf.9`). *(ADR 1293)*

**std.protobuf.5** — `encode(m)` writes each field of `m` in list order: its
tag, then its value. *(ADR 1293)*

**std.protobuf.6** — `decode` keeps every field, including numbers no schema
mentions, and `encode` writes them back in the same order, so
`encode(decode(b)) == b` when `b` used shortest varints. *(ADR 1293)*

```ail rules=std.protobuf.4,std.protobuf.5,std.protobuf.6
use std.protobuf

fn main() !
    raw := b"\x08\x01\x78\x05\x7a\x02hi"
    m := protobuf.decode(raw)?
    print(m.fields.len())                     # 3
    print(protobuf.encode(m) == raw)          # true
```

**std.protobuf.7** — A `mut` message has these builders, each appending one
field: `add_varint(num, n)` (`int32`, `int64`, `uint32`, `uint64`, enums),
`add_sint(num, n)` (zigzag), `add_bool(num, b)`, `add_fixed32(num, n)` (the low
32 bits), `add_fixed64(num, n)`, `add_float(num, x)`, `add_double(num, x)`,
`add_bytes(num, b)`, `add_str(num, s)`, `add_message(num, m)`,
`add_packed_ints(num, xs)`, `add_packed_doubles(num, xs)` and
`add_packed_floats(num, xs)`. Details: [below](#more-on-stdprotobuf7). *(ADR 1293)*

```ail rules=std.protobuf.7
use std.protobuf

fn main()
    mut inner := protobuf.new_message()
    inner.add_varint(1, 150)
    mut m := protobuf.new_message()
    m.add_str(2, "testing")
    m.add_message(3, inner)
    m.add_packed_ints(4, [3, 270, 86942])
    print(protobuf.encode(m).hex())   # 120774657374696e671a030896012206038e029ea705
```

**std.protobuf.8** — A message has `all(num)` (every `Value` of that number, in
order), `get(num)` (the last, which wins in a singular field, as `Value?`) and
`has(num)`. The typed reads give `none` when the field is absent or has another
wire type: `get_int` (`int32`, `int64`, `uint32`, `uint64`, enums), `get_sint`,
`get_bool`, `get_fixed32`, `get_fixed64` (the pattern), `get_float`,
`get_double`, `get_bytes`, `get_str` (also `none` for text that is not UTF-8).
`get_message(num)` is the nested message, an empty one when the field is absent,
and an error when the field is not length-delimited or its bytes are not a
message. *(ADR 1293)*

```ail rules=std.protobuf.8
use std.protobuf

fn main() !
    m := protobuf.decode(b"\x08\x96\x01\x12\x07testing")?
    print(m.get_int(1) else 0)                # 150
    print(m.get_str(2) else "")               # testing
    print(m.has(3))                           # false
    print(m.get_int(2) == none)               # true
```

**std.protobuf.9** — The repeated reads `ints`, `sints`, `doubles`, `floats`,
`strs` and `messages` give every value of a number in wire order. Details: [below](#more-on-stdprotobuf9). *(ADR 1293)*

```ail rules=std.protobuf.9
use std.protobuf

fn main() !
    packed := protobuf.decode(b"\x22\x06\x03\x8e\x02\x9e\xa7\x05")?
    print(packed.ints(4)?.join(","))          # 3,270,86942
    loose := protobuf.decode(b"\x20\x03\x20\x8e\x02")?
    print(loose.ints(4)?.join(","))           # 3,270
```

**std.protobuf.10** — `Message.get`, `all` and the reads never change the
message, and a message is a value: `m2 := m` is independent of `m`
(`mem.val.1`). *(ADR 1293)*

## Schemas

**std.protobuf.11** — `parse_schema(text)` reads `.proto` text into a `Schema`,
or fails with `ProtoError` and the 1-based line. Details: [below](#more-on-stdprotobuf11). *(ADR 1293)*

**std.protobuf.12** — The text read is: `syntax`, `package`, `message` and `enum`
(nested to any depth), fields `[repeated|optional|required] TYPE name = N
[options]`, `map<K, V>`, `oneof` (its fields have explicit presence) and the
option `packed`. Details: [below](#more-on-stdprotobuf12). *(ADR 1293)*

```ail rules=std.protobuf.11,std.protobuf.12
use std.protobuf

fn main() !
    s := protobuf.parse_schema("syntax = \"proto3\";\npackage demo;\nmessage A \{ message B \{ int32 x = 1; } B b = 1; }")?
    print(s.package)                          # demo
    print(s.names.join(","))                  # demo.A,demo.A.B
    print(s.has_message("A"))                 # true
```

**std.protobuf.13** — `to_json(s, msg, b)` reads `b` as message `msg` and gives a
`Json` object whose members are the declared fields that are on the wire, in
field-number order, named as in the `.proto`. Details: [below](#more-on-stdprotobuf13). *(ADR 1293)*

**std.protobuf.14** — `from_json(s, msg, j)` writes the object `j` as message
`msg`. Details: [below](#more-on-stdprotobuf14). *(ADR 1293)*

**std.protobuf.15** — A field's JSON form by type: `int32`, `uint32`, `sint32`,
`fixed32`, `sfixed32` and `bool` are a number or `true` or `false`; `float` and
`double` are a number (the strings `"NaN"`, `"Infinity"` and `"-Infinity"` are
read too); the 64-bit integer types are decimal strings, because a `Json`
number is a `float` (a number or a string is read); `string` is a string;
`bytes` is a base64 string; an enum is the value's name, or the number when
the enum has no such value; a message is an object. Details: [below](#more-on-stdprotobuf15). *(ADR 1293)*

```ail rules=std.protobuf.13,std.protobuf.14,std.protobuf.15
use std.protobuf

fn main() !
    s := protobuf.parse_schema("syntax = \"proto3\"; message P \{ string name = 1; repeated int64 ids = 2; }")?
    j := parse_json("\{\"name\":\"Ann\",\"ids\":[\"1\",\"300\"]}")?
    b := protobuf.from_json(s, "P", j)?
    print(b.hex())                            # 0a03416e6e120301ac02
    print(protobuf.to_json(s, "P", b)?.to_text())
```

**std.protobuf.16** — `ProtoError` has `kind ProtoKind`, `why str` and `at int`
(the byte offset for wire errors, the line for `.proto` text, else 0). The kinds
are `Truncated` (the bytes end inside a field), `BadVarint` (more than 64 bits),
`BadTag` (a field number outside 1 to 536870911), `BadWireType` (a group, or a
value of the wrong wire type for its schema type), `BadProto`, `NoMessage`,
`BadValue` and `BadUtf8` (a `string` field that is not UTF-8). *(ADR 1293)*

```ail rules=std.protobuf.16
use std.protobuf

fn main()
    _ := protobuf.decode(b"\x12\x05ab") else err
        print("{err.kind.name()}: {err}")   # Truncated: length-delimited field runs past the end (at 2)
```

### `std.protobuf` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.protobuf.11

`s.package` is the package, `s.names` the full names of the messages in file order (nested ones as `pkg.Outer.Inner`), and `s.has_message(name)` tests a name written in full or without the package.

#### More on std.protobuf.12

A type is a scalar (`double`, `float`, `int32`, `int64`, `uint32`, `uint64`, `sint32`, `sint64`, `fixed32`, `fixed64`, `sfixed32`, `sfixed64`, `bool`, `string`, `bytes`) or a message or enum, found from the innermost enclosing scope outward (a leading `.` is absolute). `import`, `option`, `reserved`, `extensions`, `extend` and `service` are skipped; `//` and `/* */` comments are blanks.

#### More on std.protobuf.13

A field the schema does not declare is dropped from the object (`decode` keeps it). A singular field takes its last value; a repeated field is an array, whether the wire has it packed or not; a `map` is an object. A message name that is not in the schema is `NoMessage`; a value of the wrong wire type is `BadWireType`.

#### More on std.protobuf.14

A member is found by the field's name or its lowerCamelCase form, fields are written in number order, and an absent or `null` member is left out. A singular field with its default (0, `""`, `false`) is left out unless it is `optional` or in a `oneof`; a message field is always written. A repeated numeric field is packed (unless `[packed = false]`). A value that does not fit the field's type is `BadValue`.

#### More on std.protobuf.15

Nested messages are limited to a depth of 64.

#### More on std.protobuf.7

A `num` outside 1 to 536870911 traps. `new_message()` is the empty message.

#### More on std.protobuf.9

A numeric list may be packed, unpacked or a mix of both, as the format allows. Each can fail with `ProtoError`.
