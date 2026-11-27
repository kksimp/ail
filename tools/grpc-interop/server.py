"""grpcio server for the interop check of the AIL client: raw bytes, no protobuf.
Writes its port to the file named by the first argument, then serves until killed."""
import sys
from concurrent import futures
import grpc


def echo(req, ctx):
    ctx.set_trailing_metadata((("x-trail", "done"),))
    return b"echo:" + req


def count(req, ctx):
    for i in range(1, int(req) + 1):
        yield f"m{i}".encode()


def total(it, ctx):
    return str(sum(int(m) for m in it)).encode()


def chat(it, ctx):
    for m in it:
        yield b"you said " + m


def fail(req, ctx):
    ctx.abort(grpc.StatusCode.NOT_FOUND, "not here: ü")


def slow(req, ctx):
    import time
    time.sleep(1.0)
    return b"late"


ident = lambda b: b
H = grpc.method_handlers_generic_handler("t.S", {
    "Echo": grpc.unary_unary_rpc_method_handler(echo, ident, ident),
    "Count": grpc.unary_stream_rpc_method_handler(count, ident, ident),
    "Sum": grpc.stream_unary_rpc_method_handler(total, ident, ident),
    "Chat": grpc.stream_stream_rpc_method_handler(chat, ident, ident),
    "Fail": grpc.unary_unary_rpc_method_handler(fail, ident, ident),
    "Slow": grpc.unary_unary_rpc_method_handler(slow, ident, ident),
})
server = grpc.server(futures.ThreadPoolExecutor(max_workers=4))
server.add_generic_rpc_handlers((H,))
port = server.add_insecure_port("127.0.0.1:0")
open(sys.argv[1], "w").write(str(port))
server.start()
server.wait_for_termination()
