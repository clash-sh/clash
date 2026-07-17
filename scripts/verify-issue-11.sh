#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "== issue #11: reported Windows path-shape mismatch =="
cargo test worktree::manager::tests::windows_reported_path_shapes_do_not_match_without_normalization -- --exact
cargo test worktree::manager::tests::windows_extended_prefix_and_slashes_match -- --exact
cargo test worktree::manager::tests::normalized_relative_path_handles_windows_prefix_mismatch -- --exact
cargo test worktree::manager::tests::normalized_relative_path_handles_windows_prefix_mismatch_reversed -- --exact

echo "== full Rust test suite =="
cargo test

echo "== clippy =="
cargo clippy --all-targets --all-features -- -D warnings

echo "== linked-worktree CLI smoke test =="
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

main="$tmp/repo"
linked="$tmp/linked-a"

git init -q "$main"
git -C "$main" config user.email test@example.com
git -C "$main" config user.name Test

mkdir -p "$main/src"
printf 'one\n' > "$main/src/lib.rs"
git -C "$main" add src/lib.rs
git -C "$main" commit -q -m initial
git -C "$main" worktree add -q "$linked" -b linked-a

cargo build --quiet
output="$(target/debug/clash check "$linked/src/lib.rs")"
printf '%s\n' "$output"

printf '%s\n' "$output" | grep -q '"file": "src/lib.rs"'
printf '%s\n' "$output" | grep -q '"current_worktree": "linked-a"'
printf '%s\n' "$output" | grep -q '"current_branch": "linked-a"'

echo "issue #11 verification passed"
