//! `ail api --expanded` (ct.gen.4, ADR 0228, ADR 0308): the methods derive
//! generators wrote, with their origin and text, and which `impl`s are
//! built-in derives, listed by name (OQ-096 option b).

use super::*;

impl<'p> Api<'p> {
    /// The generated code of `im`, if a generator filled it in.
    fn generated(&self, im: &ImplDecl) -> Option<&'p Generated> {
        self.prog.generated.iter().find(|g| g.impl_pos == im.pos)
    }

    /// Whether `pos` is in the text `g` generated.
    fn in_generated(&self, g: Option<&Generated>, pos: Pos) -> bool {
        g.is_some_and(|g| {
            let base = self.prog.files[g.file].base;
            pos.line > base && pos.line <= base + FILE_LINES
        })
    }

    /// The methods listed under an `impl`: those written in it, and with
    /// `--expanded` those its generator wrote, including associated
    /// functions of the type (ct.gen.1), each with `origin` and `text`.
    pub(super) fn impl_methods(&self, im: &ImplDecl) -> Vec<Item> {
        let prog = self.prog;
        let g = self.generated(im);
        let mut items: Vec<Item> =
            im.methods.iter().filter(|&&m| !self.in_generated(g, prog.fns[m].pos)).map(|&m| self.fn_item(m)).collect();
        let Some(g) = g.filter(|_| self.expanded) else { return items };
        let file = &prog.files[g.file];
        let mut fns: Vec<usize> = (0..prog.fns.len()).filter(|&i| self.in_generated(Some(g), prog.fns[i].pos)).collect();
        fns.sort_by_key(|&i| prog.fns[i].pos.line);
        let lines: Vec<&str> = file.src.lines().collect();
        for (n, &i) in fns.iter().enumerate() {
            let pos = prog.fns[i].pos;
            let start = pos.line - file.base;
            let end = fns.get(n + 1).map_or(lines.len() + 1, |&j| prog.fns[j].pos.line - file.base);
            let text = lines[start - 1..end - 1].join("\n");
            let mut item = self.fn_item(i);
            item.pos = Pos { line: start, col: pos.col };
            item.extra.push((
                "origin",
                Json::Obj(vec![
                    ("kind", Json::Str("derive".into())),
                    ("impl", Json::Str(format!("impl:{}", g.label))),
                    ("generator", Json::Str(format!("fn:{}", g.generator))),
                    ("at", Json::Str(format!("gen:impl:{}:{start}:{}", g.label, pos.col))),
                ]),
            ));
            item.extra.push(("text", Json::Str(format!("{}\n", text.trim_end()))));
            items.push(item);
        }
        items
    }

    /// With `--expanded`, how an `impl` without written methods is derived:
    /// by its generator, or `built-in` for `Eq`, `Ord` and `ToStr`
    /// (trait.derive.1, OQ-096).
    pub(super) fn derive_extra(&self, im: &ImplDecl) -> Vec<(&'static str, Json)> {
        if !self.expanded {
            return Vec::new();
        }
        if let Some(g) = self.generated(im) {
            return vec![("derived_by", Json::Str(format!("fn:{}", g.generator)))];
        }
        let built_in = self
            .prog
            .traits
            .iter()
            .any(|t| t.name == im.trait_name && t.derive.is_none() && crate::ast::is_prelude(t.pos));
        if im.methods.is_empty() && built_in {
            return vec![("derived_by", Json::Str("built-in".into()))];
        }
        Vec::new()
    }
}
