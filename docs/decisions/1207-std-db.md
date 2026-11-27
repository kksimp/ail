# 1207. `std.db`: an embedded SQL database written in AIL

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the standard library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** std.db.1-16 ([17-std](../spec/17-std.md))
- **Builds on:** [ADR 1098](1098-std-kv.md) (the crash-safe log), [ADR 0898](0898-ail-database-engine.md) (the database showcase)

## Context

An audit found that agents asked to build anything that needs a database stop in AIL: there is no package
registry and nothing in std beyond `std.kv`, so they hand-write a file format or a JSON file per change. SQL is the
query language every model already writes. The project's rule is in-house code (TLS and crypto included) and no
registry; linking the system's `libsqlite3` would add a C dependency that Windows lacks and a product decision.

## Decision

`std.db`, one file of AIL (`std/db/db.ail`) over `std.fs` and `std.hash`, in scope as `db` with no `use`
(std.use.4):

- `db.open(path) Db ! DbError` and `db.memory() Db`; `Db` is `unique`. `d.exec(sql, params) int` runs one or more
  statements and returns the rows changed; `d.query(sql, params) List[Row]` runs one SELECT. Parameters are `?` (or
  `?N`) with a `List[Value]`; `Value` is `Null | Int | Real | Text | Blob`, written bare in the list
  (`[Text("ann"), Int(31)]`). `Row` reads columns by name: `r.int("age")` (`int?`, like `Json.int`), `float`, `str`,
  `bytes`, `bool`, `is_null`, `get`, and `to_json()` for `T.from_json`.
- A subset of SQLite's SQL: CREATE/DROP TABLE and INDEX, INSERT (with OR IGNORE / OR REPLACE), UPDATE, DELETE,
  SELECT with inner, LEFT and cross joins, WHERE, GROUP BY, HAVING, ORDER BY, LIMIT/OFFSET, DISTINCT, the common
  aggregates and scalar functions, CASE and CAST. What is left out is a `Syntax` error that says so and, where there
  is one, names the form to write instead.
- Strict column types: a value of the wrong type is an error, never converted (an INTEGER column takes 3.0 as 3; a
  REAL column takes 3 as 3.0). SQLite's rules map a declared name to a type, so `VARCHAR(80)` is TEXT. An
  INTEGER PRIMARY KEY is the row id, never reused. NOT NULL, PRIMARY KEY, UNIQUE and unique indexes are enforced,
  with SQLite's messages. `REFERENCES` is accepted and not checked, as SQLite does by default.
- Each statement runs whole or not at all (an undo list puts rows back); `begin`, `commit`, `rollback` (or the SQL
  statements) group changes. A transaction's rollback restores a copy of the tables taken at `begin`.
- Storage: all tables in memory; the file is the `std.kv` log design, one CRC-framed record per statement or
  transaction holding row-level changes (CREATE text, put row, delete row, drop). `open` replays and cuts a torn tail;
  `compact` (also automatic past four changes per live row) rewrites through a temporary file and a rename. A failed
  write poisons the handle until it is reopened, so memory never silently differs from the file.
- Errors: one `DbError` with `kind DbKind` (`Syntax`, `Schema`, `Constraint`, `Mismatch`, `Usage`,
  `Transaction`, `Io`, `Corrupt`), so a caller tests `err.kind == Constraint`.
- Speed: `col = value` on the row id or a one-column unique key reads one row; an equality join builds a map of
  the second table. Everything else scans.

## Rejected alternatives

- **Link the system `libsqlite3` through FFI.** Far less code, but a C library std would depend on (absent on
  Windows), a licence and supply question, and against the in-house rule; ADR 0898 rejected it for the same reasons.
  Not taken without the designer.
- **An SQLite-compatible file format now.** Lets other tools open the file, but needs the B-tree page format,
  varints, overflow pages and the schema table before the first query works: several times this slice. The SQL
  layer here does not depend on the storage, so a page store (ADR 0898's B-tree and log) can replace the in-memory
  tables later, and an SQLite importer can be added beside it.
- **A key-value or document API instead of SQL.** `std.kv` already covers keys and bytes; agents write SQL fluently
  and need joins, filters and aggregates, which a key-value API makes them hand-write.
- **SQLite's loose typing (affinity).** Matches SQLite exactly but stores `'abc'` in an INTEGER column without a
  word; strict types turn that mistake into an error at the statement (AI understanding first, ADR 1077).
- **Optional parameters (`exec(sql)` alone).** A defaulted parameter must be passed by name (expr.call.4), so
  `exec(sql, [Int(1)])` would be an error; one required `params` list, `[]` when there are none, is one form.
- **Accessors that fail (`int ! DbError`) or trap on NULL.** `T?` matches `Json.int`/`Json.str` and reads with
  `else`; a column name that is not in the row stops the program instead, since that is a typo, not data.
- **Undo by snapshot for every statement.** With copy-on-write values every statement would copy the table it
  changes; the undo list costs only the rows the statement touches.
- **Persist statements as SQL text and replay them.** Replay would re-run expressions and depend on evaluation
  order; row-level changes replay exactly and compact to one change per row.

## Consequences

- The whole database must fit in memory (std.db.14). A page store is the next storage slice.
- Not yet: subqueries, `INSERT ... SELECT`, `ALTER TABLE`, upsert with `ON CONFLICT`, `RETURNING`, views,
  triggers, time functions, foreign-key checks, a non-unique index used for lookups, `query_as[T]` decoding rows into
  structs (rows decode with `T.from_json(r.to_json())`), and a `db` shell.
- `docs/design/not-in-std.txt` no longer sends database searches away from std.

## Measurements

None yet: the first gate run gives the test timings. No tokbench twin exists.
