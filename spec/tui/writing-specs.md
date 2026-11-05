# How the Terminal Pages Work

Back to the [specification index](../README.md).

These pages describe what only the terminal does, and prove that its keys and its
screen reach the [core behavior](../writing-specs.md#core-and-interfaces). A rule that
any interface would follow belongs in the core, so a table here only needs enough rows
to show that the terminal is connected to it.

Most pages here use the `Tui` fixture. For each row it starts a fresh temoc-ui in an
80 by 24 in-memory terminal, presses the keys, waits for any test run to finish,
and then reads the screen. The `MarkdownRendering` fixture is described in
[Rendering](rendering.md).

### Inputs

| Column      | Meaning                                                             |
| ----------- | ------------------------------------------------------------------- |
| `workspace` | The folder under [`samples/`](../../samples) to start in.              |
| `arguments` | Command line arguments for temoc-ui. Empty means none.              |
| `keys`      | The keys to press, in order, separated by spaces.                   |
| `look for`  | Text to check for. Used with `visible?` or `in graph?`.             |
| `color term` | The value of `COLORTERM`. Empty means that it is not set.          |
| `terminal background` | What the terminal answers when asked for its background, or `no answer`. Empty means `no answer`. |

### Keys notation

| Written as                                 | Meaning                       |
| ------------------------------------------ | ----------------------------- |
| `j`, `/`, `G`                              | Press that key.               |
| `Enter`, `Esc`, `Tab`, `Backspace`         | Press that named key.         |
| `Shift-Tab`, `PageUp`, `PageDown`          | Press that named key.         |
| `Up`, `Down`, `Left`, `Right`, `Space`     | Press that named key.         |
| `Ctrl-t`                                   | Hold Ctrl and press the key.  |
| `"some text"`                              | Type each character in turn.  |

Keys wait for any test run to finish before the next one is pressed, which leaves
nothing to see of a run in progress. `Wait` is for looking at one.

| Written as    | Meaning                                                                              |
| ------------- | ------------------------------------------------------------------------------------ |
| `Wait "text"` | Stop waiting for the run, and wait until the text is on screen. Later keys do not wait for the run either. |

Mouse actions are written in the same column. They find their position from text
on screen, searching from the top, so that they do not depend on the layout.

| Written as                   | Meaning                                                       |
| ---------------------------- | ------------------------------------------------------------- |
| `Click "text"`               | Click the first character of the text.                        |
| `Drag "from" "to"`           | Press on the first character of `from`, release on the last character of `to`. |
| `DragBack "from" "to"`       | Press on the last character of `from`, release on the first character of `to`. |
| `WheelDown`, `WheelUp`       | Turn the mouse wheel one step.                                |
| `WheelLeft`, `WheelRight`    | Turn a sideways mouse wheel one step.                         |
| `Shift-WheelDown`, `Shift-WheelUp` | Turn the mouse wheel one step while holding Shift.      |
| `ClickOutside`               | Click a spot outside the open pop-up.                         |

### Expected values

| Column            | Meaning                                                       |
| ----------------- | ------------------------------------------------------------- |
| `document?`       | Path of the open document within the workspace, or `none`.    |
| `visible?`        | `yes` if the `look for` text is on screen, otherwise `no`.    |
| `highlight?`      | How the `look for` text is highlighted: `green`, `red`, `yellow`, `reversed`, or `none`. `green`, `red` and `yellow` are the theme's colours for passed, failed and snoozed. |
| `next line?`      | The text on the screen line below the `look for` text, or `none` if it is blank. |
| `status?`         | The status bar message, or `none`.                            |
| `still open?`     | `yes` while temoc-ui is running, `no` after it exits.         |
| `focused link?`   | The text of the focused link, or `none`.                      |
| `search open?`    | `yes` while the search box is showing.                        |
| `results?`        | Search results in order, separated by commas, or `none`.      |
| `first result?`   | The top search result, or `none`.                             |
| `focused table?`  | The name of the focused test table, or `none`.                |
| `focused row?`    | The line of the focused row, or `none`.                       |
| `table results?`  | Each table's name and result, separated by commas.            |
| `row results?`    | Each row that has a result, written `line: result` and separated by commas, or `none`. |
| `file result?`    | The result for the whole document.                            |
| `failures?`       | Each failure with its table and line, separated by semicolons, or `none`. |
| `temoc arguments?`| The arguments of the last temoc command, or `none`.           |
| `tree open?`      | `yes` while the file tree is showing.                         |
| `tree?`           | The rows of the file tree in order, or `none`.                |
| `tree selection?` | The selected row of the file tree, or `none`.                 |
| `graph open?`     | `yes` while the graph view is showing.                        |
| `columns?`        | The graph's columns, separated by semicolons, or `none`.      |
| `links?`          | Every link in the graph, written `from -> to`.                |
| `in graph?`       | `yes` if the `look for` text is in the graph, ignoring the panel. |
| `distance?`       | Links from the start page to the selected file, or `not reachable`. |
| `selected file?`  | The file selected in the graph, or `none`.                    |
| `links out?`      | The links out of the selected file, or `none`.                |
| `links in?`       | The links into the selected file, or `none`.                  |
| `selection?`      | The selected text, or `none`.                                 |
| `clipboard?`      | The text last copied to the clipboard, or `none`.             |
| `clipboard sequence?` | The last clipboard escape sequence written, or `none`.    |
| `theme?`          | The name of the theme in use.                                 |
| `foreground?`     | The text colour at the `look for` text.                       |
| `background?`     | The background colour at the `look for` text.                 |
| `contrast?`       | The contrast between those two colours.                       |
| `bar against page?` | The contrast between the status bar and the page.           |
| `unpainted cells?` | How many cells use the terminal's default colours.           |
