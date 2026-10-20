//! Facts the checker records for `ail api --locals/--type` (api.loc.1,
//! api.type.1, ADR 0258). Recording is off except for `ail api`, so a normal
//! check pays one branch per expression and local.

use crate::diag::Pos;

/// A local name and its type, as the checker left it when the name went
/// out of scope (or the function ended).
#[derive(Debug, Clone)]
pub struct LocalFact {
    pub name: String,
    /// The type as written in a diagnostic, `?` when it is not known.
    pub ty: String,
    pub mutable: bool,
    /// `param`, `self`, `loop` or `binding`.
    pub kind: &'static str,
    /// Where the name is written; `None` for `self`.
    pub at: Option<Pos>,
    /// Where a `:=` binding's name is written (it is in scope after it).
    pub decl: Option<Pos>,
    /// The first position the name is in scope from: its declaration for a
    /// `:=` binding, the first statement of the block it scopes otherwise.
    pub anchor: Pos,
}

/// What the checker recorded about one function-level pass.
#[derive(Debug, Default)]
pub struct Facts {
    pub locals: Vec<LocalFact>,
    /// The type of each expression, innermost first: `(start, type)`. A
    /// method call or field read is also recorded at its member name.
    pub exprs: Vec<(Pos, String)>,
}
