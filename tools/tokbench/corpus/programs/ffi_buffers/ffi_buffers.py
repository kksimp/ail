import ctypes
import ctypes.util


def main():
    libc = ctypes.CDLL(ctypes.util.find_library("c"))
    libc.strtod.restype = ctypes.c_double
    libc.strtod.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p)]
    a = (ctypes.c_int64 * 4)(100, 200, 300, 400)
    b = (ctypes.c_int64 * 4)()
    ctypes.memmove(b, a, 32)
    print(sum(b))
    buf = (ctypes.c_uint8 * 8)(*([1] * 8))
    ctypes.memset(buf, 255, 4)
    print(buf[0])
    print(buf[3])
    print(buf[4])
    text = b"3.25kg"
    rest = ctypes.c_char_p()
    value = libc.strtod(text, ctypes.byref(rest))
    print(repr(value))
    print((rest.value or b"").decode())


if __name__ == "__main__":
    main()
