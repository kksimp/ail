# compiler
entry: main in main.ail

## (root)
```
fn main() ! [env fail fs io]
fn check_dump(str, bool) ! [env fail fs io]
fn runtime(str) !ast.Program [fail fs io]
fn runtime_with(str, str, str) !ast.Program [fail fs io]
fn runtime_syntax(str, str) [io]
fn lower_file(str, ast.Program?, str) !mir.Module [fail fs io]
fn std_file(str, str) str
fn parse_source(str, str) !mir.Source [fail fs io]
```

## a64
```
pub a64.SP := 31
pub a64.XZR := 31
pub a64.FP := 29
pub a64.LR := 30
pub a64.EQ := 0
pub a64.NE := 1
pub a64.HS := 2
pub a64.LO := 3
pub a64.MI := 4
pub a64.PL := 5
pub a64.VS := 6
pub a64.VC := 7
pub a64.HI := 8
pub a64.LS := 9
pub a64.GE := 10
pub a64.LT := 11
pub a64.GT := 12
pub a64.LE := 13
pub struct a64.Fixup
pub struct a64.Asm
pub fn a64.new_asm() a64.Asm
pub fn a64.fits_imm12(int) bool
pub fn a64.invert(int) int
pub fn a64.patch(mut List[int], a64.Fixup, int, int) [mutates]
pub fn a64.registers() codegen.Target
pub a64.LINUX := 1
pub a64.MACOS := 2
pub fn a64.generate(mir.Module, int) codegen.Image
```

## ast
```
pub struct ast.Pos
pub struct ast.Ident
pub struct ast.Program
pub struct ast.SourceFile
pub struct ast.Grant
pub struct ast.Generated
pub struct ast.Use
pub struct ast.ConstDecl
pub struct ast.GenericParam
pub struct ast.TraitDecl
pub struct ast.ImplDecl
pub struct ast.StructDecl
pub struct ast.Field
pub struct ast.EnumDecl
pub struct ast.Variant
pub struct ast.FnDecl
pub struct ast.Extern
pub struct ast.Contract
pub struct ast.Old
pub struct ast.Attr
pub enum ast.Within
pub struct ast.Owner
pub struct ast.Receiver
pub struct ast.Param
pub struct ast.TypeRef
pub enum ast.TypeShape
pub enum ast.Stmt
pub enum ast.Else
pub enum ast.BinOp
pub enum ast.UnOp
pub enum ast.Expr
pub struct ast.MapEntry
pub struct ast.Branch
pub enum ast.CompKind
pub struct ast.CompFor
pub enum ast.Cond
pub enum ast.InterpPart
pub struct ast.FormatSpec
pub struct ast.Arg
pub struct ast.Arm
pub enum ast.Pattern
pub enum ast.Literal
```

## check
```
pub fn check.expr_pos(ast.Expr) ast.Pos
pub fn check.stmt_pos(ast.Stmt) ast.Pos
pub enum check.Step
pub struct check.Place
pub enum check.ArgKind
pub struct check.Slot
pub fn check.field_slots(List[ast.Field], List[ast.Arg]) List[check.Slot]
pub fn check.op_symbol(ast.BinOp) str
pub fn check.full_name(ast.FnDecl) str
pub fn check.has_attr(ast.FnDecl, str) bool
pub fn check.module_at(ast.Program, ast.Pos) str
pub fn check.hidden(ast.Program, ast.Pos, ast.Pos) bool
pub struct check.Report
pub fn check.check_program(ast.Program) check.Report [env io]
pub fn check.check_declarations(ast.Program) List[check.Diag] [env io]
pub enum check.Need
pub struct check.Builtin
pub check.STR_METHODS := ...
pub check.BYTES_METHODS := ...
pub check.CHAR_METHODS := ...
pub check.INT_METHODS := ...
pub check.FLOAT_METHODS := ...
pub check.SIZED_INT_METHODS := ...
pub check.F32_METHODS := ...
pub check.LIST_METHODS := ...
pub check.MAP_METHODS := ...
pub check.SET_METHODS := ...
pub enum check.Cand
pub struct check.TParam
pub struct check.Scoped
pub struct check.ImplInfo
pub struct check.SigParam
pub struct check.Sig
pub struct check.Deferred
pub struct check.Checker
pub enum check.LocalKind
pub struct check.Local
pub struct check.Pending
pub struct check.Moved
pub fn check.new_checker(ast.Program) check.Checker
pub check.COVERED_RULES := ...
pub struct check.Skip
pub fn check.skipped(ast.Program, check.Report) List[check.Skip]
pub struct check.Note
pub struct check.Diag
pub fn check.new_diag(str, ast.Pos, str) check.Diag
pub struct check.VariantRef
pub struct check.Agree
pub struct check.Binding
pub struct check.Coverage
pub fn check.check_handles(ast.Program) List[check.Diag]
pub fn check.check_moves(ast.Program, List[check.Moved]) List[check.Diag]
pub check.PRELUDE_FNS := ...
pub check.BUILTIN_TYPES := ...
pub check.SPEC_TYPES_NOT_YET := ...
pub check.GENERATABLE := ...
pub check.SIZED := ...
pub check.CONC_TYPES := ...
pub check.KNOWN_ATTRS := ...
pub check.STD_MODULES := ...
pub check.AIL_STD_MODULES := ...
pub check.PRELUDE_LINE := 1073741824
pub fn check.is_prelude_line(int) bool
pub fn check.list_names(List[str]) str
pub fn check.count(int, str) str
pub fn check.was_were(int) str
pub fn check.suggest(str, List[str]) str?
pub fn check.edit_distance(str, str) int
pub fn check.foreign_type_word(str) str?
pub enum check.Ty
pub enum check.Errs
pub enum check.Def
pub struct check.Bind
pub fn check.sort_errors(List[check.Ty]) List[check.Ty]
pub fn check.subst_all(List[check.Ty], List[check.Bind]) List[check.Ty]
```

