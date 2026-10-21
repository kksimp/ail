import hashlib
import hmac
import zlib


def main():
    print(f"sha256 of abc: {hashlib.sha256(b'abc').hexdigest()}")
    print(f"sha512 of empty: {hashlib.sha512(b'').hexdigest()}")
    mac = hmac.new(b"key", b"The quick brown fox jumps over the lazy dog", hashlib.sha256)
    print(f"hmac-sha256: {mac.hexdigest()}")
    print(f"crc32 of 123456789: {zlib.crc32(b'123456789') & 0xFFFFFFFF}")


if __name__ == "__main__":
    main()
