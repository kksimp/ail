# 1259. `std.csv`: dialects, streaming, and struct rows by derive

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 - Standard library
- **Spec rules:** std.csv.8-15 ([17-std](../spec/17-std.md))
- **Extends:** [ADR 0626](0626-std-csv.md)

The integrator replaces `XXXX` and `ADR 1259` in `std/csv/csv.ail` and the spec.

## Context

`std.csv` (ADR 0626) read and wrote comma separated text held whole in memory.
Real files use tabs or semicolons, escape quotes with a backslash, end lines
with `\r\n`, run to gigabytes, and map to a struct per row. The writer also
built its output by repeated concatenation, which is quadratic, and the reader
appended one character at a time.

## Decision

- `Dialect` (sep, quote, backslash escape, eol, always quote) with `dialect()`,
  `tsv()`, `semicolon()`; `parse_with`, `records_with`, `quote_with`,
  `row_text`, `write_with`, `write_records_with`. The old calls are the
  comma dialect and keep their results.
- One scanner reads a record by slicing each field out of the text, so reading
  is linear; the writer joins a list of pieces.
- `open` / `open_with` give a unique `Rows` over `std.file.Reader` with `next`
  and `next_record`, reading one physical line at a time and joining lines
  while a quote is open. `write_row` writes to a `file.Writer`. Memory holds
  one row.
- `CsvRow` derives `csv_header`, `to_csv_row`, `from_csv_row` for structs of
  `str`, `int`, `float`, `bool` and their optionals, by field name, with
  `read_as[T]` and `write_as`. This is the typed-rows item ADR 0626 left open.

## Rejected alternatives

- **A native (runtime) reader.** The scanner is pure AIL; the runtime route
  waits for a measurement that shows it too slow.
- **Separate `tsv` module.** A `Dialect` covers TSV, semicolons and quote
  styles with one set of calls.
- **Positional struct mapping.** Reordered or extra columns would silently
  misread; mapping by header name fails loudly.
- **A `FromCsv` derive in the compiler prelude.** A std module owning its
  trait keeps the change out of the hot compiler files.
- **Dialect as default parameters.** Default parameters pass by name and the
  common call is `csv.parse(text)`; `_with` calls keep it short.

## Consequences

`open` treats each line ending in `\n` or `\r\n` alike and does not split on a
bare `\r`. Quoted newlines read as `\n`. Only `std.file`'s I/O errors pass
through. Windows parity: no Unix-only API.
