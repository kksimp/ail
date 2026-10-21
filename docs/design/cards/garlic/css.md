# garlic: css

## css
```
pub struct css.Decl  ## One declaration: `name: value [!important]`, the name lowercase.
    pub name str
    pub value str
    pub important bool
pub struct css.Compound  ## A compound selector: an optional type and any ids, classes and
    pub tag str
    pub ids List[str]
    pub classes List[str]
    pub attrs List[css.AttrTest]
pub struct css.AttrTest  ## `[name]` (any value) or `[name=value]`.
    pub name str
    pub value str?
pub struct css.Selector  ## A complex selector: compounds left to right; `child[k]` says whether
    pub parts List[css.Compound]
    pub child List[bool]
    pub specificity int
pub struct css.Rule  ## A style rule; `order` is its position, for the cascade. `media` holds
    pub selector css.Selector
    pub decls List[css.Decl]
    pub order int
    pub media str
pub struct css.Sheet  ## A parsed style sheet: rules indexed by their rightmost compound's id,
    pub rules List[css.Rule]
    pub by_id Map[str, List[int]]
    pub by_class Map[str, List[int]]
    pub by_tag Map[str, List[int]]
    pub general List[int]
css.USED := ...  ## The properties Garlic uses on text cells; the rest are dropped at parse.
fn css.strip_comments(src str) str  ## Removes `/* ... */` comments.
pub fn css.parse(src str, start int) css.Sheet  ## Parses a style sheet. Rules after an index of `start` keep their order
pub fn css.parse_in(src str, start int, media str) css.Sheet  ## Parses a style sheet whose rules all sit in the media queries `media`
fn css.rules_of(cs List[char], media str, start int, mut rules List[css.Rule]) [mutates]  ## Adds the rules in `cs` to `rules`, inside media queries `media`.
pub fn css.index(rules List[css.Rule]) css.Sheet  ## Builds the lookup maps of a sheet's rules.
pub fn css.concat(a css.Sheet, b css.Sheet) css.Sheet  ## Two sheets as one, `b`'s rules after `a`'s.
pub fn css.for_width(s css.Sheet, px int) css.Sheet  ## The rules of `s` that apply in a window `px` CSS pixels wide.
fn css.all_match(media str, px int) bool
pub fn css.media_matches(query str, px int) bool  ## Whether media query list `query` matches a screen `px` CSS pixels wide
fn css.one_matches(query str, px int) bool
fn css.feature_matches(f str, px int) bool
fn css.length_px(s str) int?  ## A length in CSS pixels: `N`, `Npx`, `Nem` or `Nrem`, with a fraction.
fn css.block_end(cs List[char], open int) int  ## The index just past the matching `}` of the block opening at `open`
fn css.string_end(cs List[char], at int) int  ## The index just past the string starting at `at`.
fn css.skip_at_rule(cs List[char], at int) int  ## Skips an at-rule from `@`: up to `;`, or past its block.
pub fn css.declarations(body str) List[css.Decl]  ## The declarations of a block body that Garlic uses.
pub fn css.selector(text str) css.Selector?  ## Parses one complex selector, or none when Garlic doesn't support it
fn css.compound(word str) css.Compound?  ## Parses a compound selector such as `a.nav#top[href]`.
fn css.matches_compound(d html.Doc, id int, c css.Compound) bool  ## Whether element `id` matches compound `c`.
pub fn css.matches(d html.Doc, id int, s css.Selector) bool  ## Whether element `id` matches selector `s`, right to left.
fn css.matches_from(d html.Doc, id int, s css.Selector, k int) bool
fn css.candidates(d html.Doc, id int, sheet css.Sheet) List[int]  ## The rules of `sheet` that may match element `id`, by the index.
pub fn css.cascade(d html.Doc, id int, sheet css.Sheet) Map[str, str]  ## The cascaded value of each property Garlic uses on element `id`:
pub fn css.document_sheet(d html.Doc) css.Sheet  ## Every `<style>` element's sheet, in document order, as one sheet.
```
