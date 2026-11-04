# temoc-ui Specification

temoc-ui is a terminal interface for [temoc](https://github.com/killertux/temoc),
which runs acceptance tests written in markdown. It opens as a markdown viewer
that makes it easy to jump between pages. When a page contains temoc tests, you
can run the whole file or a single table and see the results in place.

This specification is itself written in temoc format. Every table on these pages
is an executable test, so a page describes how temoc-ui behaves and proves it at
the same time.

New here? Read [how these pages work](writing-specs.md) and the
[glossary](glossary.md).

## Viewing markdown

- [Opening temoc-ui](viewer/opening.md): what appears when the app starts.
- [Rendering](viewer/rendering.md): how markdown looks in the terminal.
- [Navigation](viewer/navigation.md): scrolling, links, tables, and history.
- [Search](search.md): jumping to a file or heading, and showing only test files.
- [File tree](viewer/file-tree.md): browsing the workspace in a pop-up.
- [Graph view](viewer/graph.md): seeing how every page links together.
- [Mouse](viewer/mouse.md): scrolling, links, tables, the file tree, and the graph.
- [Selecting text](viewer/selecting-text.md): copying with the mouse or keyboard.
- [Keys](keys.md): every key in one place.

## Running tests

- [Finding tests](running/finding-tests.md): how a page is recognized as a test file.
- [Configuration](running/configuration.md): how temoc-ui finds the server command.
- [Running a file](running/running-a-file.md): running every table in a page.
- [Running a table](running/running-a-table.md): running one table in a page.
- [Reading temoc's output](running/temoc-output.md): how results are worked out.

## Scope

The first version is the terminal interface. These are planned for later, and
none of them are specified yet:

- **Running a single row.** temoc can only target a whole table today, so this
  needs a change to temoc first.
- **Symbols across rows and tables.** A table run alone cannot see `$V` symbols
  set by another table. This becomes more important once single rows can run.
- **Web interface.** It will sit behind a compile-time feature flag and reuse the
  same behavior described here.
- **Cancelling a run**, and **reloading a page** that changed on disk.
- **Respecting `.gitignore`** when listing files.
