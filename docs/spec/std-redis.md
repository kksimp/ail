# Standard library: `std.db.redis`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD). Written without a Redis server in the
> test suite: the tests play a scripted server over loopback (`tests/native/run/net_redis.ail`).

A Redis client over `net.Conn` (`std.net.12`, TLS as `std.net.13`). The module is written in AIL. Like
`std.net.websocket` it is used with `use std.db.redis`, always written, and named `redis`
(`std.use.4`). A `Client` is used from one task at a time, as a `Conn` is. Every function that talks to
the server has the effects `net` and `io`; `encode`, `decode` and the `Reply` methods are pure.

## The module

**std.redis.1** — `connect(address str) Client ! RedisError` opens
`redis://[user:password@]host[:port][/db][?protocol=3]` (port 6379, database 0, RESP2) or `rediss://...`
(TLS 1.3, port 6379 unless given). The host may be an IPv6 address in brackets. A userinfo with a `:`
is `user:password` (an empty user is the default user); one without is the password of the default
user. `%XX` in the user and password is decoded. A password sends `AUTH`; a database other than 0
sends `SELECT`; `protocol=3` (or `resp=3`) sends `HELLO 3`, with `AUTH` when there is a password, and
`c.proto` is then 3, otherwise 2. An address of another form, a port outside 1 to 65535 or a bad
database number fails with kind `BadUrl`; a refused password with `Auth`. `c.close()` closes the
connection; `c.set_timeout(ms)` is `Conn.set_timeout` (default 30000). *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.1
use std.db.redis

fn open() redis.Client ! redis.RedisError
    c := redis.connect("redis://app:s3cret@127.0.0.1:6379/2?protocol=3")?
    c.set_timeout(5000)
    c

fn main() ! redis.RedisError
    c := open()?
    print(c.proto)
    c.close()
```

**std.redis.2** — The codec is pure. `encode(args List[str]) bytes` and `encode_bytes(args List[bytes])
bytes` give the bytes of one command: an array of bulk strings, `*N\r\n$LEN\r\nARG\r\n...`.
`decode(data bytes) Parsed ! RedisError` decodes the first reply in `data` as `Parsed { reply, used }`;
`used` is 0 when `data` stops inside a reply (read more and call again). RESP2 types give `Reply.Status`
(`+`), `Failure` (`-`), `Int` (`:`), `Bulk` (`$`, binary safe) and `Array` (`*`); `$-1` and `*-1` give
`Nil`. RESP3 adds `_` (`Nil`), `#` (`Bool`), `,` (`Double`), `(` (`BigNum`), `=` (`Bulk` without its
three-letter format and `:`), `!` (`Failure`), `%` (`Table`, key and value alternating), `~` (`Members`)
and `>` (`Push`). Bytes that are not RESP fail with kind `Protocol`: an unknown type byte, a length or
number that does not read, a bulk string not followed by CRLF, a bulk string over 512 MiB, an aggregate
of over 16777216 items, nesting over 32. A double of `inf` or `nan` is refused. The methods
`as_text`, `as_bytes`, `as_int`, `as_float`, `as_bool` and `as_list` read a reply as that type (none
when it is not one; `as_list` also reads sets, pushes and a map as key, value, key, ...), and
`is_nil`, `is_failure` and `is_push` test it. *(ADR TBD)*

```ail tool=run,FILE expect=14 rules=std.redis.2
use std.db.redis

fn main() !
    print(redis.encode(["PING"]).len())
```

```ail tool=run,FILE expect=17 rules=std.redis.2
use std.db.redis

fn main() !
    got := redis.decode(b"*2\r\n$3\r\nfoo\r\n:7\r\n+next")?
    print(got.used)
```

