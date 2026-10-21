import ctypes
import ctypes.util


def main():
    libm = ctypes.CDLL(ctypes.util.find_library("m"))
    libm.hypot.restype = ctypes.c_double
    libm.hypot.argtypes = [ctypes.c_double, ctypes.c_double]
    libc = ctypes.CDLL(ctypes.util.find_library("c"))
    print(repr(float(libm.hypot(3.0, 4.0))))
    print(libc.abs(ctypes.c_int(-7)))
    print(libc.toupper(ctypes.c_int(97)))
    print(libc.strlen(ctypes.c_char_p("héllo".encode())))
    print(libc.atoi(ctypes.c_char_p(b"1234 apples")))


if __name__ == "__main__":
    main()
