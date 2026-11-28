# Standard library: `std.sqlite`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1294, writer ADR 1312). Reads the SQLite 3 database file
> format: the header, b-tree pages, overflow pages, records and the schema. It runs no SQL. A `Writer`
> (`std.sqlite.15` to `std.sqlite.22`) builds a database in memory and writes a complete new file. UTF-16 text,
> WITHOUT ROWID tables and the write-ahead log are out of scope (`std.sqlite.12`, `std.sqlite.22`).

## The module

**std.sqlite.1** — `use std.sqlite` provides `open(data bytes) Database ! SqliteError`, which reads a
database file already in memory, and `open_file(path str) Database ! SqliteError`, which reads the file
first (an unreadable file is `SqliteKind.Io`). Details: [below](#more-on-stdsqlite1). *(ADR 1294)*

**std.sqlite.2** — `d.page_size()` is the size of a page in bytes and `d.page_count()` the number of
pages, which is the file's length divided by the page size. Details: [below](#more-on-stdsqlite2). *(ADR 1294)*

**std.sqlite.3** — The schema table is the b-tree rooted at page 1. Details: [below](#more-on-stdsqlite3). *(ADR 1294)*

```ail rules=std.sqlite.1,std.sqlite.2,std.sqlite.3,std.sqlite.8
fn main() !
    d := sqlite.open(tiny())?
    print("{d.page_size()} {d.page_count()}")
    print(d.tables())
    print(d.indexes())
    for e in d.schema()
        print("{e.kind} {e.name} {e.table} {e.root}")
    cols := d.columns("t")?
    print(cols)

fn tiny() bytes
    parts := [
        "53514c69746520666f726d61742033000200010100402020000000030000000600000000000000000000000200000004000000000000000000000001",
        "00".repeat(35),
        "03002e95c90d00000002017f0001b2017f",
        "00".repeat(271),
        "31020617190f0149696e646578745f6e616d65740343524541544520494e44455820745f6e616d65204f4e2074286e616d65294c0107170f0f0181077461626c65747402435245415445205441424c45207428696420494e5445474552205052494d415259204b45592c206e616d6520544558542c2073636f7265205245414c290d0000000301190001ef01e60119",
        "00".repeat(267),
        "85420305008b0501",
        "7a".repeat(193),
        "00000006070204001300426f620f0104001307416e6e40040000000000000a0000000301c40001f901f101c4",
        "00".repeat(438),
        "8541048b0501",
        "7a".repeat(35),
        "0000000407031301426f620206031309416e6e00000005",
        "7a".repeat(508),
        "00000000",
        "7a".repeat(157),
        "03",
        "00".repeat(354),
        "7a".repeat(507),
        "04",
    ]
    bytes.from_hex(parts.join("")) else b""
```

**std.sqlite.4** — A stored value is a `SqlValue`: `Null`, `Int(n int)`, `Real(x float)`, `Text(s str)` or
`Blob(b bytes)`. A record is a header of varints naming each column's serial type, then the contents:
types 1 to 6 are two's-complement integers of 1, 2, 3, 4, 6 and 8 bytes, 7 is a big-endian IEEE double, 8
and 9 are the integers 0 and 1, and an even type `N` of 12 or more is a blob and an odd one a text, each
`(N - 12) / 2` bytes. Text is read as UTF-8; bytes that are not valid UTF-8 become U+FFFD. `v.is_null()`
tells NULL. `v.show()` writes a value as an SQL literal the way the `sqlite3` shell's quote mode does:
`NULL`, `42`, `2.5`, `'it''s'`, `X'00FF'`. A record whose header, serial types or contents do not fit in
its bytes is `Corrupt`. *(ADR 1294)*

**std.sqlite.5** — A `Row` has `rowid int` and `cols List[SqlValue]`. `r.len()` is the number of values;
`r.get(i)` is value `i`, `Null` when `i` is out of range. Details: [below](#more-on-stdsqlite5). *(ADR 1294)*

**std.sqlite.6** — `d.scan(table)` is every row of the table in rowid order, `d.get(table, rowid)` is the
row with that rowid or `none` (it walks one path down the b-tree), and `d.count(table)` is the number of
rows (it counts cells and decodes no row). Details: [below](#more-on-stdsqlite6). *(ADR 1294)*

**std.sqlite.7** — `d.index_entries(name)` is every entry of an index in key order. An index entry is a
`Row`: `cols` are the indexed columns and `rowid` is the row the entry points at. An interior page of an
index holds entries of its own between its children, and the walk reports them in order.
`d.index_find(name, key)` is the rowids, in key order, of the entries whose first columns equal the
`List[SqlValue]` `key`; values compare exactly, so `Int(1)` is not `Real(1.0)`. It reads the whole index.
A column that the index's CREATE INDEX text names and that has REAL affinity (`std.sqlite.9`) comes back as `Real`; the `sqlite_autoindex_...` indexes have no text and give the stored integers.
*(ADR 1294)*

```ail rules=std.sqlite.4,std.sqlite.5,std.sqlite.6,std.sqlite.7,std.sqlite.9
fn main() !
    d := sqlite.open(tiny())?
    n := d.count("t")?
    print(n)
    rows := d.scan("t")?
    for r in rows
        name := r.str(1) else "?"
        score := r.float(2) else -1.0
        print("{r.rowid} {name.byte_len()} {score} {r.get(2).show()}")
    found := d.get("T", 2)?
    row := found else
        return
    print(row.line("|"))
    missing := d.get("t", 9)?
    print(missing == none)
    entries := d.index_entries("t_name")?
    for e in entries
        print("{e.rowid} {e.cols.len()}")
    ids := d.index_find("t_name", [sqlite.SqlValue.Text(s: "Bob")])?
    print(ids)
    v := sqlite.SqlValue.Text(s: "it's")
    nothing := sqlite.SqlValue.Null
    print("{v.show()} {nothing.is_null()}")

fn tiny() bytes
    parts := [
        "53514c69746520666f726d61742033000200010100402020000000030000000600000000000000000000000200000004000000000000000000000001",
        "00".repeat(35),
        "03002e95c90d00000002017f0001b2017f",
        "00".repeat(271),
        "31020617190f0149696e646578745f6e616d65740343524541544520494e44455820745f6e616d65204f4e2074286e616d65294c0107170f0f0181077461626c65747402435245415445205441424c45207428696420494e5445474552205052494d415259204b45592c206e616d6520544558542c2073636f7265205245414c290d0000000301190001ef01e60119",
        "00".repeat(267),
        "85420305008b0501",
        "7a".repeat(193),
        "00000006070204001300426f620f0104001307416e6e40040000000000000a0000000301c40001f901f101c4",
        "00".repeat(438),
        "8541048b0501",
        "7a".repeat(35),
        "0000000407031301426f620206031309416e6e00000005",
        "7a".repeat(508),
        "00000000",
        "7a".repeat(157),
        "03",
        "00".repeat(354),
        "7a".repeat(507),
        "04",
    ]
    bytes.from_hex(parts.join("")) else b""
```

**std.sqlite.8** — `d.columns(table)` is the column names of a table in declaration order, read from the
text of its CREATE TABLE: the definitions inside the outer parentheses, split at the commas outside
parentheses and quotes. Details: [below](#more-on-stdsqlite8). *(ADR 1294)*

**std.sqlite.9** — The file does not hold what a query returns for two kinds of column, so `scan` and `get`
fix them from the column's declaration. A column declared `INTEGER PRIMARY KEY` is the table's rowid, which
the file stores as NULL: its value is `Int(rowid)`. Details: [below](#more-on-stdsqlite9). *(ADR 1294)*

**std.sqlite.10** — A cell too large for its page continues on overflow pages, a chain of pages each
holding the next page's number and then content. The reader follows the chain for table and index cells
with SQLite's limits (the on-page share of a cell is computed from the usable page size, which is the
page size less the reserved bytes of the header) and joins the pieces; a chain that ends early, leaves
the file or has more pages than the file is `Corrupt`. *(ADR 1294)*

```ail rules=std.sqlite.10
fn main() !
    d := sqlite.open(tiny())?
    rows := d.scan("t")?
    big := rows[2].str(1) else ""
    starts := big.starts_with("zzz")
    print("{big.byte_len()} {starts}")
    entries := d.index_entries("t_name")?
    last := entries[2].str(0) else ""
    print(last.byte_len())

fn tiny() bytes
    parts := [
        "53514c69746520666f726d61742033000200010100402020000000030000000600000000000000000000000200000004000000000000000000000001",
        "00".repeat(35),
        "03002e95c90d00000002017f0001b2017f",
        "00".repeat(271),
        "31020617190f0149696e646578745f6e616d65740343524541544520494e44455820745f6e616d65204f4e2074286e616d65294c0107170f0f0181077461626c65747402435245415445205441424c45207428696420494e5445474552205052494d415259204b45592c206e616d6520544558542c2073636f7265205245414c290d0000000301190001ef01e60119",
        "00".repeat(267),
        "85420305008b0501",
        "7a".repeat(193),
        "00000006070204001300426f620f0104001307416e6e40040000000000000a0000000301c40001f901f101c4",
        "00".repeat(438),
        "8541048b0501",
        "7a".repeat(35),
        "0000000407031301426f620206031309416e6e00000005",
        "7a".repeat(508),
        "00000000",
        "7a".repeat(157),
        "03",
        "00".repeat(354),
        "7a".repeat(507),
        "04",
    ]
    bytes.from_hex(parts.join("")) else b""
```

## Writing

**std.sqlite.15** — `sqlite.create()` is an empty `Writer`, and `d.edit()` on an opened `Database` is a
`Writer` holding its tables, rows, indexes, views and triggers (`std.sqlite.21`). A `Writer` holds the whole
database in memory; it runs no SQL. `w.tables()`, `w.scan(t)`, `w.get(t, rowid)` and `w.count(t)` read what it
holds as the reader methods of the same name do (`std.sqlite.5`, `std.sqlite.6`). Details: [below](#more-on-stdsqlite15). *(ADR 1312)*

**std.sqlite.16** — `w.create_table(sql)` adds the table of a CREATE TABLE text, and `w.drop_table(name)` removes
a table with its indexes. The text is stored as given. A WITHOUT ROWID table, a virtual table, a temporary
table and CREATE TABLE AS SELECT are `Unsupported`; a name in use is `Misuse`; a name starting `sqlite_` is
`Misuse`. A UNIQUE or PRIMARY KEY constraint that is not the rowid alias gets the automatic index
`sqlite_autoindex_TABLE_N` (N counts from 1 in the order of the constraints), and the first AUTOINCREMENT
table adds `sqlite_sequence`. *(ADR 1312)*

**std.sqlite.17** — `w.create_index(sql)` adds an index from its CREATE INDEX text and fills it from the rows;
`w.drop_index(name)` removes it (not the automatic index of a constraint, which is `Misuse`). Columns must be
plain column names, each ASC or DESC, with BINARY collation; a WHERE clause, an expression or another
collation is `Unsupported`. A UNIQUE index over rows that already repeat a key is `Constraint`. Entries are
ordered as SQLite orders them: NULL, then numbers by value, then text and blobs by their bytes, then the
rowid. *(ADR 1312)*

**std.sqlite.18** — `w.insert(table, vals)` adds a row of one `SqlValue` per column, in column order, and
returns its rowid. A `Null` in the INTEGER PRIMARY KEY column takes the next rowid (one more than the
largest; with AUTOINCREMENT, one more than the larger of the largest and the table's `sqlite_sequence`
value, which the insert then raises); an `Int` there is the rowid; any other value is `Misuse`. Column
affinity is applied first (`std.sqlite.9`): TEXT turns an integer into text, INTEGER and NUMERIC turn text
that reads as a number, and a whole `Real`, into an integer, and REAL turns an integer or numeric text into a
`Real`. A `Real` for a TEXT column is `Misuse` (`Int` becomes text, a `Real` has no one spelling: pass `Text`). A wrong number of values is `Misuse`; NOT NULL,
UNIQUE, a PRIMARY KEY, a taken rowid and a UNIQUE index are `Constraint`, with SQLite's message
(`UNIQUE constraint failed: people.name`). DEFAULT, CHECK, FOREIGN KEY and generated columns stay in the
table's text and are not applied or checked. *(ADR 1312)*

**std.sqlite.19** — `w.update(table, rowid, vals)` replaces the values of the row with that rowid, with the
checks of `std.sqlite.18`; a `Null` in the rowid alias column keeps the rowid and an `Int` moves the row. A
failed update changes nothing. A missing row is `NoSuch`. `w.delete(table, rowid)` removes a row and is
`false` when there was none; with AUTOINCREMENT the table's sequence is not lowered. *(ADR 1312)*

**std.sqlite.20** — `w.bytes()` is the database as an SQLite 3 file and `w.save(path)` writes it (an unwritable
path is `Io`); `w.database()` opens what `bytes` gives. The file is complete and new: every table and index
is a freshly built b-tree, with page size 4096, UTF-8 text, no free pages, no write-ahead log, and the
SQLite library's `PRAGMA integrity_check` accepts it. Payloads too large for a page spill to overflow
pages. `SqliteKind` has two more members for writing: `Constraint` and `Misuse`. *(ADR 1312)*

**std.sqlite.21** — `d.edit()` reads every table row, index definition, view and trigger of `d`, and its
`user_version` and `application_id`, into a `Writer`; `w.bytes()` then writes the changed database, with the
same content as `d` where nothing was changed. The reader's limits apply (`std.sqlite.12`); also `Unsupported`
are a row with fewer values than its table has columns (ALTER TABLE ADD COLUMN), an expression or partial index,
and an index with a collation other than BINARY. Automatic indexes are rebuilt from the table's constraints. *(ADR 1312)*

**std.sqlite.22** — Not written, and `Unsupported`: WITHOUT ROWID tables, virtual tables, UTF-16 text, the
write-ahead log, partial and expression indexes, triggers and views made by this module (a `Writer` carries
those of an edited file unchanged; it makes none), foreign key enforcement, SQL text. A `Writer` is not a
database engine: it keeps no journal, so a process that stops before `save` leaves no file, and the file is
written whole. *(ADR 1312)*

```ail rules=std.sqlite.15,std.sqlite.16,std.sqlite.17,std.sqlite.18,std.sqlite.19,std.sqlite.20
fn main() !
    mut w := sqlite.create()
    w.create_table("CREATE TABLE pets(id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, legs INTEGER)")?
    w.create_index("CREATE INDEX pets_legs ON pets(legs)")?
    cat := w.insert("pets", [sqlite.SqlValue.Null, sqlite.SqlValue.Text(s: "cat"), sqlite.SqlValue.Int(n: 4)])?
    bird := w.insert("pets", [sqlite.SqlValue.Null, sqlite.SqlValue.Text(s: "bird"), sqlite.SqlValue.Text(s: "2")])?
    print("{cat} {bird}")
    _ := w.insert("pets", [sqlite.SqlValue.Null, sqlite.SqlValue.Text(s: "cat"), sqlite.SqlValue.Int(n: 4)]) else err
        print("{err.kind.name()}: {err}")
        0
    w.update("pets", cat, [sqlite.SqlValue.Null, sqlite.SqlValue.Text(s: "kitten"), sqlite.SqlValue.Int(4)])?
    print(w.delete("pets", 99)?)
    d := w.database()?
    for r in d.scan("pets")?
        print("{r.rowid}: {r.line(", ")}")
    print(d.index_find("pets_legs", [sqlite.SqlValue.Int(2)])?)
```


## Limits and errors

**std.sqlite.11** — A b-tree deeper than 40 levels is `Corrupt`. Details: [below](#more-on-stdsqlite11). *(ADR 1294)*

**std.sqlite.12** — Not read, and reported as `SqliteKind.Unsupported`: a database whose text encoding
is UTF-16, and a WITHOUT ROWID table (its b-tree is an index b-tree). Details: [below](#more-on-stdsqlite12). *(ADR 1294)*

**std.sqlite.13** — `SqliteError` has the fields `kind SqliteKind` and `why str`, and its text is `why`.
`SqliteKind` is `NotSqlite` (the bytes are shorter than 100 or do not start with `SQLite format 3`
followed by a NUL), `Corrupt` (the header or a page or record is damaged or points outside the file),
`Unsupported` (`std.sqlite.12`), `NoSuch` (no table or index of that name) and `Io` (`open_file` could
not read the file). `Constraint` and `Misuse` are returned by a `Writer` only (`std.sqlite.20`). *(ADR 1294)*

**std.sqlite.14** — Only the bytes of the file matter: the module gives the same rows on every target, in
big-endian and little-endian hosts alike, and takes no library from the system. *(ADR 1294)*

```ail rules=std.sqlite.11,std.sqlite.12,std.sqlite.13,std.sqlite.14
fn show(label str, data bytes)
    d := sqlite.open(data) else err
        print("{label}: {err.kind.name()}: {err}")
        return
    print("{label}: ok {d.tables().len()}")

fn main() !
    good := tiny()
    show("good", good)
    show("text", "not a database".to_bytes())
    show("short", good[0..100])
    show("utf16", good[0..56] + b"\x00\x00\x00\x02" + good[60..good.len()])
    d := sqlite.open(good)?
    rows := d.scan("nope") else err
        print("{err.kind.name()}")
        return
    print(rows.len())

fn tiny() bytes
    parts := [
        "53514c69746520666f726d61742033000200010100402020000000030000000600000000000000000000000200000004000000000000000000000001",
        "00".repeat(35),
        "03002e95c90d00000002017f0001b2017f",
        "00".repeat(271),
        "31020617190f0149696e646578745f6e616d65740343524541544520494e44455820745f6e616d65204f4e2074286e616d65294c0107170f0f0181077461626c65747402435245415445205441424c45207428696420494e5445474552205052494d415259204b45592c206e616d6520544558542c2073636f7265205245414c290d0000000301190001ef01e60119",
        "00".repeat(267),
        "85420305008b0501",
        "7a".repeat(193),
        "00000006070204001300426f620f0104001307416e6e40040000000000000a0000000301c40001f901f101c4",
        "00".repeat(438),
        "8541048b0501",
        "7a".repeat(35),
        "0000000407031301426f620206031309416e6e00000005",
        "7a".repeat(508),
        "00000000",
        "7a".repeat(157),
        "03",
        "00".repeat(354),
        "7a".repeat(507),
        "04",
    ]
    bytes.from_hex(parts.join("")) else b""
```

### `std.sqlite` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.sqlite.1

The whole file is held in memory as `bytes`, so a database larger than memory is out of scope. A `Database` is a value: it is never closed, and reading it changes nothing. `open` is pure (`eff.infer.1`); `open_file` needs the effect `fs.read`. The module does not link SQLite or any library: it is the file format written in AIL.

#### More on std.sqlite.11

Every read is bounds-checked, so a damaged or hostile file fails with `SqliteError`; it never traps or reads outside the bytes. A page that lists itself as a child can make a read slow before the depth limit stops it.

#### More on std.sqlite.12

A database in write-ahead-log mode is read as the main file holds it: the `-wal` file is not looked at, so changes not yet checkpointed are not seen. Views and triggers are listed by `schema()` but have no rows. Free pages, the lock-byte page and pointer-map pages are never read. Nothing is written.

#### More on std.sqlite.2

A page size of 1 in the header means 65536; a size that is not a power of two from 512 to 65536 is `Corrupt`. Bytes past the last whole page are ignored.

#### More on std.sqlite.3

`d.schema()` is its rows in file order as `SchemaEntry` values with the fields `kind` (`table`, `index`, `view` or `trigger`), `name`, `table` (the table it belongs to; a view's own name), `root` (the page of its b-tree, 0 for a view or a trigger) and `sql` (the text that made it, `""` when it has none). `d.tables()` and `d.indexes()` are the names of the entries of those kinds, in schema order; internal tables such as `sqlite_sequence` and the `sqlite_autoindex_...` indexes made for constraints are included.

#### More on std.sqlite.5

`r.int(i)`, `r.float(i)`, `r.str(i)` and `r.bytes(i)` are `int?`, `float?`, `str?` and `bytes?`: `none` for NULL or another type, except that `r.float(i)` also reads an `Int` as a `float`. `r.line(sep)` is the values as `show` writes them, joined with `sep`.

#### More on std.sqlite.6

A table is found by name without regard to case; no table of that name is `SqliteKind.NoSuch`. A row stored before a column was added with ALTER TABLE has fewer values than the table has columns.

#### More on std.sqlite.8

A name in `"..."`, `` `...` `` or `[...]` comes back without its quotes. A definition that starts with CONSTRAINT, PRIMARY, UNIQUE, CHECK or FOREIGN is a table constraint, not a column.

#### More on std.sqlite.9

A column with REAL affinity (SQLite's rules: a type with INT, CHAR, CLOB, TEXT or BLOB, or no type, is not REAL; one with REAL, FLOA or DOUB is) stores a whole number as an integer, and comes back as `Real`. Other affinities are not applied: NUMERIC `3.0` is stored, and read, as `Int(3)`. A table whose primary key is a separate constraint (`PRIMARY KEY(id)`) gets no rowid alias here.

#### More on std.sqlite.15

The `Writer` is a value, like a `List`: `mut w := sqlite.create()`, and every method that changes it is called on a `mut` variable. It keeps rows sorted by rowid and the keys of UNIQUE indexes in a map, so an insert in rowid order and a lookup by key are fast; an insert into the middle moves the rows after it. Other indexes are sorted when the file is written.

#### More on std.sqlite.18

`insert` takes every column because there is no SQL to name a subset; to use a default, pass it. A `Real` is not converted to text for a TEXT column, because SQLite spells floating-point text with its own rules (15 significant digits) and `PRAGMA integrity_check` calls a number in a TEXT column an error; the `Writer` refuses it, and you pass the `Text` you want. Indexed REAL and NUMERIC values follow the table's affinity, so `Int(1)` and `Real(1.0)` are the same stored value only in a column whose affinity makes them so.
