# Keys

Back to the [specification index](../README.md).

The keys are the [terminal's](../tui/keys.md), apart from these differences:

- `Ctrl-c` copies, as it does in any window, so `Ctrl-q` quits.
- `v` does nothing, because [there is no selection mode](selecting-text.md).
- `Ctrl` is `Cmd` on macOS.

| Key                 | Action                               | Described in                        |
| ------------------- | ------------------------------------ | ----------------------------------- |
| `j`, `Down`         | Scroll down one line                 | [Navigation](navigation.md)         |
| `k`, `Up`           | Scroll up one line                   | [Navigation](navigation.md)         |
| `PageDown`, `Space` | Scroll down one page                 | [Navigation](navigation.md)         |
| `PageUp`            | Scroll up one page                   | [Navigation](navigation.md)         |
| `g`, `G`            | Go to the top or bottom              | [Navigation](navigation.md)         |
| `Left`, `Right`     | Scroll a wide table or code sideways | [Navigation](navigation.md)         |
| `Tab`, `Shift-Tab`  | Focus the next or previous link      | [Navigation](navigation.md)         |
| `Enter`             | Follow the focused link              | [Navigation](navigation.md)         |
| `h`, `Backspace`    | Go back                              | [Navigation](navigation.md)         |
| `l`                 | Go forward                           | [Navigation](navigation.md)         |
| `/`                 | Open search                          | [Search](search.md)                 |
| `Ctrl-t`            | In search, show only test files      | [Search](search.md)                 |
| `]`, `[`            | Focus the next or previous test table | [Running tests](running.md)        |
| `}`, `{`            | Focus the next or previous test row  | [Running tests](running.md)         |
| `r`                 | Run the focused table                | [Running tests](running.md)         |
| `Ctrl-r`            | Run the focused row                  | [Running tests](running.md)         |
| `R`                 | Run the whole file                   | [Running tests](running.md)         |
| `t`                 | Show or hide the file tree           | [File tree](file-tree.md)           |
| `Ctrl-g`            | Show or hide the graph               | [Graph view](graph.md)              |
| `T`                 | Swap between dark and light          | [Dark and light](themes.md)         |
| `Ctrl-c`            | Copy the selected text               | [Selecting text](selecting-text.md) |
| `?`                 | Show or hide help                    | This page                           |
| `q`, `Ctrl-q`       | Quit                                 | This page                           |

The [graph](graph.md#selecting-a-file) reuses some of these keys while it shows.

## Help and quitting

`?` or the toolbar button shows help, which lists these keys, and `?` or `Esc` hides it.
The search box [keeps letters to itself](search.md#letters-stay-letters), so `?` and `q`
are typed there, and `Ctrl-q` still quits.

![Help](design/images/help.png)

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation      | workspace | actions               | help open? |
| ---------------- | --------- | --------------------- | ---------- |
| Hidden at first  | docs      |                       | no         |
| The key          | docs      | `?`                   | yes        |
| The button       | docs      | `Click "Show help"`   | yes        |
| Hides again      | docs      | `? ?`                 | no         |
| Esc hides it     | docs      | `? Esc`               | no         |
| Over the graph   | docs      | `Ctrl-g ?`            | yes        |

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation           | workspace | actions        | still open? |
| --------------------- | --------- | -------------- | ----------- |
| Running               | docs      |                | yes         |
| q quits               | docs      | `q`            | no          |
| Ctrl-q quits          | docs      | `Ctrl-q`       | no          |
| Ctrl-c does not       | docs      | `Ctrl-c`       | yes         |
| Ctrl-q in search      | docs      | `/ Ctrl-q`     | no          |
| Closing the window    | docs      | `Close window` | no          |
