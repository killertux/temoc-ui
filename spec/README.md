# temoc-ui Specification

temoc-ui is an interface for [temoc](https://github.com/killertux/temoc), which runs
acceptance tests written in markdown. It opens as a markdown viewer that makes it
easy to jump between pages. When a page contains temoc tests, you can run the whole
file, a single table or a single row and see the results in place, row by row, as
they arrive.

This specification is itself written in temoc format. Every table on these pages
is an executable test, so a page describes how temoc-ui behaves and proves it at
the same time.

New here? Read [how these pages work](writing-specs.md) and the
[glossary](glossary.md).

## Core

What temoc-ui does whatever its interface is.

### Viewing markdown

- [Opening a workspace](viewer/opening.md): the workspace and what opens first.
- [Links and history](viewer/links.md): following links, and going back and forward.
- [Search](search.md): finding a file or heading, and showing only test files.
- [File tree](viewer/file-tree.md): the files of the workspace, as a tree.
- [Graph](viewer/graph.md): how every page links together.

### Running tests

- [Finding tests](running/finding-tests.md): how a page is recognized as a test file.
- [Configuration](running/configuration.md): how temoc-ui finds the server command.
- [Running a file](running/running-a-file.md): running every table in a page.
- [Running a table](running/running-a-table.md): running one table in a page.
- [Running a row](running/running-a-row.md): running one row of a table.
- [Row results](running/row-results.md): how each row gets its result as it runs.
- [Reading temoc's output](running/temoc-output.md): how results are worked out.

## Terminal interface

What only the terminal does. Start with
[how these pages work](tui/writing-specs.md).

- [Opening temoc-ui](tui/opening.md): the command line.
- [Rendering](tui/rendering.md): how markdown looks in the terminal.
- [Navigation](tui/navigation.md): scrolling, links, tables, and history.
- [Search](tui/search.md): the search box.
- [File tree](tui/file-tree.md): browsing the workspace in a pop-up.
- [Graph view](tui/graph.md): the screen that draws the graph.
- [Running tests](tui/running.md): choosing what to run, and results on the page.
- [Row results](tui/row-results.md): marks, colours and notes.
- [Mouse](tui/mouse.md): scrolling, links, tables, the file tree, and the graph.
- [Selecting text](tui/selecting-text.md): copying with the mouse or keyboard.
- [Themes](tui/themes.md): dark and light, and the colours of each.
- [Keys](tui/keys.md): every key in one place.

## Graphical interface

What only the GUI does. None of it is built yet, so every table here is snoozed. Start
with [how these pages work](gui/writing-specs.md). The pages describe behavior, and
the [design](gui/design/README.md) shows how it should look.

- [Opening the GUI](gui/opening.md): the `--gui` flag.
- [The window](gui/window.md): the panels and what each one is for.
- [Rendering](gui/rendering.md): what the GUI adds to how markdown is shown.
- [Navigation](gui/navigation.md): scrolling, wide blocks, links, and history.
- [Search](gui/search.md): the search box.
- [File tree](gui/file-tree.md): browsing the workspace in a side panel.
- [Graph view](gui/graph.md): a graph that lays itself out.
- [Running tests](gui/running.md): buttons, keys, and the tests panel.
- [Row results](gui/row-results.md): marks and notes.
- [Selecting text](gui/selecting-text.md): selecting with the mouse and copying.
- [Dark and light](gui/themes.md): the two modes.
- [Keys](gui/keys.md): every key in one place.

## Scope

These are planned for later, and none of them are specified yet:

- **Symbols across rows and tables.** A table or row run alone cannot see `$V`
  symbols set by another one.
- **Cancelling a run**, and **reloading a page** that changed on disk.
- **Respecting `.gitignore`** when listing files.
