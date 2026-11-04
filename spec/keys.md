# Keys

Back to the [specification index](README.md).

| Key                 | Action                                   | Described in                                 |
| ------------------- | ---------------------------------------- | -------------------------------------------- |
| `j`, `Down`         | Scroll down one line                     | [Navigation](viewer/navigation.md)           |
| `k`, `Up`           | Scroll up one line                       | [Navigation](viewer/navigation.md)           |
| `PageDown`, `Space` | Scroll down one page                     | [Navigation](viewer/navigation.md)           |
| `PageUp`            | Scroll up one page                       | [Navigation](viewer/navigation.md)           |
| `g`, `G`            | Go to the top or bottom                  | [Navigation](viewer/navigation.md)           |
| `Left`, `Right`     | Scroll wide tables and code sideways     | [Navigation](viewer/navigation.md)           |
| `Tab`, `Shift-Tab`  | Focus the next or previous link          | [Navigation](viewer/navigation.md)           |
| `Enter`             | Follow the focused link                  | [Navigation](viewer/navigation.md)           |
| `h`, `Backspace`    | Go back                                  | [Navigation](viewer/navigation.md)           |
| `l`                 | Go forward                               | [Navigation](viewer/navigation.md)           |
| `/`                 | Open search                              | [Search](search.md)                          |
| `Ctrl-t`            | In search, show only test files          | [Search](search.md)                          |
| `]`, `[`            | Focus the next or previous test table    | [Navigation](viewer/navigation.md)           |
| `r`                 | Run the focused table                    | [Running a table](running/running-a-table.md) |
| `R`                 | Run the whole file                       | [Running a file](running/running-a-file.md)  |
| `t`                 | Open the file tree                       | [File tree](viewer/file-tree.md)             |
| `Ctrl-g`            | Enter the graph view                     | [Graph view](viewer/graph.md)                |
| `v`                 | Start selecting text                     | [Selecting text](viewer/selecting-text.md)   |
| `?`                 | Show or hide help                        | This page                                    |
| `q`, `Ctrl-c`       | Quit                                     | This page                                    |

The file tree, graph view, and text selection reuse some of these keys while they
are open. Their pages list what each key does there.

## Help and quitting

`?` shows a help screen listing these keys, and pressing it again hides it. `q`
quits, except in the search box, where it is typed like any other letter.
`Ctrl-c` quits from anywhere.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation             | workspace | keys  | look for           | visible? |
| ----------------------- | --------- | ----- | ------------------ | -------- |
| Help is hidden at first | docs      |       | Run the whole file | no       |
| Help opens              | docs      | `?`   | Run the whole file | yes      |
| Help closes             | docs      | `? ?` | Run the whole file | no       |

[//]: # "decisionTable Tui"

| # situation            | workspace | keys       | still open? |
| ---------------------- | --------- | ---------- | ----------- |
| Running                | docs      |            | yes         |
| q quits                | docs      | `q`        | no          |
| Ctrl-c quits           | docs      | `Ctrl-c`   | no          |
| q is typed in search   | docs      | `/ q`      | yes         |
| Ctrl-c quits in search | docs      | `/ Ctrl-c` | no          |
