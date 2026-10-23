#!/usr/bin/env bash
# The Mac gate in one command, printing only what failed (ADR 0842).
#   tools/gate/mac-gate.sh [BASE_REF]    from the repository root, on the branch under test
# Runs the full selftest with --no-fail-fast, then the native suites, and prints one
# line per failing test plus the first 3 lines of its message. Silence means green:
# the last line is `GATE GREEN` or `GATE RED (N failures)`. Full logs stay in
# $TMPDIR/ail-gate/*.log for a look when a failure needs more.
set -u
cd "$(git rev-parse --show-toplevel)"
out="${TMPDIR:-/tmp}/ail-gate"; mkdir -p "$out"; fails=0
run() { # run NAME CMD...
  name=$1; shift
  if "$@" >"$out/$name.log" 2>&1; then return 0; fi
  fails=$((fails+1))
  echo "RED $name"
  grep -E "^test .* FAILED|panicked at|differs from|^error|disagreement|Symbol not found" "$out/$name.log" | head -8 | cut -c1-200
}
# bootstrap/tests/tools.rs and the ported tests of cases.rs, explain_coverage.rs, fix.rs, effects.rs, error_paths.rs, cli_errors.rs, mistakes.rs, contracts.rs, snapshot.rs, derive_json.rs, ct_limit.rs, deterministic.rs, test_stderr.rs,
# schedule.rs, doc_scripts.rs, fmt.rs, budget.rs, limits.rs, diff.rs, build_cache.rs, parallel.rs, api_queries.rs, replay.rs, api_card.rs, edit.rs, git_deps.rs and net.rs run as gate.ail's suites (misc, docs, fmt and those named like the files) below (B6),
# so the cargo copies are skipped here (exact names).
TOOLS_RS=(editor_checks_clean_and_passes_its_tests garlic_checks_clean_and_renders_headless editor_replaces_in_files
  editor_drives_the_real_semantic_api editor_key_table_matches_the_readme editor_agent_panel_reads_the_real_outputs
  pdf_reader_checks_clean_and_reads_its_fixtures garlic_js_parses_and_runs_its_suites garlic_reads_a_page_from_serve_dir
  examples cases packages usage_errors_exit_2 json_diagnostics_have_spans_and_fixes explain_prints_rules
  property_options_replay test_trace_failing_only duplicates_carry_notes trace_options_rejected api_outlines_every_accepted_program
  every_emitted_rule_has_an_explain_entry explain_entries_cite_only_rules_that_exist rules_with_a_fix_match_the_corpora
  explain_entries_stay_short fixtures_are_fixed_and_check_clean dry_run_prints_a_diff_and_leaves_the_file unfixable_errors_exit_1
  effects_of_commands_example deny_all_runs_a_program_without_capabilities deny_rejects_an_unknown_capability
  parallel_verdicts_of_parallel_example check_errors_lists_each_failure_path api_gives_the_inferred_error_set
  explain_shows_a_rejected_program_and_its_repair deny_names_an_effect cwd_removed read_line_fails read_all_fails
  build_extern_c_fn
  each_mistake_is_named_once_with_help fixes_produce_the_recorded_text the_corpus_meets_its_targets failed_requires_names_the_function_condition_and_values failed_ensures_names_the_result_and_where_it_returned no_contracts_skips_the_checks api_lists_contracts_with_the_signature explain_knows_the_contract_rules
  failed_ensures_with_old_shows_the_value_on_entry inherited_requires_names_the_trait_and_uses_the_implementations_parameters inherited_ensures_traps_in_the_implementation invariants_name_the_struct_the_method_and_the_value api_shows_inherited_contracts_and_invariants update_rewrites_and_passes dry_run_and_json conflicts_and_non_literals_fail
  expanded_api_shows_generated_json_code a_constant_over_the_limit_traps_and_names_itself a_constant_under_the_limit_is_computed seed_makes_a_random_program_repeat deterministic_rejects_random_clock_and_proc deterministic_runs_a_program_without_them_identically_twice seed_needs_an_integer
  failing_test_shows_its_stderr_and_passing_test_hides_it ail_run_still_writes_standard_error_directly a_seed_replays_its_interleaving a_failure_prints_the_seed a_bad_schedule_is_a_usage_error cancel_trees_and_orderings_pass_every_schedule
  doc_links_resolve doc_index_and_glossary_have_no_duplicate_rows cards_have_no_exact_repeats formatting_examples_keeps_their_output check_reports_a_diff_and_fmt_rewrites a_file_with_errors_is_left_alone
  fitting_output_is_unchanged errors_first_then_a_note json_note tests_keep_their_summary run_keeps_program_arguments bare_budget_is_the_default
  timeout_traps_an_endless_loop_with_a_snapshot max_steps_traps_the_same_way_every_time max_memory_traps_a_growing_list a_test_that_times_out_fails_and_the_next_still_runs limits_need_a_positive_number_and_run_or_test coverage_counts_lines_and_functions_of_a_two_function_file coverage_json_lists_missed_ranges a_failing_test_shows_its_contract_and_stack
  text_report no_change json_lines against_a_git_revision cached_and_uncached_builds_are_identical
  par_for_runs_on_the_pool_with_sequential_output parallel_flag_pools_proven_loops deterministic_runs_on_one_thread pure_tasks_run_outside_the_scheduler par_comprehension_runs_on_the_pool_with_sequential_output
  refs_cross_user_modules type_of_a_call_into_a_module_names_the_callee exit_codes every_example_answers_locals outline_of_a_package_directory refs_across_a_package
  a_replay_repeats_the_run a_changed_program_diverges handles_stop_and_traps_repeat tests_record_their_failures bad_options_are_explained
  card_lists_the_program private_items_are_dropped_first a_small_budget_drops_in_order middle_level_keeps_first_sentences_of_public_items json_form
  rejected_programs_and_bad_options committed_module_cards_are_fresh committed_cards_are_fresh module_card_is_one_module index_lists_modules
  every_op all_or_nothing diagnostics_and_dry_runs package_rename compact_hunks compact_rename_and_errors single_file_checks_run_from_memory concurrent_readers_never_see_a_preview
  fetch_lock_and_update fetch_errors_name_the_dependency get_and_post_against_a_local_server net_is_a_capability https_against_a_local_tls_server tls_conn_against_a_local_tls_server serve_is_a_capability)
skips=(); for t in "${TOOLS_RS[@]}"; do skips+=(--skip "$t"); done
run selftest ./ail selftest --no-fail-fast -- --exact "${skips[@]}"
run native_suites bash -c 'cd bootstrap && cargo test --release --no-fail-fast --test native --test native_subset_table --test native_asm --test native_globals --test native_test'
# The gate in AIL (RR-4): the same program suites without cargo. AIL_GATE=ail skips the cargo run above.
run gate_ail ./ail run tools/gate/gate.ail examples native tools cases explain fix cli mistakes contracts snapshot misc docs fmt budget limits diff build_cache parallel api_queries replay api_card edit git_deps net
# Every native binary must start (dyld/kernel32 mix-ups once aborted all of them).
if ./ail build bootstrap/tests/native/run/arith.ail -o "$out/hello" >/dev/null 2>&1; then
  "$out/hello" >/dev/null 2>&1 || { fails=$((fails+1)); echo "RED native binaries do not start"; }
else fails=$((fails+1)); echo "RED native build of arith.ail"; fi
[ "$fails" = 0 ] && echo "GATE GREEN" || echo "GATE RED ($fails failures); logs in $out"
exit $((fails > 0))
