# 1294. `std.sqlite`: a reader for SQLite 3 database files

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.sqlite.1 to std.sqlite.14](../spec/std-sqlite.md)

The number is a placeholder: the integrator assigns it when the chain lands.

## Context

SQLite files are the most common database file agents are handed: browser profiles, app stores, phone backups,
exports. `std.db` (ADR 1207) is an engine with its own file format and cannot open them, and `not-in-std.txt`
sends agents to hand-rolled parsers. A reader of the file format, with no SQL, covers the case of getting rows
out of a file someone else wrote (std-gap plan, section 3).

## Decision

- A new module `std.sqlite`, written in AIL (`std/sqlite/sqlite.ail`), over the file's bytes. `open(bytes)` is
  pure; `open_file(path)` reads with `fs.read_bytes`. The module links no library.
- Reads the header, table and index b-trees (interior and leaf pages), overflow chains, the record format with
  all serial types, and the schema table. API: `schema`, `tables`, `indexes`, `columns`, `scan`, `get` by rowid
  (one path down the tree), `count`, `index_entries`, `index_find`. Values are `SqlValue` (`Null`, `Int`, `Real`,
  `Text`, `Blob`); rows are `Row` with index-based accessors and `show`, which writes SQL literals as the
  sqlite3 shell's quote mode does.
- `scan` and `get` return what a query would: an INTEGER PRIMARY KEY column holds the rowid and a REAL-affinity
  column holds a `Real` even where the file stored an integer. This needs the column list, so the module
  parses the CREATE TABLE text with a small splitter that handles quotes, brackets and nested parentheses.
- Every read is bounds-checked, the b-tree depth is capped, and failures are `SqliteError` with a `kind` enum
  (`NotSqlite`, `Corrupt`, `Unsupported`, `NoSuch`, `Io`): a hostile file never traps.
- Out of scope and reported as `Unsupported`: UTF-16 text and WITHOUT ROWID tables. The `-wal` file is not read.
- Named `std.sqlite`, not `std.db.sqlite_read`: `std.db` is an existing module, and a name that has it as a
  prefix invites confusion in module lists, `ail ask` and the implicit-import table. A later writer adds
  to the same module.
- Tested on fixtures made by the real SQLite library (`tests/sqlite/gen.py`): all integer widths, REAL and
  NUMERIC columns, quoted names, rowid gaps, a 3-level table and index on 512-byte pages, and overflow chains at
  the on-page limit in tables and indexes. The expected output is read back from SQLite itself, and the script
  also runs a Python port of the reader and fails if it differs from SQLite.

## Rejected alternatives

- **Link `libsqlite3`.** Needs an external library on all three targets and an FFI surface, against the in-house
  tooling rule (ADR 0009), and a licensing and distribution question for the designer.
- **Run SQL.** `std.db` is the engine; a second SQL implementation would diverge from it. Row access by table,
  rowid and index covers the reading case, and a SQL layer can sit on top later.
- **Read pages on demand with `std.file` instead of the whole file.** Saves memory for multi-gigabyte files but
  every page access becomes fallible I/O and the module loses its pure `open`. Whole-file `bytes` is shared, not
  copied, and a streaming open can be added without changing the row API.
- **Apply every column affinity.** NUMERIC, TEXT and INTEGER affinities do not change stored values on read, so
  only REAL (integer storage of whole numbers) and the rowid alias matter.
- **UTF-16 text now.** Rare in files agents meet; a conversion table is one more thing to get wrong blind.
  `Unsupported` says so plainly.
- **Walk an index by key (binary search).** Needs collation support; `index_find` reads the index in order
  instead, correct for the default BINARY collation and no worse than a table scan.

## Consequences

One new file under `std/`, one spec chapter, registration lines in the compiler's module lists, tests
`tests/cases/std_sqlite*.ail` with fixtures in `tests/sqlite/`. No syntax change and no runtime or checker change.