**std.redis.3** — `c.command(args List[str]) Reply ! RedisError` sends one command and returns its
reply; `c.command_bytes(args List[bytes])` takes binary arguments. An error reply fails with kind
`Server` (kind `Auth` when it starts with `NOAUTH`, `WRONGPASS` or `NOPERM`) and the server's text as
the message. A RESP3 push that arrives outside a subscription is dropped. On a subscribed connection
every command fails with kind `Subscribed` (`std.redis.11`). Every typed helper below is `command`
with the arguments Redis documents, so `command` reaches whatever has no helper (scripts, `INFO`,
streams). *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.3
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    r := c.command(["OBJECT", "ENCODING", "k"])?
    print(r.as_text() else "none")
    c.close()
```

**std.redis.4** — Strings. `c.ping()` fails unless the answer is `PONG`; `c.select_db(n)`. `c.get(key) str?`
and `c.get_bytes(key) bytes?` give none for a missing key; `c.set(key, value)`, `c.set_bytes(key,
bytes)`, `c.set_ex(key, value, secs)` (`SET ... EX secs`), `c.set_nx(key, value, secs) bool` (`SET ...
NX`, with `EX secs` when `secs` is over 0; false when the key exists), `c.del(keys) int`,
`c.exists(key) bool`, `c.incr(key)`, `c.decr(key)`, `c.incr_by(key, delta)`, `c.append(key, value)` and
`c.strlen(key)`; the integers are the server's. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.4
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    c.set_ex("session", "abc", 60)?
    if c.set_nx("lock", "me", 10)?
        print("locked, hits {c.incr("hits")?}")
    print(c.get("session")? else "gone")
    c.close()
```

**std.redis.5** — Keys and expiry. `c.expire(key, secs)`, `c.pexpire(key, ms)` and `c.persist(key)` say
whether the server changed the key; `c.ttl(key)` is the seconds left, -1 for no expiry and -2 for a
missing key; `c.keys(pattern) List[str]` blocks the server on a large database; `c.scan(cursor,
pattern, count) ScanPage` is one page (`page.cursor`, `page.keys`): start at 0 and stop when `cursor` is 0
again; an empty `pattern` matches all and a `count` of 0 leaves it to the server. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.5
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    mut cursor := 0
    while true
        page := c.scan(cursor, "user:*", 100)?
        for k in page.keys
            print("{k} {c.ttl(k)?}")
        cursor = page.cursor
        if cursor == 0
            break
    c.close()
```

**std.redis.6** — Hashes. `c.hset(key, field, value)`, `c.hset_all(key, Map[str, str])` (fields in sorted
order, so the command is the same on every run), `c.hget(key, field) str?`, `c.hgetall(key)
Map[str, str]`, `c.hdel(key, fields)`, `c.hexists`, `c.hincr_by(key, field, delta)`, `c.hkeys` and
`c.hlen`. `hgetall` reads the RESP2 flat array and the RESP3 map. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.6
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    c.hset_all("user:1", {"name": "Ann", "plan": "pro"})?
    print(c.hgetall("user:1")?["name"])
    c.close()
```

**std.redis.7** — Lists. `c.lpush(key, values)` and `c.rpush(key, values)` give the new length;
`c.lpop(key)` and `c.rpop(key)` give none for an empty or missing list; `c.llen(key)`;
`c.lrange(key, start, stop) List[str]` (`stop` -1 is the last element). Blocking pops are not
provided: they hold the connection past its timeout; use `command`. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.7
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    c.rpush("jobs", ["a", "b"])?
    print(c.lpop("jobs")? else "empty")
    c.close()
```

**std.redis.8** — Sets and sorted sets. `c.sadd`, `c.srem` (member lists, counts back),
`c.smembers`, `c.sismember` and `c.scard`. `c.zadd(key, score, member)`, `c.zrem(key, members)`,
`c.zrange(key, start, stop) List[str]` (lowest score first), `c.zrange_scored(...) List[Scored]`
(`s.member`, `s.score`; the RESP2 flat list and the RESP3 list of pairs are both read),
`c.zscore(key, member) float?`, `c.zrank(key, member) int?`, `c.zcard` and `c.zincr_by(key, delta,
member) float`. Scores are written as `float` text (`1.5`). *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.8
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    c.zadd("scores", 1.5, "ann")?
    for s in c.zrange_scored("scores", 0, -1)?
        print("{s.member} {s.score}")
    c.close()
```

