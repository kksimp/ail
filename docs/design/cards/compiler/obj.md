# compiler: obj

## obj
```
pub obj.X86_64 := 62  ## ELF machine numbers.
pub obj.AARCH64 := 183
pub fn obj.elf(img codegen.Image, machine int) List[u8]
pub fn obj.link(img codegen.Image, code int, data int, bss int) List[u8]  ## The code with every relocation patched, for code at address `code`,
pub fn obj.link_reloc(mut out List[u8], r codegen.Reloc, to int, pc int) [mutates]  ## Patch relocation `r` in `out`: `to` is the symbol's address and `pc` the
pub fn obj.align_up(n int, a int) int
pub fn obj.put(mut out List[u8], v int, n int) [mutates]  ## `v` in `n` little-endian bytes.
pub fn obj.bytes_of(xs List[int]) List[u8]  ## The bytes of a list of byte values, each 0 to 255.
pub fn obj.hex(bytes List[u8]) str  ## Bytes as lines of hexadecimal text, for the driver to write out until
pub fn obj.macho(img codegen.Image, ident str) List[u8]  ## The executable, signed with `ident` as its identifier.
pub fn obj.macho_with(img codegen.Image, ident str, image List[u8]) List[u8]  ## `macho` with the start-up image `image` (empty: none) before the globals.
pub obj.PE_ARM64 := 43620  ## IMAGE_FILE_MACHINE_ARM64 (aarch64-windows, ADR 0968).
pub struct obj.PeDll  ## One DLL and the functions the program imports from it.
    pub name str
    pub funcs List[str]
pub fn obj.pe(img codegen.Image, dlls List[obj.PeDll], machine int) List[u8]  ## A PE32+ executable for `img` that imports `dlls`. The entry point is
pub fn obj.pe_image(img codegen.Image) List[u8]  ## A PE32+ executable for `img`, with its imports taken from the `imp:NAME`
pub fn obj.pe_image_for(img codegen.Image, machine int) List[u8]  ## `pe_image` for `machine`: `PE_ARM64` for the ARM64 backend, whose
pub fn obj.pe_sample() List[u8]  ## A sample program for the writer's test (`ail run compiler pe-sample`):
pub fn obj.sha256(bytes List[u8], from int, to int) List[u8]  ## The digest of `bytes[from..to]`, as 32 bytes.
```

trimmed to fit 2000 tokens (~668); dropped: private items
