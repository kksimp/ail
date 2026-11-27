# Standard library: `std.db.mysql`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/db/mysql.ail`, ADR 1283). Not run against a live server by the
> gate: the suite `std_mysql` checks the wire encoding and decoding with goldens. The TLS paths have an interop job
> against real servers (`tools/interop/db-tls.sh`).

A MySQL and MariaDB client in AIL over `net.Conn`: protocol 10 handshake and protocol 41 packets, with no C
library. It is the network counterpart of [std.db](17-std.md), whose `Value`, `Row` and `query`/`exec` shapes it
follows, and the sibling of the PostgreSQL client ([std-postgres.md](std-postgres.md)). Use it with
`use std.db.mysql` (always with `use`, `std.use.4`), as `mysql`.

## The module

**std.mysql.1** — `use std.db.mysql` provides `connect(addr str) Conn ! MyError`,
`connect_with(cfg Config) Conn ! MyError`, `config(user str, database str) Config`,
`config_from_url(addr str) Config ! MyError`, `pool(cfg Config, cap int) Pool` and the types `Conn`, `Config`,
`Pool`, `Row`, `Value`, `Column`, `Packet`, `Handshake`, `OkPacket`, `Stmt`, `MyKind` and the error type `MyError`. Details: [below](#more-on-stdmysql1). *(ADR 1283)*

**std.mysql.2** — Packets are built by pure functions. Details: [below](#more-on-stdmysql2). *(ADR 1283)*

```ail rules=std.mysql.1,std.mysql.2
use std.db.mysql

fn main()
    print(mysql.frame(1, b"abc").hex())              # 03000001616263
    print(mysql.encode_stmt_close(7).hex())          # 1907000000
    if p := mysql.split_packet(mysql.frame(0, b"hi"))
        print("{p.seq} {p.body.text()}")             # 0 hi
```

**std.mysql.3** — `decode_handshake(body bytes) Handshake ! MyError` reads the server's first packet (protocol
10): `version`, `thread_id`, the 20-byte `salt`, the `capabilities` and `status` flags, the `charset` and the
preferred login `plugin`. Details: [below](#more-on-stdmysql3). *(ADR 1283)*

```ail rules=std.mysql.3
use std.db.mysql

fn main()
    h := mysql.decode_handshake(b"\xff\x15\x04#28000Access denied") else err
        print("{err.kind.name()} {err.errno}")   # Server 1045
        return
    print(h.version)
```

**std.mysql.4** — `native_scramble(password str, salt bytes) bytes` is the `mysql_native_password` answer:
SHA1(password) XOR SHA1(salt + SHA1(SHA1(password))), 20 bytes. Details: [below](#more-on-stdmysql4). *(ADR 1283)*

```ail rules=std.mysql.4
use std.db.mysql

fn main()
    print(mysql.native_scramble("", b"01234567890123456789").len())   # 0
    print(mysql.sha2_scramble("pw", b"01234567890123456789").len())   # 32
```

**std.mysql.5** — `decode_ok(body bytes) OkPacket` reads an OK packet: `affected`, `last_id`, `status`,
`warnings` (length-encoded integers). Details: [below](#more-on-stdmysql5). *(ADR 1283)*

```ail rules=std.mysql.5
use std.db.mysql

fn main()
    ok := mysql.decode_ok(b"\x00\x03\x07\x02\x00\x00\x00")
    print("{ok.affected} {ok.last_id} {ok.status}")                    # 3 7 2
    e := mysql.decode_error(b"\xff\x26\x04#23000Duplicate entry")
    print("{e.errno} {e.code} {e.message}")                            # 1062 23000 Duplicate entry
```

**std.mysql.6** — `decode_column(body bytes) Column` reads a column definition: `name`, the type code `kind`,
the `flags` (32 is UNSIGNED) and the `charset` (63 is binary). `decode_value(col Column, data bytes) Value` types
a column from its text form: the integer types and YEAR (1, 2, 3, 8, 9, 13) are `Int` (an unsigned BIGINT above
the largest `int` stays `Text`); FLOAT and DOUBLE are `Real`; a string or BLOB type with the binary character set,
BIT and GEOMETRY are `Blob`; every other type (DECIMAL, dates and times, JSON, ENUM, SET, text) is `Text` exactly
as the server wrote it, so no precision is lost. `decode_text_row(cols, body bytes) List[Value]` reads a row of the
text protocol (length-prefixed strings, byte 0xFB for NULL). `Row.float` also reads `Int` and a `Text` that is a
number; `Row.bool` reads `Bool` and an integer (MySQL has no boolean: `TINYINT(1)` is `Int`). A value prints as
`NULL`, `true`, `42`, `2.5`, the text, or `X'0102'`. *(ADR 1283)*

```ail rules=std.mysql.6
use std.db.mysql

fn main()
    cols := [mysql.Column(name: "n", kind: 8, flags: 0, charset: 63), mysql.Column(name: "s", kind: 253, flags: 0, charset: 45)]
    vals := mysql.decode_text_row(cols, b"\x0242\x02hi")
    print(vals[0])   # 42
    print(vals[1])   # hi
```

**std.mysql.7** — Prepared statements use the binary protocol. Details: [below](#more-on-stdmysql7). *(ADR 1283)*

```ail rules=std.mysql.7
use std.db.mysql

fn main()
    print(mysql.encode_execute(3, [mysql.Value.Int(1)]).hex())   # 17030000000001000000000108000100000000000000
    col := mysql.Column(name: "a", kind: 1, flags: 0, charset: 63)
    print(mysql.decode_binary_row([col], b"\x00\x00\xfb")[0])    # -5
```

**std.mysql.8** — `config(user, database) Config` has the defaults `localhost`, port 3306, no password, a 30 s
timeout. Details: [below](#more-on-stdmysql8). *(ADR 1283)*

```ail rules=std.mysql.8
use std.db.mysql

fn main()
    c := mysql.config_from_url("mysql://al:p%40ss@db:3307/shop") else err
        print(err.message)
        return
    print("{c.host} {c.port} {c.user} {c.password} {c.database}")   # db 3307 al p@ss shop
```

**std.mysql.9** — `connect(addr)` and `connect_with(cfg)` open a connection: TCP, the server's handshake, the login
and the server's OK. Details: [below](#more-on-stdmysql9). *(ADR 1283)*

```ail norun rules=std.mysql.9
use std.db.mysql

fn main()
    c := mysql.connect("mysql://root:secret@127.0.0.1/shop") else err
        print("{err.kind.name()}: {err.message}")
        return
    print(c.server_version)
```

**std.mysql.10** — `query(sql, params List[Value]) List[Row] ! MyError` runs a statement with `?` marks and returns
its rows (an empty list when it returns none); `exec(sql, params) int ! MyError` returns the rows changed, and
`last_id` holds the AUTO_INCREMENT id of the last INSERT. Details: [below](#more-on-stdmysql10). *(ADR 1283)*

```ail norun rules=std.mysql.10
use std.db.mysql

fn main()
    mut c := mysql.connect("mysql://root:secret@127.0.0.1/shop") else err
        print(err.message)
        return
    c.batch("CREATE TABLE IF NOT EXISTS t (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(40))") else err
        print(err.message)
        return
    n := c.exec("INSERT INTO t (name) VALUES (?)", [mysql.Value.Text("ann")]) else 0
    rows := c.query("SELECT id, name FROM t WHERE name = ?", [mysql.Value.Text("ann")]) else []
    print("{n} {c.last_id} {rows.len()}")
```

**std.mysql.11** — `begin()` is `START TRANSACTION`, `commit()` and `rollback()` their statements,
`in_transaction()` the server's last status flag. *(ADR 1283)*

**std.mysql.12** — A failed call is a `MyError` with `kind` (`MyKind`: `BadUrl`, `Auth`, `Server`, `Protocol`,
`Closed`, `TimedOut`, `Net`, `Usage`), `message`, `errno` (the server's error number, 0 for a client-side failure)
and `code` (the SQLSTATE, empty for a client-side failure). Details: [below](#more-on-stdmysql12). *(ADR 1283)*

```ail rules=std.mysql.12
use std.db.mysql

fn main()
    e := mysql.decode_error(b"\xff\x7a\x04#42S02Table 'x' doesn't exist")
    print("{e.kind.name()} {e.errno} {e.code}")   # Server 1146 42S02
```

**std.mysql.13** — `pool(cfg, cap) Pool` keeps up to `cap` connections open, made on demand. Details: [below](#more-on-stdmysql13). *(ADR 1283)*

**std.mysql.14** — Not done, and said plainly: the full `caching_sha2_password` exchange without TLS (it needs
RSA-OAEP, which `std.crypto` does not have; see `std.mysql.16`); LOCAL INFILE (refused); compression; cursors and
long-data parameters; client certificates; `sha256_password`, `mysql_clear_password` and authentication plugins
other than the two above. Details: [below](#more-on-stdmysql14). *(ADR 1284)*

**std.mysql.15** — TLS. `Config.tls` is a `Tls`: `Disable`, `Prefer` (the default) or `Require`. Unless it is
`Disable` and the server's greeting has the SSL capability (bit 11), `connect_with` sends the SSL request
(`encode_ssl_request(database)`: the first 32 bytes of the login with `CLIENT_SSL` set, as packet 1), runs the
TLS 1.3 handshake of `std.net.50` on the same connection (`start_tls` with `cfg.host`), and sends the login
(`encode_login_tls`, `CLIENT_SSL` set) inside it as packet 2. A server without the capability is used in the
clear by `Prefer` and refused by `Require` (kind `Tls`). A failed handshake or certificate check is kind `Tls` and
is never retried in the clear. The chain and the host name are always verified against the system roots
(`std.net.5`; `SSL_CERT_FILE` names a bundle instead), so there is no mode that skips verification. The URL form
is `?ssl-mode=disable|prefer|require|verify-ca|verify-full` (also MySQL's `DISABLED`, `PREFERRED`, `REQUIRED`,
`VERIFY_CA`, `VERIFY_IDENTITY`, in any case; the last three mean `Require`); `tls_from_text(text)` is the
parser; unknown values fail with kind `BadUrl`. `Prefer` falls back when the greeting lacks the capability, so an
attacker who can alter the stream can strip TLS: use `Require` where that matters. The TLS client offers the X25519 group only (`std.net.5`): MySQL 8.0 accepts NIST groups
only and answers `handshake_failure`, which is kind `Tls`; MariaDB and a front that terminates TLS work. *(ADR 1284)*

```ail rules=std.mysql.15
use std.db.mysql

fn main()
    print(mysql.encode_ssl_request("").hex())   # 05aa0b00ffffff002d0000000000000000000000000000000000000000000000
    c := mysql.config_from_url("mysql://app@db.example.com/shop?ssl-mode=REQUIRED") else err
        print(err.message)
        return
    print(c.tls.name())   # Require
    print(mysql.config("app", "shop").tls.name())   # Prefer
```

**std.mysql.16** — The full `caching_sha2_password` exchange. Details: [below](#more-on-stdmysql16). *(ADR 1284)*

```ail norun rules=std.mysql.16
use std.db.mysql

fn main()
    cfg := mysql.config_from_url("mysql://app:secret@db.example.com/shop?ssl-mode=require") else err
        print(err.message)
        return
    c := mysql.connect_with(cfg) else err
        print("{err.kind.name()}: {err.message}")
        return
    print(c.server_version)
```

### `std.mysql` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.mysql.1

A `Value` is `Null`, `Bool(b bool)`, `Int(n int)`, `Real(x float)`, `Text(s str)` or `Blob(data bytes)`. A `Row` has the fields `columns List[str]` and `values List[Value]`, and `index(name)`, `get(name)`, `is_null(name)`, and `int`, `float`, `str`, `bytes` and `bool` (each `T?`, `none` for NULL or another type). `get` of a name that is not a column stops the program. A `Conn` is used from one task at a time.

#### More on std.mysql.10

With parameters the call prepares the statement, runs it in the binary protocol and closes it; a value is never part of the SQL text. Without parameters the text protocol runs `sql` as written in one round trip, and it may hold several statements separated by `;`: the first result set is kept and the rows changed are summed. `batch(sql) ! MyError` runs such text and returns no rows; `ping()` sends COM_PING. `prepare(sql) Stmt ! MyError` gives a handle to run with `query_stmt(s, params)` and `exec_stmt(s, params)` as often as needed; `close_stmt(s)` frees it. A wrong number of parameters fails with kind `Usage`; a server error with kind `Server`, and the connection stays usable.

#### More on std.mysql.12

`1062` with `23000` is a duplicate key. A `Server` error leaves the connection usable; `Protocol`, `Closed`, `TimedOut` and `Net` may not (`is_open()` says).

#### More on std.mysql.13

`get() Conn ! MyError` returns a free connection or makes one while fewer than `cap` are out; when all are out it fails with kind `Usage` (it never waits, so a task cannot deadlock on its own connections). `put(c)` gives one back; a closed connection, or one with a transaction open, is closed instead of kept. `query` and `exec` are `get`, the call and `put`. `close()` closes the idle connections.

#### More on std.mysql.14

MySQL 8.4 and later turn `mysql_native_password` off by default: use TLS, so the full `caching_sha2_password` exchange works, or enable it for the user.

#### More on std.mysql.16

When the server answers the scrambled password with "perform full authentication" (`01 04`), a TLS connection (`std.mysql.15`) sends the password as a NUL-ended packet, which the encrypted stream protects, and reads the server's OK; the server then caches the user, so later logins take the fast path. Without TLS the exchange needs the server's RSA key and RSA-OAEP, which are not provided: the call fails with kind `Auth` and a message that names TLS.

#### More on std.mysql.2

`frame(seq int, body bytes) bytes` is the 3-byte little-endian length of the body, the sequence number (one byte), then the body; a `Conn` splits a body of 16777215 bytes or more and joins such packets on reading. `split_packet(buf bytes) Packet?` is the first whole packet of `buf`, or `none` when more bytes are needed; it took `4 + p.body.len()` bytes. Command bodies: `encode_query(sql)` (COM_QUERY), `encode_prepare(sql)` (COM_STMT_PREPARE), `encode_stmt_close(id)`, `encode_ping()` and `encode_quit()`.

#### More on std.mysql.3

An ERR packet there fails with kind `Server` and another protocol version with kind `Protocol`. `encode_login(user, answer bytes, database, plugin) bytes` is the client's HandshakeResponse41: capabilities (protocol 41, secure connection, plugin auth, transactions, multiple statements and results, and connect-with-database when `database` is not empty), a 16 MiB packet limit, utf8mb4, the user, the answer (length-prefixed), the schema and the plugin name.

#### More on std.mysql.4

`sha2_scramble(password, salt) bytes` is the `caching_sha2_password` answer: SHA256(password) XOR SHA256(SHA256(SHA256(password)) + salt), 32 bytes. Both are empty for an empty password. SHA-1 is broken, but it is the server's protocol.

#### More on std.mysql.5

`decode_error(body bytes) MyError` reads an ERR packet: kind `Server`, `errno` the error number, `code` the SQLSTATE after `#` (empty when the packet has none) and the message.

#### More on std.mysql.7

`decode_prepare_ok(body bytes) Stmt` reads the first packet of a prepare answer: `id`, `params` and `columns`. `encode_execute(id int, params List[Value]) bytes` is a COM_STMT_EXECUTE body for one run: the NULL bitmap, the types (`Null` NULL, `Bool` TINY, `Int` LONGLONG, `Real` DOUBLE, `Text` VAR_STRING, `Blob` BLOB), then the values. `decode_binary_row(cols, body bytes) List[Value]` reads a binary row: header byte 0, a NULL bitmap with an offset of two bits, then each value by its type: signed or unsigned TINY, SHORT, YEAR, LONG, INT24 and LONGLONG as `Int` (an unsigned BIGINT above the largest `int` as `Text`), FLOAT and DOUBLE as `Real`, dates, datetimes, timestamps and times as the `Text` MySQL prints (`2024-01-02 03:04:05.000250`, `-26:03:04`), any other type as `decode_value` gives.

#### More on std.mysql.8

`config_from_url(addr) Config ! MyError` reads `mysql://[USER[:PASSWORD]@]HOST[:PORT][/DB]` (or `mariadb://`); user and password are percent-decoded, a `?query` is ignored, an IPv6 host is written `[::1]`. The user defaults to `root`, the schema to none. A bad address, port or escape fails with kind `BadUrl`.

#### More on std.mysql.9

The login answers the server's plugin: `mysql_native_password`, or `caching_sha2_password` when the server has the user's password cached (its fast path); an authentication switch is followed. A refused login fails with kind `Auth`; so does the full `caching_sha2_password` exchange when the connection is not TLS (std.mysql.16). `Conn` has the public fields `thread_id`, `server_version` and `last_id`; `is_open()`, `set_timeout(ms)` and `close()` (COM_QUIT; closing a closed connection does nothing). Transport failures give kinds `Net`, `Closed`, `TimedOut`. After the login the connection runs `SET NAMES utf8mb4`, so client, results and connection use UTF-8 even when the server ignores the handshake's character set (`character_set_client_handshake=0`, a latin1 `init_connect`).
