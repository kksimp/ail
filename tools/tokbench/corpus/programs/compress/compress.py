import gzip


def main():
    log = "".join("level=info msg=ok\n" for _ in range(200))
    packed = gzip.compress(log.encode())
    print(f"gzip is smaller: {len(packed) < len(log.encode())}".lower())
    back = gzip.decompress(packed).decode()
    print(f"round trip equal: {back == log}".lower())
    try:
        gzip.decompress(bytes([1, 2, 3]))
        rejected = False
    except OSError:
        rejected = True
    print(f"corrupt input rejected: {rejected}".lower())


if __name__ == "__main__":
    main()
