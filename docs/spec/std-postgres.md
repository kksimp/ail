# Standard library: `std.db.postgres`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/db/postgres.ail`, ADR TBD). Not run against a live server by the
> gate: the suite `std_postgres` checks the wire encoding and decoding with goldens. The TLS path has an interop job
> against a real server (`tools/interop/db-tls.sh`, run by the runner on ail-linux).

A PostgreSQL client in AIL over `net.Conn`: the frontend/backend protocol version 3, with no C library. It is the
network counterpart of [std.db](17-std.md), whose `Value`, `Row` and `query`/`exec` shapes it follows. Use it with
`use std.db.postgres` (always with `use`, `std.use.4`), as `postgres`.

## The module

**std.postgres.1** — `use std.db.postgres` provides `connect(addr str) Conn ! PgError`,
`connect_with(cfg Config) Conn ! PgError`, `config(user str, database str) Config`,
`config_from_url(addr str) Config ! PgError`, `pool(cfg Config, cap int) Pool` and the types `Conn`, `Config`,
`Pool`, `Row`, `Value`, `Column`, `Message`, `PgKind` and the error type `PgError`. A `Value` is `Null`,
`Bool(b bool)`, `Int(n int)`, `Real(x float)`, `Text(s str)` or `Blob(data bytes)`. A `Row` has the fields
`columns List[str]` and `values List[Value]`, and `index(name)`, `get(name)`, `is_null(name)`, and `int`, `float`,
`str`, `bytes` and `bool` (each `T?`, `none` for NULL or another type). `get` of a name that is not a column stops
the program. A `Conn` is used from one task at a time. *(ADR TBD)*

**std.postgres.2** — Wire messages are built by pure functions. `frame(kind int, body bytes) bytes` is the type
byte (none for kind 0) and the 32-bit big-endian length of the body plus 4, then the body.
`encode_startup(user, database)` is the startup message (protocol 3.0), `encode_query(sql)` a Query message,
`encode_password(secret bytes)` a PasswordMessage (also the SASL messages) and `encode_terminate()` a Terminate.
*(ADR TBD)*

```ail rules=std.postgres.1,std.postgres.2
use std.db.postgres

fn main()
    print(postgres.encode_query("SELECT 1").hex())   # 510000000d53454c454354203100
    print(postgres.encode_terminate().hex())         # 5800000004
```

**std.postgres.3** — `encode_extended(sql str, params List[Value]) bytes` is Parse, Bind, Describe (portal),
Execute and Sync for one statement with the unnamed statement and portal. Parameters `$1`, `$2`, ... are sent in
text form with no declared type, so the server infers each: `Int` and `Real` as their decimal text, `Bool` as `t`
or `f`, `Text` as it is, `Blob` as `\x` and hex (the `bytea` input form), `Null` as a NULL. Results are asked for
in text form. A value is never part of the SQL text. *(ADR TBD)*

```ail rules=std.postgres.3
use std.db.postgres

fn main()
    wire := postgres.encode_extended("SELECT $1", [postgres.Value.Int(42)])
    print(wire.len())   # 59
```

**std.postgres.4** — `split_message(buf bytes) Message?` is the first whole backend message of `buf` (type byte
and body), or `none` when more bytes are needed; it took `5 + m.body.len()` bytes. `decode_error(body bytes)
PgError` reads an ErrorResponse (or NoticeResponse): kind `Server`, `message` from field `M`, `code` the SQLSTATE
from `C`, `detail` from `D`. `decode_columns(body bytes) List[Column]` reads a RowDescription (name and type OID)
and `decode_row(cols, body bytes) List[Value]` a DataRow. *(ADR TBD)*

```ail rules=std.postgres.4
use std.db.postgres

fn main()
    body := bytes.from_hex("534552524f5200433233353035004d6475706c6963617465206b65790000") else b""
    e := postgres.decode_error(body)
    print("{e.code} {e.message}")   # 23505 duplicate key
```

**std.postgres.5** — `decode_value(oid int, data bytes) Value` types a column from its text form and its type
OID: `bool` (16) is `Bool`; `int2`, `int4`, `int8` and `oid` (21, 23, 20, 26) are `Int`; `float4` and `float8` (700,
701) are `Real`; `bytea` (17) is `Blob`; every other type (`numeric`, `text`, `varchar`, dates and times, `uuid`,
`json`, arrays) is `Text` exactly as the server wrote it, so no precision is lost. `Row.float` also reads `Int` and
a `Text` that is a number. A value prints as `NULL`, `true`, `42`, `2.5`, the text, or `X'0102'`. *(ADR TBD)*

```ail rules=std.postgres.5
use std.db.postgres

fn main()
    print(postgres.decode_value(20, b"-9"))                      # -9
    print(postgres.decode_value(16, b"f"))                       # false
    print(postgres.decode_value(1114, b"2024-01-02 03:04:05"))   # 2024-01-02 03:04:05
```

