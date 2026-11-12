# compiler: cheader
tests: cheader.cheader.defines_become_constants, cheader.cheader.enums_become_constants, cheader.cheader.follows_the_headers_own_typedefs, cheader.cheader.function_pointers_are_opaque_pointers, cheader.cheader.keeps_variadic_prototypes, cheader.cheader.long_follows_the_target, cheader.cheader.looks_through_guards_and_attributes, cheader.cheader.module_names_round_trip, cheader.cheader.reads_scalars_and_pointers, cheader.cheader.skips_what_the_ffi_cannot_carry, cheader.cheader.unions_stay_behind_pointers

## cheader
```
pub fn cheader.tokens(text str) List[str]  ## The tokens of a header: identifiers, numbers and single symbols. Comments, preprocessor
pub fn cheader.module_text(src str, windows bool) str  ## The AIL source of the module for header text `src`: one `pub extern c fn` per prototype
pub fn cheader.module_name(header str) str  ## Every module of a header is named `__c_` and the header's spelling, each character that is
pub fn cheader.is_use(parts List[str]) bool  ## Whether the import path `parts` is the one `module_name` makes.
pub fn cheader.header_of(name str) str  ## The header a module name stands for.
pub fn cheader.dir_of(file str) str  ## The directory part of a file path ("." for none).
pub fn cheader.find(header str, dir str) str? [fs io]  ## The file of `header`: beside the importing file in `dir`, else the system's (ADR 0926).
pub fn cheader.module_for(header str, dir str) str? [env fs io]  ## The module text for `header`, or none when no file of that name is found.
```

trimmed to fit 2000 tokens (~493); dropped: private items
