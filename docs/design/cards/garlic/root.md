# garlic: (root)
entry: main in garlic.ail
tests: a_meta_refresh_moves_to_its_target_at_once, a_slow_meta_refresh_stays_put, a_wrapped_link_spans_rows, absolute_children_sit_at_the_positioned_ancestors_offsets, agent_actions_follow_click_back_and_report, any_input_parses, attributes_quoted_unquoted_bare_and_duplicate, back_and_forward_show_kept_pages_without_a_fetch, bodies_decode_by_charset_and_content_encoding, bottom_anchors_an_absolute_child_to_the_ancestors_lower_edge, browser_finds_text_and_scrolls, browser_follows_links_and_goes_back, budgets_keep_the_viewport_then_key_nodes, character_references, clear_moves_below_the_floats, comments_and_doctype, css_cascade_specificity_order_important_and_inline, css_hides_inlines_and_styles_text, css_lengths_and_colors_parse, css_selectors_parse_and_match, data_tables_line_up_in_columns, decoded_pictures_go_to_the_host_once, dom_mutations_are_recorded, enter_in_a_field_types_then_submits, explicit_role_and_aria_label_win, find_in_page_scrolls_to_the_match_and_marks_it, flex_items_margins_nbsp_and_scripts_lay_out_like_a_browser, flex_rows_lay_items_side_by_side, floats_take_a_corner_and_text_wraps_beside_them, forms_render_and_join_the_model, headings_lists_quotes_and_pre, hidden_elements_are_not_drawn, images_and_inputs_show_their_text, implied_html_head_and_body, legacy_reference_in_attribute_stays_literal_before_equals, linked_style_sheets_apply, links_are_numbered_and_resolved, long_words_break_and_whitespace_collapses, media_queries_follow_the_window_width, p_closes_on_block_and_li_closes_li, page_model_json_marks_the_viewport, paragraphs_wrap_at_the_width_with_one_blank_row_between, pictures_without_sizes_take_their_own_from_the_fetched_header, pixel_clicks_use_form_controls, pixel_forms_take_typing_and_submit_with_enter, pixel_text_widths_follow_the_font_table, pixel_textarea_shows_its_text, plain_text_round_trips, post_forms_send_the_body, pre_drops_its_first_newline, refresh_targets_parse_like_browsers, rounded_corners_cut_the_rectangle_into_strips, script_and_style_are_raw_text, script_ends_only_at_its_own_end_tag, session_diffs_report_only_what_changed, shift_tab_wraps_to_the_last_link, svg_self_closing_tags_close, tables_close_rows_and_cells, tables_render_one_row_per_line, tabs_keep_their_own_pages_and_switch_back, the_address_field_opens_urls_and_cmd_l_focuses_it, the_app_shows_the_page_and_follows_links_by_click, the_display_list_draws_each_run_at_its_cell, the_focused_target_gets_a_highlight_and_clicks_map_to_cells, the_native_build_draws_the_same_pixels_as_the_interpreter, the_native_window_scrolls_clicks_and_resizes_by_points, the_pixel_layout_paints_backgrounds_and_finds_links, the_pixel_layout_stacks_blocks_with_margins_and_wraps_words, title_goes_in_head_and_text_in_body, typing_checking_selecting_and_submitting_with_get, unterminated_tag_and_comment_do_not_trap, url_resolution, urlencode_escapes_utf8, void_elements_and_stray_end_tags, voltare_page_gets_variables_logo_icon_link_and_gradient_text, wrapping_chips_keep_their_padding_apart
ail.pkg:
    package garlic
    dep garlicjs path js
    dep image path ../pdf/image caps fs

## (root)
```
KEYS := ...
enum Prompt
    Status
    Url(text str)
    Find(text str)
    Field(id int, text str)
struct Browser
    url str
    doc html.Doc
    sheet css.Sheet
    pg page.Page
    scroll int
    focus int
    width int
    height int
    back List[str]
    message str
    prompt Prompt
    last_find str
    viewport_only bool
    budget int
    diff bool
    last_url str
    last_nodes Map[int, str]
    cj std.net.jar.Jar
    stamp int
    sizes Map[str, List[int]]
    kept Map[str, Kept]
    kept_order List[str]
    ahead List[str]
    bits Map[str, images.Bitmap]
struct Kept
    doc html.Doc
    sheet css.Sheet
    scroll int
MAX_KEPT := 16
fn options() net.Options [io net]
fn image_options() net.Options [io net]
fn fetch(url str, method str, body str) str [clock fs io net]
fn fetch_with(url str, method str, body str, mut cj std.net.jar.Jar) str [clock fs io mutates net]
WINDOWS_1252 := ...
fn body_text(r net.RawResponse) str
fn decode(data bytes, content_type str) str
fn charset_label(s str) str
fn error_page(url str, why str) str
MAX_REFRESHES := 5
MAX_REFRESH_DELAY := 5
fn refresh_target(d html.Doc, url str, scripting bool) str?
MAX_SHEETS := 8
fn linked_sheets(d html.Doc, url str) css.Sheet [fs io net]
fn fetch_text(url str) str [fs io net]
struct Loaded
    url str
    doc html.Doc
fn load_doc(target str, method str, body str, mut cj std.net.jar.Jar) Loaded [clock env fs io mutates net]
fn script_url(u str) str
fn local_url(u str) str
fn edit_text(text str, key str) str?
fn form_fields(d html.Doc, form int, submitter int) str
fn urlencode(s str) str
fn utf8(k int) List[int]
struct Gui
    b Browser
    cell_w float
    cell_h float
    cols int
    rows int
    pending float
    area_w float
    area_h float
    pixel bool
    scene gfx.Scene
    scene_stamp int
    scene_w float
    scene_url str
    scene_edit int
    found str
    top float
    sent Set[str]
    fresh Set[str]
    meas Map[str, float]
fn address(text str) str [fs io]
MEASURE_RUNS := 5000
struct Garlic
    g Gui
    tabs List[Gui]
    cur int
    url_text str
    want str
impl std.app.App for Garlic
fn to_app(op draw.Op, g Gui) std.app.Op
GUI_W := 960.0
GUI_H := 600.0
GUI_CELL_W := 7.8
GUI_CELL_H := 16.0
fn app_main(b Browser, pixel bool) ! [app clock env fail fs io net proc]
fn flag(args List[str], name str) str?
fn usage() [io]
VALUE_FLAGS := ...
fn target_of(args List[str]) str
fn op_line(op draw.Op) str
fn pixels_dump(mut b Browser, width float) [fs io mutates net]
fn main() ! [app clock env fail fs io net proc]
fn tree(src str) str
fn lay(src str, width int) page.Page
fn testdata() str [env fs io]
fn show_ops(ops List[draw.Op]) List[str]
fn resized(w float, h float) std.app.Event
fn clicked(x float, y float) std.app.Event
fn node_with(b Browser, tag str, attr str, value str) int
fn pixel_text(ops List[draw.Op]) List[str]
fn pixels(src str, width float) gfx.Scene
fn bench_run(phase int, src str, d html.Doc, sheet css.Sheet) int
fn bench(args List[str]) [clock fs io]
fn bitmap_data_lens(ops List[std.app.Op]) List[List[int]]
fn flat(s gfx.Scene) List[str]
```

trimmed to fit 2000 tokens (~1777); dropped: private items, doc comments