**std.postgres.6** — `md5_password(user, password, salt bytes) str` is the md5 login answer: `md5` and the hex MD5
of (the hex MD5 of password and user) followed by the 4-byte salt. MD5 is broken; the server asks for it only
when its `pg_hba.conf` says `md5`, and `connect` answers it. Cleartext password requests are answered as asked.
*(ADR TBD)*

```ail rules=std.postgres.6
use std.db.postgres

fn main()
    print(postgres.md5_password("user", "pass", b"\x01\x02\x03\x04"))
    # md56cf524962d8413e6b0cdf79fddff891c
```

**std.postgres.7** — SCRAM-SHA-256 (RFC 5802, RFC 7677), the server's default. `scram_client_first(user, nonce)`
is `n,,n=USER,r=NONCE`; `scram_client_final(user, password, nonce, server_first) ScramFinal ! PgError` reads
`r=...,s=SALT,i=COUNT` and gives `message` (the client-final with its proof) and `server_final` (the `v=...` the
server must answer with). It fails with kind `Auth` when the server nonce does not extend ours, a part is missing,
the salt is not base64 or the count is not 1 to 100000000. `connect` sends an empty user name, as the server
expects, and checks the server's signature with `crypto.ct_eq`; a wrong one fails with kind `Auth`. *(ADR TBD)*

```ail rules=std.postgres.7
use std.db.postgres

fn main()
    nonce := "rOprNGfwEbeRWgbNEkqO"
    print(postgres.scram_client_first("user", nonce))   # n,,n=user,r=rOprNGfwEbeRWgbNEkqO
    first := "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096"
    step := postgres.scram_client_final("user", "pencil", nonce, first) else err
        print(err.message)
        return
    print(step.server_final)   # v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=
```

**std.postgres.8** — `config(user, database) Config` has host `localhost`, port 5432, no password and
`timeout_ms` 30000. `config_from_url` reads `postgres://[USER[:PASSWORD]@]HOST[:PORT][/DB]` or `postgresql://`: user
and password are percent-decoded, a `?query` is ignored, an IPv6 host is `[::1]`, the user defaults to `postgres` and
the database to the user. Anything else fails with kind `BadUrl`. *(ADR TBD)*

```ail rules=std.postgres.8
use std.db.postgres

fn main()
    c := postgres.config_from_url("postgres://alice:s%40cret@db.example.com:6543/shop") else err
        print(err.message)
        return
    print("{c.host} {c.port} {c.user} {c.password} {c.database}")   # db.example.com 6543 alice s@cret shop
```

