# TBD. `std.sqlite`: a writer that builds and rewrites SQLite 3 files

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.sqlite.15 to std.sqlite.22](../spec/std-sqlite.md)

The number is a placeholder: the integrator assigns it when the chain lands. It extends [ADR 1294](1294-std-sqlite.md).

## Context

The state review of 2026-11-27 listed `std.sqlite` being read-only as a std gap: `std.db` writes its own file
format, so an agent asked for "a SQLite database" had no way to make one, and a file a user hands over could be
read but not changed.

## Decision

- A `Writer` type in `std.sqlite` (the same file, one namespace) holds a database in memory as tables of rows
  with their definitions, and `bytes()` writes the whole database as a new SQLite 3 file. Nothing patches an
  existing file: every table and index b-tree is built again from the rows, page size 4096, UTF-8, no free pages.
- API: `sqlite.create()`, `Database.edit()` (load an opened file), `create_table(sql)`, `create_index(sql)`,
  `drop_table`, `drop_index`, `insert(table, vals)` (returns the rowid), `update(table, rowid, vals)`,
  `delete(table, rowid)`, `tables`, `scan`, `get`, `count`, `bytes`, `save(path)`, `database()`. Two `SqliteKind`
  members are added: `Constraint` and `Misuse`.
- The table and index definitions are the CREATE text, which the module already splits for the reader. NOT NULL,
  UNIQUE, PRIMARY KEY, UNIQUE indexes, a taken rowid and AUTOINCREMENT (with `sqlite_sequence`) are enforced on
  every write, with SQLite's messages; the automatic indexes of UNIQUE and PRIMARY KEY constraints are created
  and named as SQLite names them, because the library refuses a file whose schema lacks them. Column affinity
  is applied on insert, as the library applies it, because `PRAGMA integrity_check` reports a stored value that
  breaks it.
- Index entries are ordered by SQLite's BINARY collation (NULL, numbers, text and blobs by bytes, then rowid),
  with DESC columns reversed. Interior index pages hold entries, so entries move up out of the leaves as the
  tree is built; payloads over the page limit spill to overflow pages.
- Loading an existing file rebuilds the automatic indexes from the table text and carries views and triggers as
  schema text. A row shorter than its table (ALTER TABLE ADD COLUMN), a partial or expression index and a
  non-BINARY collation are `Unsupported`, so an edit never changes what a query on the file returns.
- Tested against the real SQLite library, not against the module's own reader: `tests/sqlite/write/verify.py`
  applies seeded operation lists (thousands of inserts, updates and deletes, large blobs and text, composite
  unique keys, DESC indexes, AUTOINCREMENT, mixed affinities, a table of 260,000 rows and an index tree three
  levels deep) to SQLite and to the Writer, and checks that every outcome agrees, that `PRAGMA integrity_check`
  passes on the written file and on a reopen-and-edit round trip, and that every table, index and
  `sqlite_sequence` holds the same typed rows. `tests/cases/std_sqlite_write.out` is written by
  `tests/sqlite/write/gen_case.py` from SQLite's own answers.

## Rejected alternatives

- **Patch pages in place.** Needs page splits, merges, a free list and the rollback journal to be safe; a
  crash mid-write would damage the file. Rebuilding is simple, deterministic and verified by the library's
  own integrity check. The cost is that a large file is rewritten as a whole, and the writer holds it in memory.
- **Run SQL.** The same reason as ADR 1294: `std.db` is the engine, and a second SQL implementation would
  diverge. The writer takes rows as values; an agent who has SQL text still has `std.db`.
- **Implement DEFAULT, CHECK and FOREIGN KEY.** They need an expression evaluator. They stay in the table's
  text, so a later write by SQLite itself still applies them, and the doc says the writer does not.
- **Convert a REAL to text for a TEXT column.** SQLite has its own 15-digit spelling; a different spelling
  would put different text in the file than SQLite does. `Misuse` with a clear message is safer.
- **A separate module `std.sqlite.write`.** The writer needs the reader's record, varint and schema code;
  making that public would widen the API for no user. One namespace keeps both behind `std.sqlite`.

## Consequences

About 600 lines more in `std/sqlite/sqlite.ail`, one spec section, a test case `std_sqlite_write` and a
differential check `tests/sqlite/write/` (run it with `tests/sqlite/write/run.sh`; it needs `python3` only, because
the standard `sqlite3` module links the real library). The reader's behaviour is unchanged.