## codegen
```
pub codegen.FLOAT_BASE := 32
pub codegen.SLOT_BASE := 1000
pub codegen.REMAT := -2
pub codegen.NOWHERE := -1
pub struct codegen.Target
pub struct codegen.Alloc
pub fn codegen.is_remat(mir.Inst) bool
pub fn codegen.is_call(mir.Inst) bool
pub fn codegen.is_push(mir.Inst) bool
pub fn codegen.caller_used(codegen.Alloc, codegen.Target) List[int]
pub fn codegen.allocate(mir.Func, codegen.Target) codegen.Alloc
pub struct codegen.Reloc
pub struct codegen.Image
pub fn codegen.new_image() codegen.Image
pub fn codegen.add_str(mut codegen.Image, str, str) [mutates]
pub fn codegen.add_f64(mut codegen.Image, float) str [mutates]
pub fn codegen.align(mut List[int], int) [mutates]
pub fn codegen.push64(mut List[int], int) [mutates]
pub fn codegen.utf8(str) List[int]
pub fn codegen.add_trap_texts(mut codegen.Image, mir.Module, int, str) [mutates]
pub fn codegen.split_edges(mir.Func) mir.Func
pub fn codegen.block_order(mir.Func) List[int]
pub fn codegen.f64_bits(float) int
pub fn codegen.split_for_calls(mir.Func) mir.Func
pub fn codegen.wrap_block(mir.Func, List[int], List[List[bool]], List[int]) int
pub fn codegen.div_magic(int) List[int]
```

