import base64


def main():
    raw = bytes.fromhex("48656c6c6f2c2041494c")
    print(f"{raw.decode()} as base64: {base64.b64encode(raw).decode()}")
    back = base64.b64decode(base64.b64encode(raw))
    print(f"round trip: {str(back == raw).lower()}")
    print(f"hex again: {back.hex()}")


if __name__ == "__main__":
    main()
