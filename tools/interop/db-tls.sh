#!/usr/bin/env bash
# Interop of std.db.postgres and std.db.mysql TLS (std.postgres.15, std.mysql.15,
# std.mysql.16) against real servers in containers: PostgreSQL 16, MySQL 8.4 and
# MariaDB 11, each with a certificate for `localhost` from a throwaway CA that the
# client trusts through SSL_CERT_FILE. Not a gate suite. Needs docker, openssl and
# sudo (the key files are owned by the container's database user). Run on
# ail-linux from the repo root: tools/interop/db-tls.sh
# Prints one line per case (`ok ...` or `BAD ...`) and `RESULT: GREEN` or `RED`.
set -uo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
names="ail-itls-pg ail-itls-my ail-itls-ma"
cleanup() { docker rm -f $names >/dev/null 2>&1 || true; sudo rm -rf "$tmp"; }
trap cleanup EXIT
fail=0
PW=itls-secret-1

# ---- certificates: a CA, and a server certificate for localhost and 127.0.0.1
openssl req -x509 -newkey rsa:2048 -nodes -keyout "$tmp/ca.key" -out "$tmp/ca.crt" -days 2 -subj "/CN=ail interop CA" >/dev/null 2>&1
openssl req -newkey rsa:2048 -nodes -keyout "$tmp/server.key" -out "$tmp/server.csr" -subj "/CN=localhost" >/dev/null 2>&1
printf 'subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth\n' > "$tmp/ext.cnf"
openssl x509 -req -in "$tmp/server.csr" -CA "$tmp/ca.crt" -CAkey "$tmp/ca.key" -CAcreateserial -out "$tmp/server.crt" -days 2 -extfile "$tmp/ext.cnf" >/dev/null 2>&1
mkdir -p "$tmp/certs"
cp "$tmp/ca.crt" "$tmp/server.crt" "$tmp/server.key" "$tmp/certs/"
sudo chown -R 999:999 "$tmp/certs"
sudo chmod 600 "$tmp/certs/server.key"
sudo chmod 644 "$tmp/certs/ca.crt" "$tmp/certs/server.crt"
echo "== certificates made"

# ---- servers
docker run -d --name ail-itls-pg -e POSTGRES_PASSWORD=$PW -p 127.0.0.1:55432:5432 -v "$tmp/certs:/certs:ro" postgres:16 \
  -c ssl=on -c ssl_cert_file=/certs/server.crt -c ssl_key_file=/certs/server.key >/dev/null
docker run -d --name ail-itls-my -e MYSQL_ROOT_PASSWORD=$PW -p 127.0.0.1:53306:3306 -v "$tmp/certs:/certs:ro" mysql:8.4 \
  --ssl-ca=/certs/ca.crt --ssl-cert=/certs/server.crt --ssl-key=/certs/server.key >/dev/null
docker run -d --name ail-itls-ma -e MARIADB_ROOT_PASSWORD=$PW -p 127.0.0.1:53307:3306 -v "$tmp/certs:/certs:ro" mariadb:11 \
  --ssl-ca=/certs/ca.crt --ssl-cert=/certs/server.crt --ssl-key=/certs/server.key >/dev/null

./ail build tools/interop/db_tls.ail -o "$tmp/db_tls" > "$tmp/build.out" 2>&1 || { cat "$tmp/build.out"; echo "RESULT: RED (build)"; exit 1; }

wait_for() { # name, command inside the container
  for _ in $(seq 1 90); do docker exec "$1" sh -c "$2" >/dev/null 2>&1 && return 0; sleep 2; done
  echo "BAD $1 did not come up"; docker logs --tail 20 "$1"; return 1
}
wait_for ail-itls-pg "pg_isready -h 127.0.0.1 -U postgres && psql -U postgres -h 127.0.0.1 -c 'select 1'" || fail=1
wait_for ail-itls-my "mysql -h 127.0.0.1 -uroot -p$PW -e 'select 1'" || fail=1
wait_for ail-itls-ma "mariadb -h 127.0.0.1 -uroot -p$PW -e 'select 1'" || fail=1
# MariaDB: a native-password user; MySQL 8.4: a caching_sha2_password user (the default plugin), not cached yet.
docker exec ail-itls-ma mariadb -uroot -p$PW -e "CREATE USER 'nat'@'%' IDENTIFIED VIA mysql_native_password USING PASSWORD('$PW'); GRANT ALL ON *.* TO 'nat'@'%';" >/dev/null 2>&1 || { echo "BAD mariadb user"; fail=1; }
docker exec ail-itls-my mysql -uroot -p$PW -e "CREATE USER 'sha'@'%' IDENTIFIED BY '$PW'; GRANT ALL ON *.* TO 'sha'@'%';" >/dev/null 2>&1 || { echo "BAD mysql user"; fail=1; }
docker exec ail-itls-pg psql -U postgres -h 127.0.0.1 -c "CREATE USER scram_user PASSWORD '$PW' SUPERUSER" >/dev/null 2>&1 || { echo "BAD pg user"; fail=1; }

run() { # label, program args...
  label=$1; shift
  out=$(SSL_CERT_FILE="$tmp/ca.crt" timeout 60 "$tmp/db_tls" "$@" 2>&1); rc=$?
  echo "$label: $out"
  if [ $rc -ne 0 ] || [ "${out#ok}" = "$out" ]; then fail=1; fi
}
PG=postgres://scram_user:$PW@localhost:55432/postgres
run "pg   require (SCRAM over TLS)"    pg "$PG?sslmode=require" tls
run "pg   prefer (default)"            pg "$PG" tls
run "pg   verify-full"                 pg "$PG?sslmode=verify-full" tls
run "pg   disable"                     pg "$PG?sslmode=disable" plain
run "pg   by 127.0.0.1 (IP SAN)"       pg "postgres://scram_user:$PW@127.0.0.1:55432/postgres?sslmode=require" tls
# An untrusted certificate must fail, and not fall back, even under prefer.
out=$(timeout 60 "$tmp/db_tls" pg "$PG?sslmode=prefer" fail 2>&1); echo "pg   untrusted CA, prefer: $out"; [ "${out#ok}" != "$out" ] || fail=1
MY=mysql://sha:$PW@localhost:53306/mysql
run "mysql 8.4 require (sha2 full exchange)" my "$MY?ssl-mode=required" tls
run "mysql 8.4 require again (cached, fast path)" my "$MY?ssl-mode=required" tls
run "mysql 8.4 root, prefer"           my "mysql://root:$PW@localhost:53306/mysql" tls
run "mysql 8.4 disable, cached user"   my "$MY?ssl-mode=disabled" plain
out=$(timeout 60 "$tmp/db_tls" my "$MY?ssl-mode=required" fail 2>&1); echo "mysql untrusted CA, require: $out"; [ "${out#ok}" != "$out" ] || fail=1
MA=mysql://nat:$PW@localhost:53307/mysql
run "mariadb require (native)"         my "$MA?ssl-mode=required" tls
run "mariadb prefer"                   my "$MA" tls
run "mariadb disable"                  my "$MA?ssl-mode=disable" plain

if [ $fail -eq 0 ]; then echo "RESULT: GREEN"; else echo "RESULT: RED"; docker logs --tail 5 ail-itls-pg ail-itls-my ail-itls-ma 2>&1 | tail -20; exit 1; fi