## mir
```
pub fn mir.fold_addresses(mir.Func) mir.Func
pub struct mir.Versioned
pub fn mir.bce(mir.Func, bool, bool) mir.Func
pub fn mir.lists_only_grow(mir.Module) bool
pub fn mir.fold(mir.Func) mir.Func
pub fn mir.inline_module(mir.Module) mir.Module
pub fn mir.merge_blocks(mir.Func) mir.Func
pub struct mir.Def
pub fn mir.defs_of(mir.Func) List[mir.Def]
pub fn mir.inst_of(mir.Func, List[mir.Def], int) mir.Inst?
pub fn mir.const_of(mir.Func, List[mir.Def], int) int?
pub struct mir.LoopInfo
pub fn mir.find_loops(mir.Func) List[mir.LoopInfo]
pub fn mir.is_innermost(List[mir.LoopInfo], mir.LoopInfo) bool
pub struct mir.Counted
pub fn mir.counted(mir.Func, List[mir.Def], mir.LoopInfo) mir.Counted?
pub struct mir.Sig
pub struct mir.Problem
pub error struct mir.Problems
pub impl ToStr for mir.Problems
pub fn mir.lower_module(str, str, ast.Program, ast.Program?) mir.Module ! mir.Problems [fail]
pub struct mir.Source
pub fn mir.lower_package(List[mir.Source], ast.Program?) mir.Module ! mir.Problems [fail]
pub fn mir.expr_pos(ast.Expr) ast.Pos
pub mir.RT_GLOBALS := 64
pub fn mir.aliases_of(ast.Program) Map[str, str]
pub fn mir.imports_of(ast.Program) List[str]
pub enum mir.Ty
pub struct mir.Inst
pub fn mir.inst(int, str, List[int]) mir.Inst
pub struct mir.Term
pub fn mir.jump_to(int, List[int]) mir.Term
pub fn mir.branch_on(int, int, int) mir.Term
pub fn mir.return_of(int) mir.Term
pub fn mir.unreachable_term() mir.Term
pub struct mir.Block
pub struct mir.Func
pub struct mir.Site
pub struct mir.Module
pub mir.LIMIT := 1073741824
pub struct mir.Proven
pub fn mir.prove(mir.Func, List[mir.Def], mir.LoopInfo, mir.Counted, List[int], List[bool]) mir.Proven
pub fn mir.reuse_lists(mir.Module) mir.Module
pub fn mir.simplify(mir.Func) mir.Func
pub fn mir.subset_help(str) str
pub fn mir.module_text(mir.Module) str
pub fn mir.func_text(mir.Func) str
pub fn mir.verify(mir.Func) List[str]
pub fn mir.dominators(mir.Func) List[List[bool]]
```

## obj
```
pub obj.X86_64 := 62
pub obj.AARCH64 := 183
pub fn obj.elf(codegen.Image, int) List[int]
pub fn obj.link(codegen.Image, int, int, int) List[int]
pub fn obj.align_up(int, int) int
pub fn obj.put(mut List[int], int, int) [mutates]
pub fn obj.hex(List[int]) str
pub fn obj.macho(codegen.Image, str) List[int]
pub struct obj.PeDll
pub fn obj.pe(codegen.Image, List[obj.PeDll]) List[int]
pub fn obj.pe_sample() List[int]
pub fn obj.sha256(List[int], int, int) List[int]
```

## syntax
```
pub fn syntax.tokens_text(syntax.Lexed) str
pub fn syntax.diag_lines(List[syntax.Diag], mut List[str]) [mutates]
pub fn syntax.one_per_line(List[syntax.Diag]) List[syntax.Diag]
pub fn syntax.quote(str) str
pub fn syntax.ast_text(syntax.Lexed, syntax.Parsed) str
pub fn syntax.report_text(str, str, List[syntax.Diag]) str
pub enum syntax.Tok
pub enum syntax.Part
pub struct syntax.Token
pub struct syntax.FormatSpec
pub struct syntax.Diag
pub struct syntax.Comment
pub struct syntax.Lexed
pub fn syntax.lex(str) syntax.Lexed
pub struct syntax.Parsed
pub fn syntax.parse(List[syntax.Token]) syntax.Parsed
```

## x64
```
pub x64.RAX := 0
pub x64.RCX := 1
pub x64.RDX := 2
pub x64.RBX := 3
pub x64.RSP := 4
pub x64.RBP := 5
pub x64.RSI := 6
pub x64.RDI := 7
pub x64.R8 := 8
pub x64.R9 := 9
pub x64.R10 := 10
pub x64.R11 := 11
pub x64.CC_O := 0
pub x64.CC_B := 2
pub x64.CC_AE := 3
pub x64.CC_E := 4
pub x64.CC_NE := 5
pub x64.CC_BE := 6
pub x64.CC_A := 7
pub x64.CC_P := 10
pub x64.CC_NP := 11
pub x64.CC_L := 12
pub x64.CC_GE := 13
pub x64.CC_LE := 14
pub x64.CC_G := 15
pub struct x64.Mem
pub fn x64.at(int, int) x64.Mem
pub fn x64.at_index(int, int, int, int) x64.Mem
pub fn x64.at_sym(str, int) x64.Mem
pub struct x64.Fixup
pub struct x64.Asm
pub fn x64.new_asm() x64.Asm
pub fn x64.fits32(int) bool
pub fn x64.alu_digit(str) int
pub fn x64.sd_opcode(str) int
pub fn x64.registers() codegen.Target
pub fn x64.generate(mir.Module) codegen.Image
```

trimmed to fit 4000 tokens (~3656); dropped: private items, doc comments, parameter names, members, contracts and examples (docs past their first sentence), fields, variants and test names
