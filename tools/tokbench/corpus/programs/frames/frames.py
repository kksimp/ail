def main():
    wire = b"\x05hello\x03abc\x00\x02hi"
    at = 0
    n = 0
    while at < len(wire):
        size = wire[at]
        body = wire[at + 1:at + 1 + size]
        n += 1
        print(f"frame {n}: {body.decode('utf-8', 'replace')} ({body.hex()})")
        at += 1 + size
    print(f"{n} frames in {len(wire)} bytes")


if __name__ == "__main__":
    main()
