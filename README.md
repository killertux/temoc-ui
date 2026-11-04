# temoc-ui

A terminal interface for [temoc](https://github.com/killertux/temoc), which runs
acceptance tests written in markdown.

temoc tests are ordinary markdown pages, so they sit next to the documentation they
describe. A hidden line marks a table as a test, and each row of the table is one
test case:

```markdown
## Addition

[//]: # "decisionTable Calculator"

| a | b | sum? |
| - | - | ---- |
| 1 | 2 | 3    |
```

temoc-ui lets you read those pages the way they were meant to be read. It opens as
a markdown viewer that makes it easy to jump between pages, follow links and find
what you are looking for. The goal is that when a page contains tests, you can also
run the whole file or a single table and see the results in place.

> [!NOTE]
> temoc-ui is in early development. It works as a markdown viewer today, but it
> **cannot run tests yet**. See [Project status](#project-status).

## Getting started

You need a recent stable [Rust](https://www.rust-lang.org/tools/install) toolchain.
Install temoc-ui from this repository:

```sh
cargo install --git https://github.com/killertux/temoc-ui temoc-ui
```

Then open a folder, or a single markdown file:

```sh
temoc-ui              # the current folder
temoc-ui docs/        # another folder
temoc-ui docs/api.md  # one page
```

temoc-ui opens the folder's `README.md`. If there isn't one, search opens so that
you can pick a page.

The folder it works in is called the workspace, and it decides which pages search
can find. Opening a file inside the current folder keeps the current folder as the
workspace. Opening a file outside it uses that file's folder instead.

To try it without any pages of your own, run it from a clone of this repository on
one of the sample workspaces:

```sh
cargo run -- samples/docs
```

## Using it

| Key                           | What it does                          |
| ----------------------------- | ------------------------------------- |
| `j`, `k`, `Down`, `Up`        | Scroll one line                       |
| `Space`, `PageDown`, `PageUp` | Scroll one page                       |
| `g`, `G`                      | Go to the top or bottom               |
| `Tab`, `Shift-Tab`            | Focus the next or previous link       |
| `Enter`                       | Follow the focused link               |
| `h` or `Backspace`, `l`       | Go back or forward                    |
| `]`, `[`                      | Focus the next or previous test table |
| `/`                           | Open search                           |
| `q`, `Ctrl-c`                 | Quit                                  |

Search looks through file paths and headings across the workspace, and the letters
you type don't need to be next to each other. Use `Down` and `Up` to choose a result
and `Enter` to open it, or `Esc` to close search. `Ctrl-t` shows only the files that
contain temoc tests.

temoc-ui never opens a web browser. External links stay on the page and show their
address in the status line.

## Project status

**Working today**

- Rendering GitHub Flavored Markdown, the format temoc reads
- Scrolling, following links and going back and forward through history
- Searching files and headings, with a filter for test files
- Finding test tables and moving between them

**Coming next**

- Running a whole file or a single table, and showing the results in place
- A pop-up file tree
- A graph of how the pages link together
- Mouse support
- Selecting and copying text
- A help screen

**Further out**

- Running a single row, which needs a change to temoc first
- A web interface, behind a compile-time feature flag

## The specification

The [specification](spec/README.md) describes how temoc-ui behaves, including the
features that are not built yet. It is written in temoc format, so its tables are
executable tests: each page explains a feature and proves that it works. It is a
good place to read about anything this page leaves out.

A table that is specified but not built yet is marked as snoozed. Building a feature
means removing the snooze from its tables, so this command lists what is left:

```sh
grep -rn 'snooze until 2099' spec
```

## Contributing

Work here starts from the specification: a new behavior is written as a failing spec
first, and then the code makes it pass.

Running the specification needs
[temoc](https://github.com/killertux/temoc/releases) on your `PATH`. These are the
checks that CI runs on every push and pull request:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
temoc -c temoc.toml
```

The full rules for changing the code are in [AGENTS.md](AGENTS.md).
