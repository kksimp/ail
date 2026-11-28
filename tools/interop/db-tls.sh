#!/usr/bin/env bash
# Interop of std.db.postgres and std.db.mysql TLS (std.postgres.15, std.mysql.15,
# std.mysql.16) against real PostgreSQL 16 and MySQL 8.0 servers. Not a gate suite.
# Needs root on a Debian or Ubuntu host with the packages `postgresql` and
# `mysql-server` (apt), openssl and python3; it starts its own servers under a
# temporary directory and stops them at the end. Run from the repo root:
#   sudo tools/interop/db-tls.sh
# Prints one line per case (`ok ...` or `BAD ...`) and `RESULT: GREEN` or `RED`.
#
# The clients talk to the servers' own TLS directly (AIL's TLS client offers X25519
# and P-256, std.crypto.11; PostgreSQL and MySQL 8.0 accept only NIST groups). What
# is checked is the AIL side: SSLRequest and the SSL request packet, the handshake,
# the certificate and host name checks, SCRAM, the full caching_sha2_password
# exchange over TLS, the refusal of that exchange on plain TCP, and the queries
# after the upgrade, with the server's own view of the session (pg_stat_ssl,
# Ssl_cipher).
set -uo pipefail
cd "$(dirname "$0")/../.."
[ "$(id -u)" = 0 ] || { echo "run as root (sudo)"; exit 2; }
tmp=$(mktemp -d); chmod 755 "$tmp"
PW=itls-secret-1
PGBIN=$(ls -d /usr/lib/postgresql/*/bin | tail -1)
fail=0
cleanup() {
  su postgres -c "$PGBIN/pg_ctl -D $tmp/pgdata -m immediate stop" >/dev/null 2>&1
  mysqladmin -uroot -p$PW -S "$tmp/run/my.sock" shutdown >/dev/null 2>&1
  kill $(jobs -p) 2>/dev/null
  sleep 1; rm -rf "$tmp"
}
trap cleanup EXIT
mkdir -p "$tmp/certs" "$tmp/pgdata" "$tmp/mydata" "$tmp/run"; chmod 777 "$tmp/run"

# ---- certificates: a CA, and a server certificate for localhost and 127.0.0.1
cd "$tmp/certs"
openssl req -x509 -newkey rsa:2048 -nodes -keyout ca.key -out ca.crt -days 2 -subj "/CN=ail interop CA" 2>/dev/null
openssl req -newkey rsa:2048 -nodes -keyout server.key -out server.csr -subj "/CN=localhost" 2>/dev/null
printf 'subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth\n' > ext.cnf
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 2 -extfile ext.cnf 2>/dev/null
cp server.key pg.key; cp server.key my.key
chown postgres:postgres pg.key "$tmp/pgdata"; chown mysql:mysql my.key "$tmp/mydata"; chmod 600 pg.key my.key
chmod 644 ca.crt server.crt server.key; chmod 755 "$tmp/certs"
cd - >/dev/null

# ---- servers (own data directories; the unix socket is trusted only to create the user, then password logins)
su postgres -c "$PGBIN/initdb -D $tmp/pgdata --auth-host=scram-sha-256 --auth-local=trust >/dev/null"
su postgres -c "$PGBIN/pg_ctl -D $tmp/pgdata -o '-p 55432 -c listen_addresses=127.0.0.1 -c ssl=on -c ssl_cert_file=$tmp/certs/server.crt -c ssl_key_file=$tmp/certs/pg.key -c unix_socket_directories=$tmp/run' -l $tmp/run/pg.log -w start" >/dev/null || { echo "BAD postgres did not start"; cat "$tmp/run/pg.log"; exit 1; }
su postgres -c "psql -h $tmp/run -p 55432 -c \"CREATE USER scram_user PASSWORD '$PW' SUPERUSER\"" >/dev/null || fail=1
sed -i 's/^local\(.*\)trust/local\1scram-sha-256/' "$tmp/pgdata/pg_hba.conf"; su postgres -c "$PGBIN/pg_ctl -D $tmp/pgdata reload" >/dev/null
mysqld --initialize-insecure --user=mysql --datadir="$tmp/mydata" >/dev/null 2>&1
(mysqld --user=mysql --datadir="$tmp/mydata" --port=53306 --bind-address=127.0.0.1 --socket="$tmp/run/my.sock" --mysqlx=OFF \
  --ssl-ca="$tmp/certs/ca.crt" --ssl-cert="$tmp/certs/server.crt" --ssl-key="$tmp/certs/my.key" > "$tmp/run/my.log" 2>&1 &)
for _ in $(seq 1 60); do mysql -uroot -S "$tmp/run/my.sock" -e 'select 1' >/dev/null 2>&1 && break; sleep 1; done
mysql -uroot -S "$tmp/run/my.sock" -e "ALTER USER 'root'@'localhost' IDENTIFIED BY '$PW';
  CREATE USER 'sha'@'%' IDENTIFIED BY '$PW'; CREATE USER 'sha2'@'%' IDENTIFIED BY '$PW'; CREATE USER 'sha3'@'%' IDENTIFIED BY '$PW';
  CREATE USER 'nat'@'%' IDENTIFIED WITH mysql_native_password BY '$PW';
  GRANT ALL ON *.* TO 'sha'@'%'; GRANT ALL ON *.* TO 'sha2'@'%'; GRANT ALL ON *.* TO 'sha3'@'%'; GRANT ALL ON *.* TO 'nat'@'%';" || { echo "BAD mysql setup"; tail -5 "$tmp/run/my.log"; exit 1; }

./ail build tools/interop/db_tls.ail -o "$tmp/db_tls" > "$tmp/build.out" 2>&1 || { grep -A6 'error' "$tmp/build.out"; echo "RESULT: RED (build)"; exit 1; }

# run LABEL EXPECT ARGS: it must print `ok ...`; with EXPECT `fail` the connection must fail with kind Tls.
run() {
  label=$1; shift
  out=$(SSL_CERT_FILE="$tmp/certs/ca.crt" timeout 60 "$tmp/db_tls" "$@" 2>&1); rc=$?
  echo "$label: $out"
  if [ $rc -ne 0 ] || [ "${out#ok}" = "$out" ]; then fail=1; fi
}
PG=postgres://scram_user:$PW@localhost
run "pg  require (SCRAM over TLS)"       pg "$PG:55432/postgres?sslmode=require" tls
run "pg  prefer (the default)"           pg "$PG:55432/postgres" tls
run "pg  verify-full"                    pg "$PG:55432/postgres?sslmode=verify-full" tls
run "pg  by 127.0.0.1 (IP SAN)"          pg "postgres://scram_user:$PW@127.0.0.1:55432/postgres?sslmode=require" tls
run "pg  disable, direct"                pg "$PG:55432/postgres?sslmode=disable" plain
out=$(timeout 60 "$tmp/db_tls" pg "$PG:55432/postgres?sslmode=prefer" fail 2>&1); echo "pg  untrusted CA, prefer: $out"; [ "${out#ok}" != "$out" ] || fail=1
MY=mysql://sha:$PW@localhost:53306/mysql
run "my  require, caching_sha2 full exchange" my "mysql://sha3:$PW@localhost:53306/mysql?ssl-mode=required" tls
run "my  require again (cached, fast path)"   my "mysql://sha3:$PW@localhost:53306/mysql?ssl-mode=required" tls
run "my  root, prefer"                   my "mysql://root:$PW@localhost:53306/mysql" tls
run "my  native password, require"       my "mysql://nat:$PW@localhost:53306/mysql?ssl-mode=required" tls
run "my  disable, direct"                my "mysql://nat:$PW@localhost:53306/mysql?ssl-mode=disabled" plain
out=$(timeout 60 "$tmp/db_tls" my "$MY?ssl-mode=required" fail 2>&1); echo "my  untrusted CA, require: $out"; [ "${out#ok}" != "$out" ] || fail=1
run "my  uncached sha2 over plain TCP must refuse naming TLS" my "mysql://sha2:$PW@localhost:53306/mysql?ssl-mode=disable" refuse
if [ $fail -eq 0 ]; then echo "RESULT: GREEN"; else echo "RESULT: RED"; exit 1; fi
