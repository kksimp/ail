#!/usr/bin/env bash
# Live test of std.db.postgres (std.postgres.1-13) against a throwaway local PostgreSQL
# (the distribution's server binaries, no docker): SCRAM-SHA-256, md5 and cleartext
# logins, a refused password, typed rows, bytea, numeric, NULLs, SQLSTATE errors,
# transactions, 2000 rows of 3000 bytes, and the pool. Not a gate suite. From the repo root:
#   tools/interop/postgres-live.sh
# Prints the program's output diffed against tools/interop/postgres_live.out, then
# `RESULT: GREEN` or `RED`. Skips (exit 0) when no server binaries are installed.
set -uo pipefail
cd "$(dirname "$0")/../.."
bin=$(ls -d /usr/lib/postgresql/*/bin 2>/dev/null | sort -V | tail -1)
[ -x "$bin/initdb" ] || { echo "SKIP: no PostgreSQL server binaries"; exit 0; }
tmp=$(mktemp -d)
port=5433
if [ "$(id -u)" = 0 ]; then run() { su postgres -c "$*"; }; chown postgres "$tmp"; else run() { bash -c "$*"; }; fi
cleanup() { run "$bin/pg_ctl -D $tmp/data -m immediate stop" >/dev/null 2>&1; rm -rf "$tmp"; }
trap cleanup EXIT
chmod 755 "$tmp"
run "echo secret > $tmp/pw; $bin/initdb -D $tmp/data -U postgres --auth-local=trust --auth-host=scram-sha-256 --pwfile=$tmp/pw" >/dev/null 2>&1 || { echo "RED: initdb failed"; exit 1; }
printf "host app md5u 127.0.0.1/32 md5\nhost app clearu 127.0.0.1/32 password\n" | cat - "$tmp/data/pg_hba.conf" > "$tmp/hba" && cat "$tmp/hba" > "$tmp/data/pg_hba.conf"
printf "listen_addresses='127.0.0.1'\nport=$port\nunix_socket_directories='$tmp'\nssl=off\n" >> "$tmp/data/postgresql.conf"
run "$bin/pg_ctl -D $tmp/data -l $tmp/log -w start" >/dev/null 2>&1 || { echo "RED: server did not start"; exit 1; }
psql_() { run "$bin/psql -h $tmp -p $port -U postgres -q $*"; }
echo "create database app" | psql_ >/dev/null
printf "create user scramu password 'scpw';\ncreate user clearu password 'clpw';\nset password_encryption='md5';\ncreate user md5u password 'mdpw';\ngrant all on database app to scramu, clearu, md5u;\n" | psql_ >/dev/null
echo "grant all on schema public to scramu, clearu, md5u" | psql_ -d app >/dev/null
out=$(./ail run tools/interop/postgres_live.ail 2>&1)
if diff <(echo "$out") tools/interop/postgres_live.out; then echo "RESULT: GREEN"; else echo "RESULT: RED"; exit 1; fi
