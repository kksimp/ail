# compiler: obj

## obj
```
obj.BASE := 4194304
obj.PAGE := 65536  ## The segment alignment: 64 KiB covers both 4 KiB and 64 KiB page kernels.
obj.EHDR := 64
obj.PHDR := 56
pub obj.X86_64 := 62  ## ELF machine numbers.
pub obj.AARCH64 := 183
pub fn obj.elf(img codegen.Image, machine int) List[u8]
fn obj.symbols(mut out List[u8], img codegen.Image, text_addr int, text_off int) [mutates]  ## `.text`, `.symtab`, `.strtab` and `.shstrtab`, then the section headers,
fn obj.elf_section(mut out List[u8], name int, kind int, flags int, addr int, offset int, size int, linked int, info int, align int, entsize int) [mutates]
fn obj.patch(mut out List[u8], at int, v int, n int) [mutates]  ## Overwrite `n` little-endian bytes at `at`.
fn obj.header(mut out List[u8], machine int, entry int) [mutates]
fn obj.header_n(mut out List[u8], machine int, entry int, nphdr int) [mutates]
fn obj.segment(mut out List[u8], flags int, offset int, vaddr int, filesz int, memsz int) [mutates]  ## A `PT_LOAD` program header; `flags` is R 4, W 2, X 1.
fn obj.interpreter(machine int) str  ## The dynamic linker of a dynamic executable, by machine.
fn obj.cstr(s str) List[u8]  ## A NUL-terminated string's bytes.
fn obj.elf_dynamic(img codegen.Image, machine int) List[u8]  ## A dynamic ELF executable (ffi.load.1, ADR 0870): the same two segments as
fn obj.dynamic_entry(mut out List[u8], tag int, value int) [mutates]
fn obj.phdr(mut out List[u8], kind int, flags int, offset int, vaddr int, filesz int, memsz int, align int) [mutates]  ## A non-load program header.
pub fn obj.link(img codegen.Image, code int, data int, bss int) List[u8]  ## The code with every relocation patched, for code at address `code`,
fn obj.address(img codegen.Image, sym str, code int, data int, bss int) int  ## The address of a symbol.
fn obj.read32(bytes List[u8], at int) int
fn obj.signed32(v int) int
pub fn obj.align_up(n int, a int) int
pub fn obj.put(mut out List[u8], v int, n int) [mutates]  ## `v` in `n` little-endian bytes.
pub fn obj.bytes_of(xs List[int]) List[u8]  ## The bytes of a list of byte values, each 0 to 255.
pub fn obj.hex(bytes List[u8]) str  ## Bytes as lines of hexadecimal text, for the driver to write out until
obj.MACHO_BASE := 4294967296
obj.MACHO_PAGE := 16384
obj.HEADER := 32
pub fn obj.macho(img codegen.Image, ident str) List[u8]  ## The executable, signed with `ident` as its identifier.
obj.COMMANDS := 11
fn obj.bind_info(img codegen.Image) List[u8]  ## The bind table: each import's pointer slot in __DATA (segment 2), bound to
fn obj.uleb(mut b List[u8], n int) [mutates]
fn obj.load_commands(v List[int]) List[u8]  ## The load commands, from the layout `v`: the sizes and offsets of
fn obj.dylib_name(mut c List[u8], cmd int, name str, at int, size int) [mutates]  ## LC_LOAD_DYLINKER-shaped: a command with one string at offset `at`.
fn obj.text(mut c List[u8], s str, n int) [mutates]  ## `s` padded with zeros to `n` bytes.
fn obj.segment64(mut c List[u8], name str, vmaddr int, vmsize int, fileoff int, filesize int, prot int, nsects int) [mutates]
fn obj.section(mut c List[u8], name str, seg str, addr int, size int, offset int, align int, flags int) [mutates]
fn obj.pad_to(mut out List[u8], n int) [mutates]
fn obj.put_be(mut out List[u8], v int, n int) [mutates]  ## Big-endian, as code signatures are.
fn obj.sign(mut out List[u8], name List[u8], slots int, text_size int) [mutates]  ## Append an ad hoc signature: a superblob holding one code directory that
obj.PE_MACHINE := 34404
obj.PE_BASE := 5368709120
obj.PE_SECTION_ALIGN := 4096
obj.PE_FILE_ALIGN := 512
obj.PE_HEADERS := 1024
pub struct obj.PeDll  ## One DLL and the functions the program imports from it.
    pub name str
    pub funcs List[str]
struct obj.PeImports  ## The import directory laid out at RVA `base`: `bytes` (the descriptors,
    bytes List[u8]
    iat int
    iat_size int
    dir_size int
    slots Map[str, int]
pub fn obj.pe(img codegen.Image, dlls List[obj.PeDll]) List[u8]  ## A PE32+ executable for `img` that imports `dlls`. The entry point is
fn obj.pe_section(mut out List[u8], name str, size int, rva int, raw int, at int, flags int) [mutates]  ## One 40-byte section header: the name padded to 8 bytes, the virtual
fn obj.pe_imports(dlls List[obj.PeDll], base int) obj.PeImports  ## The import directory at RVA `base` (ADR 0598): one descriptor per DLL and
pub fn obj.pe_image(img codegen.Image) List[u8]  ## A PE32+ executable for `img`, with its imports taken from the `imp:NAME`
fn obj.pe_dll_of(name str) str  ## The DLL that exports `name`. kernel32 is the default; the socket and random
obj.PE_DLLS := ...  ## The DLLs `pe_dll_of` can return, in the order their descriptors are written.
fn obj.pe_resolve(img codegen.Image, slots Map[str, int], code_rva int, rdata int, bss int) List[u8]  ## The code with its relocations resolved, for `.text` at RVA `code_rva`,
fn obj.pe_address(img codegen.Image, slots Map[str, int], code_rva int, rdata int, bss int, sym str) int  ## The RVA of a relocation's symbol.
fn obj.pe_pad(mut out List[u8], n int) [mutates]  ## Zero bytes, up to `n` in all.
fn obj.pe_read32(bytes List[u8], at int) int
fn obj.pe_signed32(v int) int
pub fn obj.pe_sample() List[u8]  ## A sample program for the writer's test (`ail run compiler pe-sample`):
fn obj.pe_call_import(mut img codegen.Image, name str) [mutates]  ## `call qword ptr [rip + slot]` through the import `name`: FF 15 and a rel32.
obj.K := ...
obj.M32 := 4294967295
fn obj.rotr(x int, n int) int
pub fn obj.sha256(bytes List[u8], from int, to int) List[u8]  ## The digest of `bytes[from..to]`, as 32 bytes.
```
