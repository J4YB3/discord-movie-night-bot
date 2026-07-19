fmt-check:
    cargo fmt --all -- --check

# Explicit legacy baseline until PR 2; manifest deny policy remains active.
lint:
    cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -A clippy::absurd_extreme_comparisons -A clippy::cmp_owned -A clippy::collapsible_if -A clippy::comparison_to_empty -A clippy::empty_line_after_doc_comments -A clippy::enum_variant_names -A clippy::expect_used -A clippy::explicit_counter_loop -A clippy::for_kv_map -A clippy::large_enum_variant -A clippy::len_zero -A clippy::manual_find -A clippy::manual_map -A clippy::match_like_matches_macro -A clippy::needless_bool -A clippy::needless_borrow -A clippy::needless_range_loop -A clippy::needless_return -A clippy::ptr_arg -A clippy::question_mark -A clippy::redundant_field_names -A clippy::redundant_pattern_matching -A clippy::single_component_path_imports -A clippy::unnecessary_unwrap -A clippy::unwrap_used -A clippy::useless_conversion -A clippy::useless_format

test:
    cargo test --workspace --all-features --locked

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked

deny:
    cargo deny check

check: fmt-check lint test doc deny
