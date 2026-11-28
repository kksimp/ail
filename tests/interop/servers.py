#!/usr/bin/env python3
"""AIL std interop checks that need a real server: Redis, PostgreSQL, MySQL and MariaDB in docker.

Run through run.sh (modules: redis postgres mysql mariadb). Environment: BIN (built cli_<module>
programs; mariadb uses cli_mysql), WORK, OUT. Appends to OUT/results.tsv: module, case, PASS|FAIL|SKIP, detail.
Every case runs one `cli_<module> URL CASE` and compares its stdout lines with the expected lines; a line that
starts with `re:` is a full-match regex. The data is then read back with the server's own client (redis-cli, psql,
mysql) so AIL's writes are checked by the reference and the reference's writes are read by AIL.
"""
import os, re, subprocess, sys, time

BIN = os.environ["BIN"]; WORK = os.environ["WORK"]; OUT = os.environ["OUT"]
ONLY = set(sys.argv[1:])
rows = []
HOST = "127.0.0.1"
LOCAL = os.environ.get("LOCAL_SERVERS") == "1"   # servers installed on this host, no docker (the cloud container)
PROCS = {}
DOCKER = os.environ.get("DOCKER", "docker").split()   # "sudo docker" when the user is not in the docker group

def rec(mod, case, ok, detail=""):
    status = "PASS" if ok is True else "SKIP" if ok is None else "FAIL"
    rows.append((mod, case, status, detail.replace("\n", " ")[:400]))
    print("%-8s %-4s %s %s" % (mod, status, case, "" if ok is True else detail.replace("\n", " ")[:200]), flush=True)

def run(cmd, inp=None, timeout=180):
    p = subprocess.run(cmd, input=inp, capture_output=True, timeout=timeout)
    return p.returncode, p.stdout.decode(errors="replace"), p.stderr.decode(errors="replace")

def xcmd(name, cmd, env=()):
    """A command run where the server runs: in its container, or on this host in LOCAL mode."""
    if LOCAL: return ["env", *env, *cmd]
    return [*DOCKER, "exec", *[x for e in env for x in ("-e", e)], name, *cmd]

def docker(*args, timeout=180): return run([*DOCKER, *args], timeout=timeout)

def cli_lines(exe, url, case, *extra, timeout=240):
    p = subprocess.run([exe, url, case, *map(str, extra)], capture_output=True, timeout=timeout)
    return p.stdout.decode(errors="replace").splitlines(), p.returncode, p.stderr.decode(errors="replace")

def lines_match(got, want):
    if len(got) != len(want): return False
    for g, w in zip(got, want):
        if w.startswith("re:"):
            if not re.fullmatch(w[3:], g): return False
        elif g != w: return False
    return True

def diff(got, want):
    for i in range(max(len(got), len(want))):
        g = got[i] if i < len(got) else "<missing>"; w = want[i] if i < len(want) else "<missing>"
        if not (w.startswith("re:") and re.fullmatch(w[3:], g)) and g != w:
            return "line %d: got %r want %r" % (i + 1, g, w)
    return ""

def case(mod, exe, name, url, want, *extra, label=None):
    label = label or name
    try:
        got, rc, err = cli_lines(exe, url, name, *extra)
    except subprocess.TimeoutExpired:
        rec(mod, label, False, "timed out"); return None
    ok = lines_match(got, want)
    rec(mod, label, ok, "" if ok else diff(got, want) + ((" rc=%d stderr=%s" % (rc, err[:160])) if rc else ""))
    return got

def check(mod, label, got, want):
    rec(mod, label, got == want, "" if got == want else "got %r want %r" % (got, want))

def wait_for(fn, what, tries=90):
    for _ in range(tries):
        try:
            if fn(): return True
        except Exception:
            pass
        time.sleep(1)
    return False

def have_docker():
    try: return docker("version", "--format", "{{.Server.Version}}")[0] == 0
    except Exception: return False

