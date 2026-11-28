# compiler: (root)
entry: main in main.ail

## (root)
```
fn main() ! [clock env fail fs io net proc random]
    fails with JsonError | fs.IoError
STANDALONE_HELP := ...
fn standalone_root() str [env fs io]
fn standalone(args List[str]) [env fs io proc]
PERF_USAGE := "usage: ail perf FILE [ARGS...] | ail perf --explain FN FILE"
struct PerfLine
    path str
    line int
    kind str
    text str
fn perf_command(rest List[str]) int [clock env fs io proc]
fn perf_par(r check.ParRow) str
BUILD_USAGE := ...
fn positive(v str) bool
fn fwd(p str) str [env]
fn host_target() str [env io proc]
fn build_cli(rest List[str]) [clock env fs io proc random]
fn compiler_dir() str [env io]
fn compile_to(file str, out_arg str, target_arg str, ms str, mb str, attach bool) [clock env fs io proc random]
fn cache_store(file str, exe str, target str, ms str, mb str, attach bool) [clock env fs io proc random]
fn cached_copy(exe str, out str, target str) bool [fs io]
fn compile_fresh(file str, out str, target str, ms str, mb str, attach bool) [clock env fs io proc random]
IMAGE_NOTE := "image\tdump"
fn image_on(target str) bool [env io proc]
fn image_make(code codegen.Image, ident str) List[u8] [clock env fs io proc random]
fn image_none(why str) List[u8] [env io]
fn image_list(a bytes) List[u8] [env io]
fn finish_build(m mir.Module, file str, out str, target str) [env fs io]
fn write_exe(exe bytes, out str, target str) [fs io]
fn check_dump(path str, many bool) ! [env fail fs io]
    fails with JsonError | fs.IoError
fn runtime(dir str, path str) !ast.Program [clock env fail fs io proc]
    fails with fs.IoError
fn runtime_with(dir str, path str, ms str, mb str) !ast.Program [clock env fail fs io proc]
    fails with fs.IoError
fn source_text(path str) !str [fail fs io]
    fails with fs.IoError
fn imported_text(file_arg str, text str, std str) str [env fs io]
fn executable_hex(m mir.Module, path str, target str) str [env io]
fn executable_bytes(m mir.Module, path str, target str) bytes [env io]
fn executable_list(m mir.Module, path str, target str) List[u8] [env io]
fn runtime_syntax(path str, text str) [io]
fn lower_file(path str, rt ast.Program?, std str) !mir.Module [clock env fail fs io proc]
    fails with fs.IoError
fn lower_file_with(path_arg str, rt ast.Program?, std str, over str) !mir.Module [clock env fail fs io proc]
    fails with fs.IoError
fn lower_file_mode(path_arg str, rt ast.Program?, std str, over str, mode str) !mir.Module [clock env fail fs io proc]
    fails with fs.IoError
PROOF_MAX_BYTES := 1500000
fn par_keys(file str, std str, size int) Set[str] [clock env fs io proc]
fn parallel_on() bool [env]
fn proven_loop_keys(file str, std str) Set[str] [clock env fs io proc]
fn mentions_prelude(srcs List[mir.Source]) bool
fn std_file(std str, imp str) str
fn derive_builtin(srcs List[mir.Source]) List[mir.Source] [io]
fn derive_user(srcs List[mir.Source], rt ast.Program?, std str) List[mir.Source] [clock env fs io proc]
fn run_generators(srcs List[mir.Source], jobs List[derive.Job], rt ast.Program?, quiet bool) List[str] [clock env fs io proc]
fn documented(srcs List[mir.Source]) List[ast.Program]
fn derive_failure(label str) str
fn remove_quietly(path str) [fs io]
fn user_main_renamed(s mir.Source) mir.Source
fn user_derive_texts(loaded load.Loaded, path_arg str) Map[str, str] [clock env fs io proc]
fn derived_source(module str, label str, body List[str]) mir.Source [io]
fn uses_regex(prog ast.Program) bool
fn uses_ui(prog ast.Program) bool
fn mentions_ui(text str) bool
fn parse_source(module str, path str) !mir.Source [fail fs io]
    fails with fs.IoError
fn parse_module(module str, paths List[str]) !List[mir.Source] [fail fs io]
    fails with fs.IoError
fn prelude_generic_names() Set[str]
fn parse_text(module str, path str, text str) mir.Source [io]
PRELUDE_GENERIC := ...
fn parse_with(module str, path str, text str, names Set[str]) mir.Source [io]
fn parse_aliased(module str, path str, text str, names Set[str], aliases Set[str]) mir.Source [io]
CACHE_KEEP := 64
fn cache_dir() str [env]
fn tree_stamp(tag str, root str) str [fs io]
fn sched_points() bool [env]
fn exe_ext(target str) str
fn cache_key(file str, target str, ms str, mb str, attach bool) str [env fs io]
fn own_build() str [env fs io]
fn cache_touch(exe str) bool [clock fs io]
fn cache_trim(dir str) [fs io]
fn cli_prepare(file str) str [clock env fs io proc random]
fn cli_prepare_with(file str, ms str, mb str, attach bool) str [clock env fs io proc random]
fn cli_prepare_cmd(rest List[str]) [clock env fs io proc random]
fn cli_run(rest List[str]) [clock env fs io proc random]
fn chop(s str) str
fn load_dump(files List[str]) [clock env fs io proc]
fn check_cmd(rest List[str]) [clock env fs io proc]
fn api_syntax(rest List[str]) [clock env fs io proc]
fn api_cmd(rest List[str]) [clock env fs io proc random]
```

trimmed to fit 2000 tokens (~1687); dropped: private items, doc comments
