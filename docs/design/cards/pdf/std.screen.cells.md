# pdf: std.screen.cells

## std.screen.cells
```
fn std.screen.cells.hash(row int, col int) int
fn std.screen.cells.page_bg(runs List[std.ui.widgets.Run]) int  ## The background that covers the most cells: the page, which gets no box.
pub fn std.screen.cells.of_cells(cols int, rows int, runs List[std.ui.widgets.Run], cursor_col int, cursor_row int) std.screen.Screen  ## The items of `runs`, in paint order: a `box` for each run on a background
```