**std.postgres.9** — `connect_with(cfg)` opens TCP, sends the startup message, answers the login (trust, cleartext,
md5 or SCRAM-SHA-256) and reads the server's settings and key data up to its first ReadyForQuery. A refused login,
or a method not listed here, fails with kind `Auth` (with the server's `code` and message when it sent one); a
transport failure with `Net`, `Closed` or `TimedOut`. `c.settings` holds the ParameterStatus values and `c.pid` the
server process id. `c.close()` sends Terminate and closes; `c.is_open()` says whether the connection is usable;
`c.set_timeout(ms)` sets the wait for the server. *(ADR TBD)*

```ail norun rules=std.postgres.9
use std.db.postgres

fn main()
    mut c := postgres.connect("postgres://app:secret@127.0.0.1:5432/app") else err
        print("{err.kind.name()}: {err.message}")
        return
    print(c.settings.get("server_version") else "?")
    c.close()
```

**std.postgres.10** — `c.query(sql, params) List[Row] ! PgError` runs one statement through the extended protocol
(`std.postgres.3`) and returns its rows; `c.exec(sql, params) int ! PgError` returns the rows it changed, the count
of the CommandComplete tag (`INSERT 0 3` is 3, `UPDATE 2` is 2; 0 when the tag has none). `c.batch(sql) !
PgError` runs SQL text through the simple protocol, which allows several statements separated by `;`, and returns
no rows. After a server error the call still reads to ReadyForQuery, so the connection stays usable. *(ADR TBD)*

```ail norun rules=std.postgres.10
use std.db.postgres

fn main()
    mut c := postgres.connect("postgres://app@127.0.0.1/app") else err
        print(err.message)
        return
    c.batch("CREATE TABLE IF NOT EXISTS t (id int PRIMARY KEY, name text)") else err
        print(err.message)
        return
    n := c.exec("INSERT INTO t VALUES ($1, $2)", [postgres.Value.Int(1), postgres.Value.Text("ann")]) else 0
    rows := c.query("SELECT id, name FROM t WHERE id = $1", [postgres.Value.Int(1)]) else []
    for r in rows
        print("{n} {r.int("id") else -1} {r.str("name") else "?"}")
    c.close()
```

**std.postgres.11** — `c.begin()`, `c.commit()` and `c.rollback()` run `BEGIN`, `COMMIT` and `ROLLBACK`.
`c.in_transaction()` is true while the server's last ReadyForQuery status is not idle, which includes a transaction
that failed and must be rolled back. *(ADR TBD)*

```ail norun rules=std.postgres.11
use std.db.postgres

fn main()
    mut c := postgres.connect("postgres://app@127.0.0.1/app") else err
        print(err.message)
        return
    c.begin() else err
        print(err.message)
        return
    print(c.in_transaction())   # true
    c.rollback() else err
        print(err.message)
```

**std.postgres.12** — A server error is a `PgError` of kind `Server` with `code` the five-character SQLSTATE
(`23505` unique violation, `42P01` missing table, `40001` serialization failure), `message` and `detail`. `PgKind` is
`BadUrl`, `Auth`, `Server`, `Protocol` (a message that breaks the protocol: the connection is closed), `Closed`,
`TimedOut`, `Net` and `Usage`. After `Closed`, `TimedOut`, `Net` or `Protocol` the connection is closed. *(ADR TBD)*

```ail norun rules=std.postgres.12
use std.db.postgres

fn main()
    mut c := postgres.connect("postgres://app@127.0.0.1/app") else err
        print(err.message)
        return
    c.exec("INSERT INTO t VALUES (1, 'dup')", []) else err
        if err.kind == postgres.PgKind.Server and err.code == "23505"
            print("already there")
```

**std.postgres.13** — `pool(cfg, cap)` keeps up to `cap` connections. `p.get()` gives a free connection or opens a new
one while fewer than `cap` are out; with `cap` out it fails with kind `Usage` and never waits. `p.put(c)` takes it
back: a connection that is closed, or has a transaction open, is closed instead of kept. `p.query(sql, params)` and
`p.exec(sql, params)` are `get`, the call and `put`. `p.close()` closes the idle connections. *(ADR TBD)*

```ail norun rules=std.postgres.13
use std.db.postgres

fn main()
    mut p := postgres.pool(postgres.config("app", "app"), 4)
    rows := p.query("SELECT 1 AS one", []) else err
        print(err.message)
        return
    print(rows.len())   # 1
    p.close()
```

**std.postgres.14** — Not provided: `COPY`, `LISTEN` and `NOTIFY`, SCRAM-SHA-256-PLUS (channel binding), GSS and
SSPI, named prepared statements, binary result formats and cancel requests. A server that requires a login
method other than the four of `std.postgres.9` fails with kind `Auth`. *(ADR TBD)*

```ail norun rules=std.postgres.14
use std.db.postgres

fn main()
    c := postgres.connect("postgres://app:secret@10.0.0.5/app") else err
        print(err.kind.name())
        return
    print(c.is_open())
```

**std.postgres.15** — TLS. `Config.tls` is a `Tls`: `Disable`, `Prefer` (the default) or `Require`. Unless it is
`Disable`, `connect_with` first sends the SSLRequest (`encode_ssl_request()`: the bytes `00 00 00 08 04 d2 16 2f`) and
reads the server's one-byte answer: `S` starts the TLS 1.3 handshake of `std.net.42` on the same connection
(`start_tls` with `cfg.host`), and the startup message and the login go inside it; `N` means no TLS, which `Prefer`
accepts (the session goes on in the clear) and `Require` refuses; any other byte is kind `Protocol`. A failed handshake or
certificate check is kind `Tls` and is never retried in the clear, even for `Prefer`. The certificate chain and the
host name are always verified against the system roots (`std.net.5`; `SSL_CERT_FILE` names a bundle instead), so
there is no mode that skips verification. The URL form is `?sslmode=disable|prefer|require|verify-ca|verify-full`
(the last two mean `Require`; `allow` and unknown values fail with kind `BadUrl`); `tls_from_text(text)` is the
parser. Because `Prefer` falls back when the server answers `N`, an attacker who can alter the stream can strip TLS: use
`Require` where that matters. Channel binding (SCRAM-SHA-256-PLUS) and client certificates are not provided.
*(ADR TBD)*

```ail rules=std.postgres.15
use std.db.postgres

fn main()
    print(postgres.encode_ssl_request().hex())   # 0000000804d2162f
    c := postgres.config_from_url("postgres://app@db.example.com/app?sslmode=require") else err
        print(err.message)
        return
    print(c.tls.name())   # Require
    d := postgres.config("app", "app")
    print(d.tls.name())   # Prefer
```

## Rejected alternatives

- **libpq through `ffi`.** A C dependency and no WASM or kernel use; the protocol is small enough to write in AIL.
- **Binary result format.** Faster for numbers, but each type needs its own decoder; text form keeps `numeric`, dates
  and arrays exact as `Text` until a type is asked for.
- **Named prepared statements and a statement cache.** Saves a Parse per call, but a pool then needs per-connection
  caches; deferred until a measurement asks for it.
- **A mode that skips certificate verification (libpq `require` semantics).** It makes TLS protect nothing against an
  active attacker; a server with a private CA is reached by naming the CA in `SSL_CERT_FILE`.
- **A blocking pool.** `get` fails with `Usage` instead of waiting, so a task never deadlocks on its own connections.
