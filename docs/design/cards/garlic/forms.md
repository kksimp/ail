# garlic: forms
tests: forms.forms.a_plan_types_selects_checks_and_submits_by_label, forms.forms.the_schema_lists_labelled_fields_options_and_buttons

## forms
```
pub struct forms.Step  ## One thing to do to a control: `kind` is `type`, `check`, `select` or
    pub kind str
    pub id int
    pub arg str
pub fn forms.field_label(d html.Doc, id int) str  ## What the control is called: its label, else aria-label, placeholder, name.
fn forms.is_button(d html.Doc, id int) bool  ## `input` types that are not fields to fill.
fn forms.is_field(d html.Doc, id int) bool
fn forms.kind_of(d html.Doc, id int) str
fn forms.field_json(d html.Doc, id int) str
pub fn forms.schema(d html.Doc) str  ## Every form of the page as `{"forms":[{"id","action","method","fields","submit"}]}`.
fn forms.find_field(d html.Doc, want str) int?  ## The field whose label is `want` (ignoring case), else the first whose label contains it.
pub fn forms.plan(d html.Doc, spec List[(str, str)], mut missing List[str]) List[forms.Step] [mutates]  ## The steps that set each `label: value` of `spec` and then submit the form of the
```
