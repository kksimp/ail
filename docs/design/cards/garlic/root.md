# garlic: (root)
entry: main in garlic.ail
tests: a_meta_refresh_moves_to_its_target_at_once, a_slow_meta_refresh_stays_put, a_wrapped_link_spans_rows, agent_actions_follow_click_back_and_report, any_input_parses, attributes_quoted_unquoted_bare_and_duplicate, bodies_decode_by_charset_and_content_encoding, browser_finds_text_and_scrolls, browser_follows_links_and_goes_back, budgets_keep_the_viewport_then_key_nodes, character_references, comments_and_doctype, css_cascade_specificity_order_important_and_inline, css_hides_inlines_and_styles_text, css_selectors_parse_and_match, data_tables_line_up_in_columns, dom_mutations_are_recorded, enter_in_a_field_types_then_submits, explicit_role_and_aria_label_win, flex_items_margins_nbsp_and_scripts_lay_out_like_a_browser, forms_render_and_join_the_model, headings_lists_quotes_and_pre, hidden_elements_are_not_drawn, images_and_inputs_show_their_text, implied_html_head_and_body, legacy_reference_in_attribute_stays_literal_before_equals, linked_style_sheets_apply, links_are_numbered_and_resolved, long_words_break_and_whitespace_collapses, media_queries_follow_the_window_width, p_closes_on_block_and_li_closes_li, page_model_json_marks_the_viewport, paragraphs_wrap_at_the_width_with_one_blank_row_between, plain_text_round_trips, post_forms_send_the_body, pre_drops_its_first_newline, refresh_targets_parse_like_browsers, script_and_style_are_raw_text, script_ends_only_at_its_own_end_tag, session_diffs_report_only_what_changed, shift_tab_wraps_to_the_last_link, svg_self_closing_tags_close, tables_close_rows_and_cells, tables_render_one_row_per_line, title_goes_in_head_and_text_in_body, typing_checking_selecting_and_submitting_with_get, unterminated_tag_and_comment_do_not_trap, url_resolution, urlencode_escapes_utf8, void_elements_and_stray_end_tags
ail.pkg:
    package garlic
    dep garlicjs path js

## (root)
```
KEYS := ...  ## The keys, as `--keys` prints them.
BG := 1973790
FG := 13948116
QUIET_FG := 8750469
LINK_FG := 5676246
FOCUS_BG := 2510712
HEADING_FG := 14474410
CODE_FG := 13537656
CONTROL_FG := 5163440
STATUS_BG := 31436
STATUS_FG := 16777215
enum Prompt  ## What the bottom row is doing: showing the status, or reading a URL, a
    Status
    Url(text str)
    Find(text str)
    Field(id int, text str)
struct Browser  ## The browser: the page shown (its tree and layout), where it is scrolled,
    url str
    doc html.Doc
    sheet css.Sheet  ## The page's linked style sheets (`<link rel=stylesheet>`), as one.
    pg page.Page
    scroll int
    focus int
    width int
    height int
    back List[str]
    message str
    prompt Prompt
    last_find str
    viewport_only bool  ## Whether the page model lists only the viewport's nodes (`--viewport`).
    budget int  ## The page model's size limit in tokens (`--budget N`), 0 for none.
    diff bool  ## Whether `--session` answers with diffs (`--diff`), and what it last sent.
    last_url str
    last_nodes Map[int, str]
    jar jsdom.Jar  ## Cookies from responses and scripts, sent back to their sites.
fn options() net.Options [io net]  ## The headers Garlic sends: who it is, and that it wants HTML.
fn fetch(url str, method str, body str) str [clock fs io net]  ## The text of `url`: `http` and `https` through `net.request` (which
fn fetch_with(url str, method str, body str, mut jar jsdom.Jar) str [clock fs io mutates net]  ## `fetch`, sending `jar`'s cookies for the site and keeping the ones the
WINDOWS_1252 := ...  ## The labels that name windows-1252, which HTML reads `iso-8859-1` and
fn body_text(r net.RawResponse) str  ## A response body as text: its `Content-Encoding` (gzip or deflate)
fn decode(data bytes, content_type str) str  ## `data` as text in the charset that `content_type` names, or failing that
fn charset_label(s str) str  ## The value of the `charset=` in `s`, lowercased and unquoted; "" if none.
fn error_page(url str, why str) str
MAX_REFRESHES := 5  ## At most this many hops (a `<meta http-equiv=refresh>` or a script
MAX_REFRESH_DELAY := 5  ## A refresh with a longer delay is left alone, as the page asks to be read.
fn refresh_target(d html.Doc, url str, scripting bool) str?  ## Where the page's `<meta http-equiv=refresh content="N; url=...">` sends
MAX_SHEETS := 8  ## At most this many linked style sheets are fetched per page.
fn linked_sheets(d html.Doc, url str) css.Sheet [fs io net]  ## The page's linked style sheets, fetched and parsed in document order.
fn fetch_text(url str) str [fs io net]  ## The text of a sub-resource, or "" when it can't be fetched.
struct Loaded  ## A loaded page: where it ended up and its tree.
    url str
    doc html.Doc
fn load_doc(target str, method str, body str, mut jar jsdom.Jar) Loaded [clock env fs io mutates net]  ## Fetches and parses `target`, then runs its scripts (unless `--no-js`),
fn script_url(u str) str  ## The address scripts see: a local file gets a file:// URL.
fn local_url(u str) str
fn edit_text(text str, key str) str?  ## `text` after a key typed into a prompt: Backspace deletes, a character
fn form_fields(d html.Doc, form int, submitter int) str  ## The form's fields, URL-encoded (`application/x-www-form-urlencoded`):
fn urlencode(s str) str  ## `s` encoded for a form: letters, digits and `*-._` as they are, space as
fn utf8(k int) List[int]  ## The UTF-8 bytes of code point `k`.
fn color(st page.Style) int  ## The text color of a run in style `st`.
fn window_main(mut b Browser) ! [clock env fail fs io mutates net ui]
    fails with ui.UiError
fn flag(args List[str], name str) str?  ## The value after flag `name` (`--width 60`), or none.
fn usage() [io]
fn main() ! [clock env fail fs io net ui]
fn tree(src str) str  ## The tree of `src` in the html5lib-tests format.
fn lay(src str, width int) page.Page
fn testdata() str [env fs io]  ## `tools/garlic/testdata`, from the repository root or `bootstrap/`.
fn node_with(b Browser, tag str, attr str, value str) int  ## The id of the first element `tag` whose `attr` is `value` in `b`'s page.
```

trimmed to fit 2000 tokens (~1831); dropped: private items
