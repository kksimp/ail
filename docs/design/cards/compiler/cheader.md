# compiler: cheader
tests: cheader.cheader.follows_the_headers_own_typedefs, cheader.cheader.long_follows_the_target, cheader.cheader.looks_through_guards_and_attributes, cheader.cheader.module_names_round_trip, cheader.cheader.reads_scalars_and_pointers, cheader.cheader.skips_what_the_ffi_cannot_carry

## cheader
```
cheader.QUALIFIERS := ...  ## Words that qualify a declaration and say nothing about its type.
cheader.GROUPS := ...  ## Words followed by a parenthesised group that is no part of the type.
cheader.SCALARS := ...  ## The words of a C scalar type.
cheader.RESERVED := ...  ## AIL's reserved words (the lexer's list): a function named so is left out.
fn cheader.ident_start(c char) bool
fn cheader.ident_char(c char) bool
pub fn cheader.tokens(text str) List[str]  ## The tokens of a header: identifiers, numbers and single symbols. Comments, preprocessor
fn cheader.after_group(toks List[str], open int) int  ## The index just past the group that opens at `open` (a `(`, `[` or `\{`).
fn cheader.clean(toks List[str]) List[str]  ## `toks` without qualifiers and without the groups that follow `__attribute__` and the like.
fn cheader.base_aliases(windows bool) Map[str, str]  ## The AIL types of the standard typedefs a header names without defining.
struct cheader.CType  ## A C type read as an AIL type: `text` is the AIL spelling, empty for plain `void`.
    text str
fn cheader.stars_of(n int, base str) str
fn cheader.scalar_type(words List[str], stars int, windows bool) cheader.CType?  ## The AIL type of a run of C scalar words (`unsigned long`, `char`, `long long`), `void` as "".
fn cheader.parse_type(toks List[str], aliases Map[str, str], windows bool) cheader.CType?  ## The type spelled by `toks` (no name among them), or none when the FFI cannot carry it.
struct cheader.CFn  ## One function: the C name and its `pub extern c fn` line.
    name str
    line str
fn cheader.split_params(toks List[str]) List[List[str]]  ## The split of `toks` at the commas outside any group.
fn cheader.param_type(toks List[str], aliases Map[str, str], windows bool) cheader.CType?  ## The type of one parameter, with or without a name after it; none for `void`.
fn cheader.prototype(decl List[str], aliases Map[str, str], windows bool) cheader.CFn?  ## The function a declaration's tokens declare, if the FFI can carry it.
fn cheader.add_typedef(decl List[str], mut aliases Map[str, str], windows bool) [mutates]  ## Record `typedef T name;` when `T` is a type the FFI carries.
pub fn cheader.module_text(src str, windows bool) str  ## The AIL source of the module for header text `src`: one `pub extern c fn` per prototype
pub fn cheader.module_name(header str) str  ## Every module of a header is named `__c_` and the header's spelling, each character that is
fn cheader.code_char(code int) str  ## The character with code `code`, as text ("" for none).
fn cheader.hex_digit(cs List[char], i int) int
pub fn cheader.is_use(parts List[str]) bool  ## Whether the import path `parts` is the one `module_name` makes.
pub fn cheader.header_of(name str) str  ## The header a module name stands for.
pub fn cheader.dir_of(file str) str  ## The directory part of a file path ("." for none).
cheader.SYSTEM_DIRS := ...  ## Where the system keeps C headers.
pub fn cheader.find(header str, dir str) str? [fs io]  ## The file of `header`: beside the importing file in `dir`, else the system's (ADR 0926).
fn cheader.cat_dirs(dir str) List[str]
pub fn cheader.module_for(header str, dir str) str? [env fs io]  ## The module text for `header`, or none when no file of that name is found.
```
