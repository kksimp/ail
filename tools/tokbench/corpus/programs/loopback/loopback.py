import json
import threading
import urllib.parse
import urllib.request
from http.server import BaseHTTPRequestHandler, HTTPServer


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        url = urllib.parse.urlsplit(self.path)
        query = dict(urllib.parse.parse_qsl(url.query))
        print(f"server got {self.command} {url.path} query {json.dumps(query)}")
        body = b"hi there"
        self.send_response(200)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


def main():
    server = HTTPServer(("127.0.0.1", 0), Handler)
    port = server.server_address[1]
    worker = threading.Thread(target=server.handle_request)
    worker.start()
    with urllib.request.urlopen(f"http://127.0.0.1:{port}/hello?name=ail") as reply:
        body = reply.read().decode()
        status = reply.status
    worker.join()
    server.server_close()
    print(f"client got {status}: {body}")


if __name__ == "__main__":
    main()
