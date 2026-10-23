# garlic: ws
tests: ws.ws.addresses_split_into_host_port_and_path, ws.ws.sha1_matches_the_fips_vectors, ws.ws.the_accept_key_follows_rfc_6455

## ws
```
ws.OP_CONT := 0
ws.OP_TEXT := 1
ws.OP_BIN := 2
ws.OP_CLOSE := 8
ws.OP_PING := 9
ws.OP_PONG := 10
ws.GUID := "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"  ## The GUID the server's answer is hashed with (RFC 6455 1.3).
pub struct ws.Sock  ## An open connection: `buf` holds bytes read but not yet framed, `part`
    conn net.Conn
    buf List[int]
    part List[int]
    part_op int
    pub open bool
    pub protocol str  ## What the server accepted of the requested sub-protocols.
    fn ws.Sock.frame(mut self, op int, payload List[int]) bool [io mutates net random]  ## Writes one masked frame; false when the write failed.
    pub fn ws.Sock.send_text(mut self, text str) !net.NetError [fail io mutates net random]  ## Sends `text` as one text message.
        fails with net.NetError
    pub fn ws.Sock.close(mut self, code int) [io mutates net random]  ## Starts the closing handshake with `code` (1000 for a normal close).
    fn ws.Sock.fill(mut self) bool [io mutates net]  ## Pulls what has arrived into `buf`; false when the connection ended.
    pub fn ws.Sock.poll(mut self, wait_ms int) List[ws.Event] [io mutates net random]  ## The events for the frames now complete, waiting up to `wait_ms` for data.
pub struct ws.Event  ## Something that happened on a connection: "message" (`text`), "close"
    pub kind str
    pub text str
    pub code int
fn ws.fail(url str, message str) net.NetError
fn ws.rol(x int, n int) int
pub fn ws.sha1(data List[int]) List[int]  ## The SHA-1 digest of `data` (FIPS 180-4), as 20 byte values.
fn ws.bytes_of(xs List[int]) bytes
fn ws.ints_of(b bytes) List[int]
pub fn ws.accept_for(key str) str  ## The `Sec-WebSocket-Accept` a server must answer `key` with.
fn ws.parse(url str) (bool, str, int, str) ! net.NetError [fail]  ## Splits `ws://host:port/path` or `wss://...` into (tls, host, port, path).
    fails with net.NetError
fn ws.read_head(conn net.Conn) List[str] ! net.NetError [fail io net]
pub fn ws.connect(url str, protocols str, origin str, cookie str) ws.Sock ! net.NetError [fail io net random]  ## Opens `url`, offering `protocols` (comma-separated, may be empty), the
```
