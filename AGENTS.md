# Agent Guidelines

Rules for any agent working in this repository. "Human approval" means explicit
approval from a human in the current conversation or task. If you need approval,
stop and ask before making the change.

## Before finishing a change

All of these must pass:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
temoc -c temoc.toml
```

CI runs the same checks on every push to `main` and every pull request.

Report any failure honestly. Do not describe work as done while a check fails.

## Linting and formatting

- Always obey `cargo clippy` and `cargo fmt`. Fix the code, not the tool.
- Do not override the linter or formatter without human approval. This includes:
  - `#[allow(...)]`, `#[expect(...)]`, and their crate-level `#![...]` forms
  - `#[rustfmt::skip]`
  - Adding or changing `clippy.toml`, `rustfmt.toml`, or the `[lints]` table in
    `Cargo.toml`
  - Weakening lints through `RUSTFLAGS`, `--cap-lints`, or changed CLI flags
- When asking for approval, name the lint, explain why the code should not
  change, and propose the narrowest possible scope.
- Approved lint overrides use `#[expect(..., reason = "...")]`, not `#[allow]`.
  Any other approved override gets a comment stating the reason.

## Error handling

- Non-test code must not panic. Return `Result` or `Option` and propagate
  failures with `?`.
- Clippy denies `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`,
  `unreachable!`, and direct indexing or slicing. Use `get()`, pattern matching,
  or a proper error instead.
- Lints do not catch every panic, such as `assert!` or division by zero. Avoid
  those in non-test code too.
- Do not silently discard errors with `let _ =` or `.ok()`. Handle or propagate
  them.

## Dependencies

- Do not add a dependency without human approval. This applies to
  `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]`.
- When asking for approval, name the crate, say why it is needed, and say why
  the standard library or an existing dependency is not enough.

## Specification

- The `spec/` folder is the source of truth for how the software behaves. Read
  the relevant parts before implementing or changing behavior, and obey them.
- Work spec first: a new behavior starts as a failing spec, then the code makes
  it pass.
- Never edit `spec/` or `samples/` unless a human asks for it or approves the
  edit. `samples/` holds the workspaces the spec runs against.
- A table marked `-- snooze until 2099-12-31` is specified but not built yet.
  Removing those snoozes is the goal: when a table's behavior is built, remove
  its snooze in the same change. That edit to `spec/` is already approved.
  `grep -rn 'snooze until 2099' spec` lists what is left.
- Do not add a snooze to `spec/` without human approval.
- If the spec is ambiguous, incomplete, or conflicts with the code or the task,
  stop and ask a human. Do not guess, and do not change the spec to fit the code.

## Testing

- Add tests that validate the software. New behavior ships with tests, and bug
  fixes ship with a regression test that fails without the fix.
- Test the behavior the spec requires, not just the implementation details.
- Do not delete, weaken, or `#[ignore]` a test to get a passing run without
  human approval.
