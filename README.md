# discord-movie-night-bot

A Discord bot for managing movie-night watch lists and votes.

## Run locally

Install the Rust toolchain specified by `rust-toolchain.toml`. The temporary
`external_data` compatibility shim reads `DISCORD_TOKEN` and `TMDB_API_KEY`
from compile-time environment variables. Supply them when compiling or running:

```sh
DISCORD_TOKEN="your-discord-token" TMDB_API_KEY="your-tmdb-api-key" cargo run --locked
```

The shim stores no secrets in the repository. Typed runtime configuration will
replace this temporary mechanism in PR 3.

## Local verification

Run these commands before opening a pull request:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings -A clippy::absurd_extreme_comparisons -A clippy::cmp_owned -A clippy::collapsible_if -A clippy::comparison_to_empty -A clippy::empty_line_after_doc_comments -A clippy::enum_variant_names -A clippy::expect_used -A clippy::explicit_counter_loop -A clippy::for_kv_map -A clippy::large_enum_variant -A clippy::len_zero -A clippy::manual_find -A clippy::manual_map -A clippy::match_like_matches_macro -A clippy::needless_bool -A clippy::needless_borrow -A clippy::needless_range_loop -A clippy::needless_return -A clippy::ptr_arg -A clippy::question_mark -A clippy::redundant_field_names -A clippy::redundant_pattern_matching -A clippy::single_component_path_imports -A clippy::unnecessary_unwrap -A clippy::unwrap_used -A clippy::useless_conversion -A clippy::useless_format
cargo test --workspace --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo deny check
```

The explicit Clippy allows form a temporary baseline for existing legacy code
until PR 2. They preserve the manifest's `correctness`, `suspicious`, and `perf`
deny policy rather than capping lint levels globally; remove the allows as PR 2
remediates each legacy lint.


[`just`](https://github.com/casey/just) is optional. If installed, `just check`
runs the same checks in order. CI runs the Cargo commands directly and does not
require `just`.

## Inviting the bot to your server

Use the Discord permissions calculator at
<https://discordapi.com/permissions.html#257088>. Paste the Client ID from the
Discord Developer Portal's `OAuth2` section, then follow the generated link to
add the bot to a server.
