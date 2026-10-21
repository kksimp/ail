# garlic: html

## html
```
pub enum html.Kind  ## A node's kind.
    Document
    Element
    Text
    Comment
pub struct html.Attr  ## One attribute; names are lowercase.
    pub name str
    pub value str
pub struct html.Node  ## One node of the tree: `tag` is the lowercase element name, `text` the
    pub kind html.Kind
    pub tag str
    pub attrs List[html.Attr]
    pub text str
    pub parent int
    pub children List[int]
pub enum html.Token  ## A token, as the tokenizer emits it (13.2.5).
    Start(name str, attrs List[html.Attr], self_closing bool)
    End(name str)
    Chars(text str)
    Note(text str)
    Doctype(name str)
pub struct html.Doc  ## A parsed document.
    pub nodes List[html.Node]
    pub doctype str
    pub changed List[int]  ## Nodes changed since parsing (by scripts or form input), in order,
    pub fn html.Doc.attr(self, id int, name str) str?  ## The value of attribute `name` of node `id`, or none.
    pub fn html.Doc.all(self, tag str) List[int]  ## Every element named `tag`, in document order.
    pub fn html.Doc.first(self, tag str) int?  ## The first element named `tag`, or none.
    pub fn html.Doc.set_attr(mut self, id int, name str, value str) [mutates]  ## Sets attribute `name` of node `id` to `value`, adding it if absent.
    pub fn html.Doc.remove_attr(mut self, id int, name str) [mutates]  ## Removes attribute `name` from node `id`, if it has it.
    pub fn html.Doc.mark(mut self, id int) [mutates]  ## Records that node `id` changed.
    pub fn html.Doc.create(mut self, tag str) int [mutates]  ## A new element named `tag`, not yet in the tree (parent -1).
    pub fn html.Doc.create_text(mut self, s str) int [mutates]  ## A new text node, not yet in the tree.
    pub fn html.Doc.create_comment(mut self, s str) int [mutates]  ## A new comment node, not yet in the tree.
    pub fn html.Doc.append(mut self, parent int, id int) int [mutates]  ## Moves node `id` to the end of `parent`'s children; gives `id`.
        requires not within(parent, id)
    pub fn html.Doc.insert_before(mut self, parent int, id int, before int) int [mutates]  ## Moves node `id` into `parent` just before child `before` (at the end
        requires not within(parent, id)
    pub fn html.Doc.detach(mut self, id int) [mutates]  ## Takes node `id` out of the tree (it keeps its own children).
    pub fn html.Doc.set_text(mut self, id int, s str) [mutates]  ## Replaces the content of node `id` with `s`: a text node's text, or an
    pub fn html.Doc.within(self, id int, ancestor int) bool  ## Whether node `id` is `ancestor` or inside it.
    pub fn html.Doc.enclosing(self, id int, tag str) int?  ## The nearest element named `tag` around node `id` (or `id` itself), or none.
    pub fn html.Doc.visible_text(self, id int) str  ## The text of node `id` and its descendants that a reader would see:
    pub fn html.Doc.text(self, id int) str  ## The text of node `id` and its descendants, concatenated.
    pub fn html.Doc.title(self) str  ## The document's title, whitespace collapsed; "" when it has none.
    pub fn html.Doc.dump(self) str  ## The tree in the html5lib-tests format: one node per line, `| ` and two
pub html.VOID := ...  ## Elements that never have content (13.1.2).
pub fn html.is_space(c char) bool  ## Whether `c` is HTML whitespace (13.2.5: tab, LF, FF, CR, space).
pub fn html.windows_1252(n int) char  ## The character byte `n` stands for in windows-1252, which is what HTML
pub fn html.tokenize(src str) List[html.Token]  ## Splits `src` into tokens (13.2.5).
pub fn html.build(tokens List[html.Token]) html.Doc  ## Builds the tree from tokens: a simplified 13.2.6 with implied `html`,
pub fn html.parse(src str) html.Doc  ## Parses an HTML document.
pub fn html.collapse(s str) str  ## `s` with each run of whitespace made one space, trimmed.
```

trimmed to fit 2000 tokens (~1282); dropped: private items
