# garlic: extract
tests: extract.extract.fields_are_found_by_label_in_lists_tables_and_prose, extract.extract.reader_mode_keeps_the_article_and_drops_the_furniture, extract.extract.tables_become_header_and_rows_and_csv

## extract
```
extract.NOISE := ...  ## Elements whose content is page furniture, not the page.
extract.BLOCKS := ...  ## Elements read as one block of text each.
fn extract.clean(d html.Doc, id int) str
fn extract.is_el(d html.Doc, id int, tag str) bool
fn extract.own_table(d html.Doc, tr int) int  ## The table `tr` belongs to (not an outer one).
fn extract.cells(d html.Doc, tr int) List[int]
fn extract.rows_of(d html.Doc, t int) List[List[int]]  ## The rows of table `t`, each a list of cell nodes.
fn extract.all_headers(d html.Doc, row List[int]) bool
fn extract.texts(d html.Doc, row List[int]) List[str]  ## The text of each cell of `row`.
fn extract.jlist(items List[str]) str
pub fn extract.tables_json(d html.Doc) str  ## Every table as `{"tables":[{"id","caption","header","rows"}]}`; the header is
fn extract.csv_cell(s str) str
pub fn extract.table_csv(d html.Doc, k int) str  ## Table number `k` (0-based, in document order) as CSV, header first; "" if there is none.
fn extract.blocks(d html.Doc, id int, mut out List[str]) [mutates]  ## The text of the blocks under `id` in order, noise left out.
fn extract.score(d html.Doc, id int) int  ## How much reading `id` holds: its block text, less its links.
fn extract.main_node(d html.Doc) int  ## The node holding the main content: `main` or `article` when they have
pub fn extract.reader_json(d html.Doc) str  ## `{"title","text","tokens"}`: the main content as paragraphs separated by
fn extract.next_element(d html.Doc, id int) int  ## The element after `id` among its parent's elements, or -1.
fn extract.value_for(d html.Doc, label str) str?  ## The value shown for `label`: the element after a `dt`/`th`/`td`/`label`/`span`/`strong`/`b`
pub fn extract.fields_json(d html.Doc, labels str) str  ## `{"fields":{"label":value or null}}` for the comma-separated `labels`.
```
