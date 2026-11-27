"""Interop check for std.protobuf against the reference implementation.

usage: check.py AIL_BINARY WORKDIR
Encodes messages with the reference library and decodes them with AIL, and the
other way round, across every wire type, then compares the canonical JSON.
"""
import base64, json, math, os, subprocess, sys
from google.protobuf import json_format

here = os.path.dirname(os.path.abspath(__file__))
ail, work = sys.argv[1], sys.argv[2]
os.makedirs(work, exist_ok=True)
subprocess.run([sys.executable, "-m", "grpc_tools.protoc", f"-I{here}", f"--python_out={work}", f"{here}/all.proto"], check=True)
sys.path.insert(0, work)
import all_pb2 as pb

def build(i):
    m = pb.All()
    if i == 0:
        m.f_double = -2.25; m.f_float = 0.5; m.f_int32 = -7; m.f_int64 = -(2**62)
        m.f_uint32 = 4294967295; m.f_uint64 = 2**64 - 1
        m.f_sint32 = -2147483648; m.f_sint64 = -(2**63)
        m.f_fixed32 = 4294967295; m.f_fixed64 = 2**64 - 1
        m.f_sfixed32 = -2147483648; m.f_sfixed64 = -(2**63)
        m.f_bool = True; m.f_string = "héllo 世界"; m.f_bytes = bytes(range(256))
        m.f_color = pb.GREEN
    elif i == 1:
        m.f_inner.city = "Oslo"; m.f_inner.nums.extend([1, -1, 300, 2**31 - 1, -2**31])
        m.r_int32.extend([0, 1, -1, 127, 128, 16384, -16384])
        m.r_sint64.extend([0, -1, 1, 2**62, -(2**63), 2**63 - 1])
        m.r_double.extend([0.0, 1.5, -1e300])
        m.r_fixed32.extend([0, 1, 4294967295])
        m.r_string.extend(["", "a", "bc"])
        for c, n in (("x", [1]), ("", []), ("z", [5, 6])):
            r = m.r_inner.add(); r.city = c; r.nums.extend(n)
        m.r_color.extend([pb.RED, pb.GREEN, pb.COLOR_UNSET, 9])
        m.r_unpacked.extend([1, 2**64 - 1, 3])
    elif i == 2:
        m.m_str_int["a"] = 1; m.m_str_int["b"] = -2; m.m_str_int[""] = 0
        m.m_int_str[-5] = "neg"; m.m_int_str[2**62] = "big"
        m.m_str_inner["k"].city = "Rome"
        m.o_int32 = 0
        m.c_int = -9
    elif i == 3:
        m.c_str = "pick"
        m.o_int32 = -1
    return m

fails = 0
for i in range(4):
    m = build(i)
    wire = m.SerializeToString()
    wp, jp, op = (os.path.join(work, f"{i}.{e}") for e in ("bin", "json", "out"))
    open(wp, "wb").write(wire)
    want = json_format.MessageToDict(m, preserving_proto_field_name=True)
    # reference bytes -> AIL JSON
    got = json.loads(subprocess.run([ail, "run", f"{here}/interop.ail", "to", f"{here}/all.proto", "All", wp], capture_output=True, text=True, check=True).stdout)
    if got != want:
        fails += 1
        print(f"case {i}: AIL decode differs\n  want {json.dumps(want, sort_keys=True)}\n  got  {json.dumps(got, sort_keys=True)}")
    # JSON -> AIL bytes -> reference parse
    open(jp, "w").write(json.dumps(want))
    subprocess.run([ail, "run", f"{here}/interop.ail", "from", f"{here}/all.proto", "All", jp, op], check=True)
    back = pb.All()
    back.ParseFromString(open(op, "rb").read())
    if back != m:
        fails += 1
        print(f"case {i}: AIL encode differs\n  want {m}\n  got  {back}")
    print(f"case {i}: {len(wire)} bytes, {'ok' if fails == 0 else 'FAIL'}")
sys.exit(1 if fails else 0)
