REASONS = {200: "OK", 404: "Not Found", 405: "Method Not Allowed"}


def route(method, path):
    if path != "/health":
        return 404, "no such route"
    if method != "GET":
        return 405, "use GET"
    return 200, f"ok {path}"


def render(status, body):
    return f"HTTP/1.1 {status} {REASONS[status]}\r\nContent-Length: {len(body)}\r\n\r\n{body}"


def main():
    lines = ["GET /health HTTP/1.1", "POST /health HTTP/1.1", "GET /nope HTTP/1.1", "junk"]
    for line in lines:
        parts = line.split(" ")
        if len(parts) < 2:
            print(f"400 bad request: {line}")
            continue
        status, body = route(parts[0], parts[1])
        print(render(status, body).replace("\r", ""))
        print("--")


if __name__ == "__main__":
    main()