def start_local_redis(name, port, conf):
    path = os.path.join(WORK, "redis-%d.conf" % port)
    with open(path, "w") as f: f.write("port %d\nbind 127.0.0.1\nsave \"\"\n%s" % (port, conf))
    PROCS[name] = subprocess.Popen(["redis-server", path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return True, ""

def start_local_mariadb(name, port):
    d = os.path.join(WORK, "mariadb-data"); sock = os.path.join(d, "m.sock")
    run(["rm", "-rf", d]); run(["install", "-d", "-o", "mysql", "-g", "mysql", d])
    rc_, out, err = run(["mariadb-install-db", "--user=mysql", "--datadir=" + d, "--auth-root-authentication-method=normal", "--skip-test-db"])
    if rc_: return False, err[:200]
    PROCS[name] = subprocess.Popen(["mariadbd", "--user=mysql", "--datadir=" + d, "--port=%d" % port, "--bind-address=127.0.0.1", "--socket=" + sock,
                                    "--max-allowed-packet=64M", "--pid-file=" + os.path.join(d, "m.pid")], stdout=subprocess.DEVNULL, stderr=open(os.path.join(WORK, "mariadb.log"), "w"))
    for _ in range(60):
        if run(["mariadb", "--socket=" + sock, "-uroot", "-e", "SELECT 1"])[0] == 0: break
        time.sleep(1)
    run(["mariadb", "--socket=" + sock, "-uroot", "-e", "DELETE FROM mysql.global_priv WHERE User = ''; FLUSH PRIVILEGES; ALTER USER root@localhost IDENTIFIED BY 'rootpw'; CREATE DATABASE shop"])
    return True, ""

def start(name, image, port_map, args=(), env=()):
    docker("rm", "-f", name)
    cmd = ["run", "-d", "--name", name, "-p", port_map]
    for e in env: cmd += ["-e", e]
    cmd += [image, *args]
    rc, out, err = docker(*cmd, timeout=900)
    return rc == 0, err.strip()[:200]

def stop(name):
    if LOCAL:
        p = PROCS.pop(name, None)
        if p: p.kill()
        if name == "ail-interop-pg": run(["pg_dropcluster", "--stop", "16", "interop"])
        return
    docker("rm", "-f", "-v", name)

def need(mod, exe_name=None):
    exe = os.path.join(BIN, "cli_" + (exe_name or mod))
    if not os.path.exists(exe):
        rec(mod, "build", False, "cli_%s did not build (see build log)" % (exe_name or mod)); return None
    if not LOCAL and not have_docker():
        rec(mod, "docker", None, "docker is not available on this host"); return None
    return exe

# ------------------------------------------------------------------ redis
def t_redis():
    mod = "redis"; exe = need(mod)
    if not exe: return
    name, port = "ail-interop-redis", 16379
    conf = "printf 'requirepass secret\\nuser alice on >wonder ~* &* +@all\\n' > /tmp/r.conf && exec redis-server /tmp/r.conf"
    rport = port if LOCAL else 6379
    if LOCAL: ok, err = start_local_redis(name, port, "requirepass secret\nuser alice on >wonder ~* &* +@all\n")
    else: ok, err = start(name, "redis:7", "127.0.0.1:%d:6379" % port, ["sh", "-c", conf])
    if not ok: rec(mod, "docker run", False, err); return
    try:
        def rc(*a): return run(xcmd(name, ["redis-cli", "-p", str(rport), "-a", "secret", "--no-auth-warning", *a]))[1].rstrip("\n")
        if not wait_for(lambda: rc("ping") == "PONG", "redis"): rec(mod, "start", False, "redis did not start"); return
        base = "redis://secret@%s:%d" % (HOST, port)
        def fresh(): rc("FLUSHALL")
        # auth: password for the default user, an ACL user, a wrong password, no password
        case(mod, exe, "auth", base, ["ok"], label="auth default user password")
        case(mod, exe, "auth", "redis://alice:wonder@%s:%d" % (HOST, port), ["ok"], label="auth ACL user")
        case(mod, exe, "auth", "redis://alice:nope@%s:%d" % (HOST, port), ["connect Auth"], label="auth wrong password")
        case(mod, exe, "auth", "redis://%s:%d" % (HOST, port), ["ping Auth"], label="auth missing password")
        case(mod, exe, "auth", base + "/3", ["ok"], label="auth select db 3")
        case(mod, exe, "auth", "redis://x@%s:1" % HOST, ["re:connect (Net|Closed)"], label="connection refused")
        fresh(); rc("SET", "i:fromcli", "from-cli")
        case(mod, exe, "strings", base, [
            "from-cli from-cli", "get héllo €", "strlen 10", "missing none", "incr 1", "incr_by 42", "decr 41",
            "append 2", "append 4", "nx true", "nx false", "nx_get first", "ttl_ok true", "persist true", "ttl -1",
            "expire true", "exists true", "del 2", "exists false", "bin 00ff010d0a80", "big 200000", "big_get 200000"])
        check(mod, "strings read back by redis-cli", [rc("GET", "i:s"), rc("GET", "i:ap"), rc("GET", "i:n"), rc("STRLEN", "i:big"), rc("--no-raw", "GET", "i:bin")],
              ["héllo €", "abcd", "41", "200000", '"\\x00\\xff\\x01\\r\\n\\x80"'])
        fresh()
        case(mod, exe, "hashes", base, ["hset 1", "hset_all 2", "hget v2", "hget_missing none", "hgetall 3 v1 v2 3", "hincr 10",
             "hexists true false", "hkeys f1,f2,f3", "hlen 3", "hdel 1", "hlen 2"])
        check(mod, "hashes read back by redis-cli", rc("HGET", "i:h", "f3"), "10")
        fresh()
        case(mod, exe, "lists", base, ["rpush 3", "lpush 4", "lrange z,a,b,c", "lpop z", "rpop c", "llen 2", "lrange a", "empty_pop none"])
        check(mod, "lists read back by redis-cli", rc("LRANGE", "i:l", "0", "-1").split("\n"), ["a", "b"])
        fresh()
        case(mod, exe, "sets", base, ["sadd 3", "scard 3", "sismember true false", "srem 1", "smembers b,c"])
        fresh()
        case(mod, exe, "zsets", base, ["zadd 1", "zadd 1", "zadd 1", "zrange one,two,three", "zrank 1", "zrank_missing none", "zscore 1.5",
             r"re:zincr 2(\.0)?", r"re:scored one 2(\.0)?", r"re:scored two 2(\.0)?", r"re:scored three 3(\.0)?", "zcard 3", "zrem 1"])
        check(mod, "zset read back by redis-cli", rc("ZRANGE", "i:z", "0", "-1", "WITHSCORES").split("\n"), ["two", "2", "three", "3"])
        fresh()
        case(mod, exe, "keys", base, ["keys 25", "scan_done true", "scan_seen_ge true"])
        fresh()
        case(mod, exe, "pipeline", base, ["queued 203", "replies 203", "last3 199 true 6"])
        check(mod, "pipeline read back by redis-cli", [rc("GET", "i:p0"), rc("GET", "i:p5"), rc("DBSIZE")], ["0", "6", "200"])
        fresh()
        case(mod, exe, "transaction", base, ["tx OK", "tx 2", "tx 2", "tx2 false OK", "tx2 true none", "tx2 false text", "aborted Server", "not_applied none"])
        fresh()
        case(mod, exe, "resp3", base + "/0?protocol=3", ["proto 3", "hgetall_items 2", "hgetall_first a 1", "nil true"])
        fresh()
        case(mod, exe, "pubsub_self", base, ["receivers 1", "receivers_none 0", "msg i:self hello", "while_subscribed Subscribed"])
        fresh()
        case(mod, exe, "pool", base, ["pool_count 6", "closed"])
        # pub/sub with the reference publishing
        fresh()
        p = subprocess.Popen([exe, base, "sub", "3"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        time.sleep(4)
        for ch, msg in (("i:ch1", "hello"), ("i:pat.x", "world"), ("i:ch2", "héllo")): rc("PUBLISH", ch, msg)
        try:
            out, err = p.communicate(timeout=40)
            got = out.decode(errors="replace").splitlines()
            want = ["subs 3", "msg false i:ch1  hello", "msg true i:pat.x i:pat.* world", "msg false i:ch2  héllo", "subs 0", "command_again ok"]
            ok = lines_match(got, want)
            rec(mod, "pubsub published by redis-cli", ok, "" if ok else diff(got, want))
        except subprocess.TimeoutExpired:
            p.kill(); rec(mod, "pubsub published by redis-cli", False, "timed out waiting for messages")
        # the reference reads what AIL publishes: redis-cli subscribes, AIL publishes and sees one receiver
        sub = subprocess.Popen(xcmd(name, ["timeout", "8", "redis-cli", "-p", str(rport), "-a", "secret", "--no-auth-warning", "SUBSCRIBE", "i:out"]), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        time.sleep(3)
        case(mod, exe, "publish", base, ["receivers 1"], label="publish to a redis-cli subscriber")
        try: out = sub.communicate(timeout=20)[0].decode(errors="replace")
        except subprocess.TimeoutExpired: sub.kill(); out = ""
        rec(mod, "redis-cli received the message", True if "from-ail" in out else None, "redis-cli output not captured" if "from-ail" not in out else "")
    finally:
        stop(name)

# ------------------------------------------------------------------ postgres
def t_postgres():
    mod = "postgres"; exe = need(mod)
    if not exe: return
    name, port = "ail-interop-pg", 15432
    pgport = port if LOCAL else 5432
    if LOCAL:
        rc_, _o, err = run(["pg_createcluster", "16", "interop", "-p", str(port), "--start"])
        ok = rc_ == 0
        if ok: run(["runuser", "-u", "postgres", "--", "psql", "-p", str(port), "-c", "ALTER USER postgres PASSWORD 'pgsecret'"]); run(["sh", "-c", "printf \"listen_addresses = '127.0.0.1'\\nssl = off\\n\" >> /etc/postgresql/16/interop/postgresql.conf"]); run(["pg_ctlcluster", "16", "interop", "restart"])
    else: ok, err = start(name, "postgres:16", "127.0.0.1:%d:5432" % port, env=["POSTGRES_PASSWORD=pgsecret"])
    if not ok: rec(mod, "docker run", False, err); return
    try:
        def psql(sql, user="postgres", db="postgres", flags=("-At",)):
            return run(xcmd(name, ["psql", "-p", str(pgport), "-U", user, "-h", "127.0.0.1", "-d", db, "-v", "ON_ERROR_STOP=1", *flags, "-c", sql], env=["PGPASSWORD=pgsecret"]))
        if not wait_for(lambda: psql("select 1")[0] == 0, "pg"): rec(mod, "start", False, "postgres did not start"); return
        setup = ["CREATE DATABASE shop", "CREATE ROLE u_trust LOGIN", "CREATE ROLE u_clear LOGIN PASSWORD 'clearpw'",
                 "SET password_encryption='md5'; CREATE ROLE u_md5 LOGIN PASSWORD 'md5pw'",
                 "SET password_encryption='scram-sha-256'; CREATE ROLE u_scram LOGIN PASSWORD 'scrampw'"]
        for s in setup:
            rc_, out, err = psql(s)
            if rc_: rec(mod, "setup", False, s + ": " + err[:150]); return
        hbafile = "/etc/postgresql/16/interop/pg_hba.conf" if LOCAL else "$PGDATA/pg_hba.conf"
        hba = "printf 'host all u_trust all trust\\nhost all u_clear all password\\nhost all u_md5 all md5\\nhost all u_scram all scram-sha-256\\n' | cat - " + hbafile + " > /tmp/hba && cat /tmp/hba > " + hbafile
        run(xcmd(name, ["sh", "-c", hba])); psql("SELECT pg_reload_conf()")
        def url(u, pw="", db="shop", q="?sslmode=disable"): return "postgres://%s%s@%s:%d/%s%s" % (u, (":" + pw) if pw else "", HOST, port, db, q)
        case(mod, exe, "auth", url("u_trust"), ["ok u_trust 2"], label="auth trust")
        case(mod, exe, "auth", url("u_clear", "clearpw"), ["ok u_clear 2"], label="auth cleartext password")
        case(mod, exe, "auth", url("u_md5", "md5pw"), ["ok u_md5 2"], label="auth md5")
        case(mod, exe, "auth", url("u_scram", "scrampw"), ["ok u_scram 2"], label="auth SCRAM-SHA-256")
        case(mod, exe, "auth", url("postgres", "pgsecret"), ["ok postgres 2"], label="auth SCRAM-SHA-256 superuser")
        case(mod, exe, "auth", url("u_scram", "wrong"), [r"re:connect Auth( 28P01)?"], label="auth SCRAM wrong password")
        case(mod, exe, "auth", url("u_md5", "wrong"), [r"re:connect Auth( 28P01)?"], label="auth md5 wrong password")
        case(mod, exe, "auth", url("u_clear", "wrong"), [r"re:connect Auth( 28P01)?"], label="auth cleartext wrong password")
        case(mod, exe, "auth", url("u_scram", "scrampw", "nosuchdb"), [r"re:connect (Server|Auth) 3D000"], label="auth missing database")
        case(mod, exe, "auth", "postgres://u@%s:1/x" % HOST, [r"re:connect (Net|Closed) .*"], label="connection refused")
        pg = url("postgres", "pgsecret")
        case(mod, exe, "types", pg, ["rows 1", "int4 42", "int8 9000000000", "float8 2.5", "bool true", "text héllo €", "bytea deadbeef", "null true",
             "numeric 1.25 1.25", "timestamp 2024-01-02 03:04:05", 'jsonb {"a": 1}', "uuid a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11", "array {1,2,3}", "int2 7", "float4 1.5", "false false"])
        case(mod, exe, "crud", pg, ["insert 1", "insert_many 3", "rows 4", "first 1 O'Brien; DROP TABLE i_items; -- 3 2.5 true 0001ff null=true", "unicode héllo €",
             "none_match 0", "update 3", "delete 1", "dup Server 23505", "missing_table Server 42P01", "syntax Server 42601", "div0 Server 22012", "still_usable 5",
             "big_param 200000", "many_rows 5000 5000", "closed false"])
        check(mod, "crud read back by psql", psql("SELECT name || '|' || qty FROM i_items ORDER BY id", db="shop")[1].strip().split("\n"), ["O'Brien; DROP TABLE i_items; --|3", "b|11", "héllo €|21"])
        check(mod, "crud bytea and nulls read by psql", psql("SELECT encode(pic,'hex') || '|' || price || '|' || (note IS NULL) FROM i_items WHERE id = 1", db="shop")[1].strip(), "0001ff|2.5|true")
        # the reference writes, AIL reads
        psql("CREATE TABLE IF NOT EXISTS i_ref (id int, t text, n numeric, b bytea, j jsonb, ts timestamptz, a int[])", db="shop")
        psql("INSERT INTO i_ref VALUES (1, 'ref-written €', 12.50, '\\x0102', '{\"k\": [1, 2]}', '2024-03-04 05:06:07+00', '{4,5}')", db="shop")
        case(mod, exe, "reference", pg, ['1 ref-written € 12.50 0102 {"k": [1, 2]} 2024-03-04 05:06:07+00 {4,5}'], label="rows written by psql, read by AIL")
        case(mod, exe, "tx", pg, ["in_tx false", "in_tx true", "in_tx false", "after_rollback 0", "after_commit 1", "dup_in_tx Server 23505", "aborted_tx Server 25P02", "in_tx true", "final 1"])
        check(mod, "tx read back by psql", psql("SELECT string_agg(n::text, ',') FROM i_tx", db="shop")[1].strip(), "2")
        case(mod, exe, "pool", pg, ["exhausted Usage", "pool_query 7", "pool_exec 1", "fresh_after_open_tx false", "closed"])
        case(mod, exe, "config", "postgres://alice:p%40w@db.example:5433/shop", ["db.example 5433 alice shop"])
        # TLS: the server offers it (ssl on) and the default sslmode is prefer
        if LOCAL:
            psql("ALTER SYSTEM SET ssl = 'on'"); psql("SELECT pg_reload_conf()"); time.sleep(1)
        case(mod, exe, "auth", url("u_scram", "scrampw", q=""), ["ok u_scram 2"], label="auth over TLS (sslmode prefer, server offers TLS)")
        # connections left open by the pool case do not linger
        n = psql("SELECT count(*) FROM pg_stat_activity WHERE datname = 'shop' AND pid <> pg_backend_pid()")[1].strip()
        rec(mod, "no connections leaked", n == "0", "pg_stat_activity shows %s open" % n)
    finally:
        stop(name)

# ------------------------------------------------------------------ mysql and mariadb
def t_mysql_like(mod, image, client, port, env, args, native_sql, with_sha2):
    exe = need(mod, "mysql")
    if not exe: return
    name = "ail-interop-" + mod
    mport = port if LOCAL else 3306
    if LOCAL:
        ok, err = start_local_mariadb(name, port)
    else: ok, err = start(name, image, "127.0.0.1:%d:3306" % port, args=args, env=env)
    if not ok: rec(mod, "docker run", False, err); return
    try:
        def sql(q, user="root", pw="rootpw", db=None):
            cmd = [client, "-h127.0.0.1", "-P", str(mport), "-u" + user, "-p" + pw, "--batch", "--skip-column-names", "--default-character-set=utf8mb4"]
            if db: cmd += ["-D", db]
            return run(xcmd(name, [*cmd, "-e", q]))
        if not wait_for(lambda: sql("SELECT 1")[0] == 0, "mysql", tries=180): rec(mod, "start", False, "server did not start"); return
        for q in native_sql + ["CREATE USER 'u_empty'@'%'", "GRANT ALL ON shop.* TO 'u_native'@'%', 'u_empty'@'%'"]:
            rc_, out, err = sql(q)
            if rc_: rec(mod, "setup", False, q + ": " + err[:150]); return
        def url(u, pw="", db="shop", q="?ssl-mode=disabled"): return "mysql://%s%s@%s:%d/%s%s" % (u, (":" + pw) if pw else "", HOST, port, db, q)
        case(mod, exe, "auth", url("u_native", "nativepw"), ["ok 2"], label="auth mysql_native_password")
        case(mod, exe, "auth", url("u_native", "wrong"), ["connect Auth"], label="auth wrong password")
        case(mod, exe, "auth", url("u_empty"), ["ok 2"], label="auth no password")
        case(mod, exe, "auth", url("u_native", "nativepw", "nosuchdb"), [r"re:connect (Server|Auth)"], label="auth missing database")
        case(mod, exe, "auth", "mysql://u@%s:1/x" % HOST, [r"re:connect (Net|Closed)"], label="connection refused")
        if with_sha2:
            sql("CREATE USER 'u_sha2'@'%' IDENTIFIED WITH caching_sha2_password BY 'sha2pw'"); sql("GRANT ALL ON shop.* TO 'u_sha2'@'%'")
            case(mod, exe, "auth", url("u_sha2", "sha2pw"), ["connect Auth"], label="auth caching_sha2 cold cache (full exchange needs TLS: a clean Auth error)")
            case(mod, exe, "auth", url("u_sha2", "sha2pw", q=""), ["connect Tls"], label="auth caching_sha2 over TLS with the server's own certificate (no host name, not trusted: a clean Tls error; the full exchange is checked by tools/interop/db-tls.sh)")
            sql("SELECT 1", "u_sha2", "sha2pw")   # the reference client logs in once and fills the server's cache
            case(mod, exe, "auth", url("u_sha2", "sha2pw"), ["ok 2"], label="auth caching_sha2 fast path")
            case(mod, exe, "auth", url("root", "rootpw"), ["ok 2"], label="auth root (caching_sha2, cached)")
        else:
            case(mod, exe, "auth", url("root", "rootpw"), ["ok 2"], label="auth root")
        mp = url("u_native", "nativepw")
        types = ["rows 1", "int 42", "bigint 9000000000", "double 2.5", "bool true", "text héllo €", "blob deadbeef", "null true", "decimal 1.25",
                 "datetime 2024-01-02 03:04:05", re_json(), "float 1.5", "unsigned_max 18446744073709551615", "negative -7"]
        case(mod, exe, "types_text", mp, types, label="types, text protocol")
        case(mod, exe, "types_bin", mp, types, label="types, binary protocol (prepared)")
        case(mod, exe, "crud", mp, ["insert 1 last_id 1", "insert_many 3 last_id 2", "rows 4", "first 1 O'Brien; DROP TABLE i_items; -- 3 2.5 true 0001ff null=true", "unicode héllo €",
             "none_match 0", "update 3", "delete 1", "dup Server 1062 23000", "missing_table Server 1146 42S02", "syntax Server 1064 42000", "re:range Server 1264 .*",
             "still_usable 5", "big_param 200000", "ping ok", "stmt params 1 columns 2", "stmt_run 3", "stmt_run 1", "closed false"])
        check(mod, "crud read back by the mysql client", sql("SELECT CONCAT(name, '|', qty) FROM i_items ORDER BY id", "u_native", "nativepw", "shop")[1].strip().split("\n"),
              ["O'Brien; DROP TABLE i_items; --|3", "b|11", "héllo €|21"])
        check(mod, "crud blob read by the mysql client", sql("SELECT CONCAT(HEX(pic), '|', price, '|', note IS NULL) FROM i_items WHERE id = 1", "u_native", "nativepw", "shop")[1].strip(), "0001FF|2.5|1")
        jtype = "JSON" if with_sha2 else "TEXT"
        sql("DROP TABLE IF EXISTS i_ref; CREATE TABLE i_ref (id INT, t TEXT, d DECIMAL(6,2), b BLOB, j %s, ts DATETIME) CHARACTER SET utf8mb4" % jtype, "u_native", "nativepw", "shop")
        sql("INSERT INTO i_ref VALUES (1, 'ref-written €', 12.50, X'0102', '{\"k\": [1, 2]}', '2024-03-04 05:06:07')", "u_native", "nativepw", "shop")
        refline = '1 ref-written € 12.50 0102 {"k": [1, 2]} 2024-03-04 05:06:07'
        case(mod, exe, "reference", mp, ["text " + refline, "bin " + refline], label="rows written by the mysql client, read by AIL")
        case(mod, exe, "many", mp, ["many_rows 2000 1999", "wide 17000000"])
        case(mod, exe, "tx", mp, ["in_tx false", "in_tx true", "in_tx false", "after_rollback 0", "after_commit 1", "dup_in_tx Server 1062 23000", "in_tx true", "final 1"])
        check(mod, "tx read back by the mysql client", sql("SELECT GROUP_CONCAT(n) FROM i_tx", "u_native", "nativepw", "shop")[1].strip(), "2")
        case(mod, exe, "pool", mp, ["exhausted Usage", "pool_query 7", "pool_exec 0", "fresh_after_open_tx false", "closed"])
        n = sql("SELECT COUNT(*) FROM information_schema.processlist WHERE user = 'u_native'")[1].strip()
        rec(mod, "no connections leaked", n in ("0", "1"), "processlist shows %s u_native sessions" % n)
    finally:
        stop(name)

def re_json(): return 'json {"a": 1}'

def t_mysql():
    t_mysql_like("mysql", "mysql:8.4", "mysql", 13306, ["MYSQL_ROOT_PASSWORD=rootpw", "MYSQL_DATABASE=shop", "MYSQL_ROOT_HOST=%"],
                 ["--mysql-native-password=ON", "--max-allowed-packet=64M"],
                 ["CREATE USER 'u_native'@'%' IDENTIFIED WITH mysql_native_password BY 'nativepw'"], True)

def t_mariadb():
    t_mysql_like("mariadb", "mariadb:11", "mariadb", 13307, ["MARIADB_ROOT_PASSWORD=rootpw", "MARIADB_DATABASE=shop", "MARIADB_ROOT_HOST=%"],
                 ["--max-allowed-packet=64M"],
                 ["CREATE USER 'u_native'@'%' IDENTIFIED BY 'nativepw'"], False)

MODULES = {"redis": t_redis, "postgres": t_postgres, "mysql": t_mysql, "mariadb": t_mariadb}
for name, fn in MODULES.items():
    if ONLY and name not in ONLY: continue
    t0 = time.time()
    try: fn()
    except Exception as e: rec(name, "harness", False, "exception %r" % (e,))
    print("%-10s %.0fs" % (name, time.time() - t0), flush=True)
os.makedirs(OUT, exist_ok=True)
with open(os.path.join(OUT, "results-servers.tsv"), "w") as f:
    for row in rows: f.write("\t".join(row) + "\n")
for m in [m for m in MODULES if any(r[0] == m for r in rows)]:
    c = {s: sum(1 for r in rows if r[0] == m and r[2] == s) for s in ("PASS", "FAIL", "SKIP")}
    print("%-10s PASS %-4d FAIL %-4d SKIP %d" % (m, c["PASS"], c["FAIL"], c["SKIP"]))
    for r in rows:
        if r[0] == m and r[2] == "FAIL": print("   FAIL %s: %s" % (r[1], r[3]))
sys.exit(1 if any(r[2] == "FAIL" for r in rows) else 0)
