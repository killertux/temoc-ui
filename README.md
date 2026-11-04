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
what you are looking for. When a page contains tests, you can also run the whole
file, a single table or a single row, and watch the results appear in place.

> [!NOTE]
> temoc-ui is in early development. See [Project status](#project-status) for what
> works today.

## Getting started

You need a recent stable [Rust](https://www.rust-lang.org/tools/install) toolchain,
and [temoc](https://github.com/killertux/temoc/releases) 0.10 or later on your `PATH`
to run tests. Install temoc-ui from this repository:

```sh
cargo install --git https://github.com/killertux/temoc-ui temoc-ui
```

Then open a folder, or a single markdown file:

```sh
temoc-ui              # the current folder
temoc-ui docs/        # another folder
temoc-ui docs/api.md  # one page
temoc-ui --config ci/temoc.toml  # with a temoc configuration file
temoc-ui --theme light           # with a theme
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
| `Left`, `Right`               | Scroll wide tables and code sideways  |
| `Tab`, `Shift-Tab`            | Focus the next or previous link       |
| `Enter`                       | Follow the focused link               |
| `h` or `Backspace`, `l`       | Go back or forward                    |
| `]`, `[`                      | Focus the next or previous test table |
| `}`, `{`                      | Focus the next or previous test row   |
| `r`                           | Run the focused table                 |
| `Ctrl-r`                      | Run the focused row                   |
| `R`                           | Run the whole file                    |
| `/`                           | Open search                           |
| `t`                           | Open the file tree                    |
| `Ctrl-g`                      | Open the graph of links               |
| `T`                           | Swap between dark and light           |
| `v`                           | Start selecting text                  |
| `?`                           | Show or hide help                     |
| `q`, `Ctrl-c`                 | Quit                                  |

Search looks through file paths and headings across the workspace, and the letters
you type don't need to be next to each other. Use `Down` and `Up` to choose a result
and `Enter` to open it, or `Esc` to close search. `Ctrl-t` shows only the files that
contain temoc tests.

The mouse works too. The wheel scrolls, and holding `Shift` or using a sideways wheel
scrolls wide tables and code. Click a link to follow it, a test table or one of its rows
to focus it, or a row of the file tree to open it.

temoc-ui captures the mouse, so it selects text itself. Drag to select, and the text is
copied when you let go. From the keyboard, `v` puts a cursor on the page: move it with
`h`, `j`, `k`, `l`, the arrow keys, `w`, `b`, `0`, `$`, `g` and `G`, press `v` to start
the selection, and `y` to copy it. `Esc` cancels. Copying uses the OSC 52 escape
sequence, so it works over SSH, but your terminal must allow it.

The file tree shows the workspace's markdown files, with the open page selected. Use
`j` and `k` or the arrow keys to move, `l` and `h` to open and close a folder, and
`Enter` to open a file or folder. `t` or `Esc` closes it.

The graph shows every markdown file of the workspace and the links between them.
Files are in columns by how many links away they are from the start page, which is the
workspace's `README.md`, or else its first file. Files that cannot be reached are in a
final column of their own. Use `j`, `k`, `h`, `l` or the arrow keys to move, `g` and `G`
to go to the top and bottom of a column, and `Enter` to open the selected file. The
panel below lists its links, including broken ones. `Ctrl-g` or `Esc` closes the graph.
The mouse works here too: click a file to select it, click it again to open it, and use
the wheel to scroll.

`?` lists the keys, from anywhere but search, and `?` or `Esc` hides the list. If it
does not fit on the screen, `j`, `k`, the arrow keys and the wheel scroll it.

There are four themes: `dark` and `light` use 24-bit colour, and `dark-256` and
`light-256` use the 256-colour palette. temoc-ui matches your terminal, using `COLORTERM`
to tell whether it shows 24-bit colour and asking it for its background colour. If it
does not answer, you get a dark theme. `--theme NAME` picks one yourself, and `T` swaps
between dark and light until you close temoc-ui.

temoc-ui never opens a web browser. External links stay on the page and show their
address in the status line.

### Running tests

Running needs a temoc configuration file, which tells temoc how to start your Slim
server. temoc-ui uses `temoc.toml` at the root of the workspace, or the file given
with `--config`. Pages open without one, but nothing can run.

The focused table has a reversed header row, and the focused row is reversed too. Each
row that runs gets a mark two columns after its end and a colour, as soon as temoc
finishes it: a green `✓` when it passed, a red `✗` for a wrong value, a red `!` for an
error such as an exception, a yellow `~` when it is snoozed, and a plain `-` when it never
ran. The row temoc is working on shows `⋯`. A row that did not pass has a note below it
saying why.

Each table that did not pass shows its result below it, and the result for the whole file
is at the right of the status bar. A table takes the worst result of its rows, and a file the worst of
its tables. A table with only some of its rows run is `partial`, and so is a file with
only some of its tables run, unless something did not pass.

Tests run in the background, so you can keep reading, and results stay until you close
temoc-ui or that part of the file runs again. A row run by itself cannot see symbols set
by other rows, which temoc reports as an error in the row.

## Project status

**Working today**

- Rendering GitHub Flavored Markdown, the format temoc reads
- Scrolling, following links and going back and forward through history
- Searching files and headings, with a filter for test files
- Finding test tables and moving between them
- Running a whole file, a single table or a single row, and showing each row's result
  in place as soon as it is known
- A pop-up file tree
- Mouse support
- Selecting and copying text
- A graph of how the pages link together
- A help screen listing the keys
- Dark and light themes, in 24-bit and 256 colours

**Further out**

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