**std.redis.9** — `c.publish(channel, message) int` is the number of subscribers that received it. It
is a command, so it is not allowed on a subscribed connection. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.9
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    print(c.publish("news", "hello")?)
    c.close()
```

**std.redis.10** — Pipelines. `redis.pipeline()` is an empty `Pipeline`; `p.add(args List[str])` and
`p.add_bytes(args)` queue a command; `p.len()`. `c.run(p) List[Reply] ! RedisError` writes every queued
command in one write and reads one reply each, in order; a server error is a `Reply.Failure` in the
list and does not fail the call, so look at each one. `c.transaction(p)` sends `MULTI`, the queued
commands and `EXEC`, and returns the replies `EXEC` gave; a command the server rejects while queuing
fails the call with kind `Server`, and an empty list means the transaction was aborted (a watched
key changed). *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.10
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    mut p := redis.pipeline()
    p.add(["SET", "a", "1"])
    p.add(["INCR", "a"])
    for r in c.transaction(p)?
        print(r.as_text() else "?")
    c.close()
```

**std.redis.11** — Pub/sub. `c.subscribe(channels)` and `c.psubscribe(patterns)` (globs) wait for one
confirmation per name; `c.unsubscribe(names)` and `c.punsubscribe(names)` do the same (at least one name
is required); `c.subscriptions()` is the number of channels and patterns left. While it is over 0 the
connection is subscribed: it can only call these functions and `next_message`; anything else fails
with kind `Subscribed`. `c.next_message() Message ! RedisError` waits for the next message and returns
`Message { kind, channel, pattern, data }` (`kind` is `MsgKind.Channel` or `MsgKind.Pattern`; `pattern`
is `""` for a channel message; `m.text()` is the payload as text); messages that arrived while a
subscription was being confirmed come first. RESP2 arrays and RESP3 pushes are both read. A wait over
the timeout fails with kind `TimedOut` and leaves the connection open. Without a subscription it fails
with `NotSubscribed`. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.11
use std.db.redis

fn main() ! redis.RedisError
    mut c := redis.connect("redis://127.0.0.1")?
    c.subscribe(["news"])?
    m := c.next_message()?
    print("{m.channel}: {m.text()}")
    c.close()
```

**std.redis.12** — `redis.pool(address, size) Pool ! RedisError` opens `size` connections (at least 1).
`p.get() Client ! RedisError` takes one, waiting up to 30 seconds (kind `TimedOut` when none is free
or the pool is closed); `p.put(c)` gives it back; a connection that failed or is subscribed is closed
and replaced by a new one (which can fail with the errors of `connect`). `p.close()` closes the idle
connections; those still out are closed by their holders. At most `size` tasks use the server at once. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.12
use std.db.redis

fn bump(p redis.Pool) ! redis.RedisError
    mut c := p.get()?
    _ := c.incr("hits")?
    p.put(c)?

fn main() ! redis.RedisError
    p := redis.pool("redis://127.0.0.1", 4)?
    bump(p)?
    p.close()
```

**std.redis.13** — A failure is `error struct RedisError { kind RedisKind, message str }`. `RedisKind` is
`BadUrl`, `Auth`, `Server` (an error reply), `Protocol` (not RESP), `BadReply` (RESP, but not the shape
the call needs), `Subscribed`, `NotSubscribed`, `Closed` (the connection is closed or the server went
away), `TimedOut` or `Net` (connect, TLS, read or write failed). After a write fails, or a read fails or
times out, the connection is closed (a late reply would be taken for the next one) and later calls fail
with `Closed`, except a timeout in `next_message`. Not supported: cluster and sentinel, RESP3 client
tracking, streams and blocking commands as typed helpers, a TLS server. *(ADR TBD)*

```ail tool=check,FILE rules=std.redis.13
use std.db.redis

fn main() !
    c := redis.connect("redis://127.0.0.1:1") else err
        print(err.message)
        return
    c.close()
```
