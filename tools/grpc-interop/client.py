"""grpcio client for the interop check: every call uses raw bytes, no protobuf."""
import sys
import grpc

port = sys.argv[1]
ident = lambda b: b
ch = grpc.insecure_channel(f"127.0.0.1:{port}")
ok = True


def check(name, got, want):
    global ok
    good = got == want
    ok = ok and good
    print(("ok   " if good else "FAIL ") + name + ("" if good else f": got {got!r} want {want!r}"))


echo = ch.unary_unary("/t.S/Echo", ident, ident)
check("unary", echo(b"hi", metadata=(("x-a", "1"),)), b"echo:hi")
r, call = echo.with_call(b"x", timeout=5)
check("unary status", call.code(), grpc.StatusCode.OK)

count = ch.unary_stream("/t.S/Count", ident, ident)
check("server streaming", list(count(b"4")), [b"m1", b"m2", b"m3", b"m4"])

sm = ch.stream_unary("/t.S/Sum", ident, ident)
check("client streaming", sm(iter([b"1", b"2", b"39"])), b"42")

chat = ch.stream_stream("/t.S/Chat", ident, ident)
check("bidi", list(chat(iter([b"a", b"b"]))), [b"you said a", b"you said b"])

try:
    echo_fail = ch.unary_unary("/t.S/Fail", ident, ident)
    echo_fail(b"x")
    check("status", None, "error")
except grpc.RpcError as e:
    check("error code", e.code(), grpc.StatusCode.NOT_FOUND)
    check("error message", e.details(), "not here: ü")

try:
    ch.unary_unary("/t.S/Nope", ident, ident)(b"x")
except grpc.RpcError as e:
    check("unimplemented", e.code(), grpc.StatusCode.UNIMPLEMENTED)

big = b"z" * 1000000
check("1 MB message", echo(big), b"echo:" + big)
sys.exit(0 if ok else 1)
