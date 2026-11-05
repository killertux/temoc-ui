# How the GUI Pages Work

Back to the [specification index](../README.md).

These pages describe what only the GUI does, and prove that its buttons, keys and panels
reach the [core behavior](../writing-specs.md#core-and-interfaces). A rule that any
interface would follow belongs in the core, so a table here only needs enough rows to
show that the GUI is connected to it.

The tables test behavior: what opens, what runs, what is on screen, and what a click or
a key does. They do not test colours, sizes or positions. How the GUI looks is in the
[design](design/README.md), and these pages link to it.

The GUI is partly built, and a table that is still to come is snoozed.

The `Gui` fixture starts a fresh temoc-ui with `--gui` for each row, in a headless
window of 1000 by 700, using `egui_kittest`. It takes the actions, waits for any test run
to finish, and then reads what is on screen through the window's accessibility tree, so
that it finds things by name and not by position.

### Inputs

| Column         | Meaning                                                                 |
| -------------- | ----------------------------------------------------------------------- |
| `workspace`    | The folder under [`samples/`](../../samples) to start in.               |
| `arguments`    | Command line arguments after `--gui`. Empty means none.                 |
| `actions`      | What to do, in order, separated by spaces.                              |
| `look for`     | Text to check for. Used with `visible?`, `sideways?`, and `mark shown?`. |
| `row`          | The line of a test row. Used with `note?`.                              |
| `file`         | A file of the workspace. Used with `node result?`.                      |
| `system mode`  | What the system says: `dark`, `light`, or empty when it does not say.   |

### Actions

Keys are written as on the [terminal pages](../tui/writing-specs.md#keys-notation): `Down`,
`Enter`, `Ctrl-g`, `Alt-Shift-Up`, or `"some text"` to type each character in turn.
Everything `Ctrl` does here is `Cmd` on macOS. Keys wait for any test run to finish before the next action.

| Written as                     | Meaning                                                                  |
| ------------------------------ | ------------------------------------------------------------------------ |
| `Wait row LINE`                | Stop waiting for the run, and wait until that row is `running`. Later actions do not wait for the run either. |
| `Click "name"`                 | Click the first control or text with that name.                          |
| `Click "name" in "table"`      | Click the control with that name in the test table with that name.       |
| `Click row LINE`               | Click the test row on that line.                                         |
| `Hover row LINE`               | Move the pointer over the test row on that line.                         |
| `Hover "text"`                 | Move the pointer over that text.                                         |
| `Leave`                        | Move the pointer away from the page.                                     |
| `Click tree "name"`            | Click that row of the file tree.                                         |
| `Click node "file"`            | Click that file in the graph.                                            |
| `Hover node "file"`            | Move the pointer over that file in the graph.                            |
| `Drag node "file"`             | Drag that file in the graph to another place.                            |
| `Drag graph`                   | Drag the background of the graph.                                        |
| `Drag "from" "to"`             | Press on the first character of `from`, release on the last character of `to`. |
| `Wheel "text"`                 | Turn the mouse wheel one step over that text.                            |
| `Wheel sideways "text"`        | Scroll one step sideways over that text, as a trackpad or tilting wheel does. |
| `Mouse back`, `Mouse forward`  | Press that button of the mouse.                                          |
| `Close window`                 | Close the window the way the system does.                                |

### Expected values

| Column                | Meaning                                                                       |
| --------------------- | ----------------------------------------------------------------------------- |
| `document?`           | Path of the open document within the workspace, or `none`.                    |
| `visible?`            | `yes` if the `look for` text is on screen, otherwise `no`.                    |
| `status?`             | The status bar message, or `none`.                                            |
| `banner?`             | The text of the banner above the page, or `none`.                             |
| `still open?`         | `yes` while the window is open, `no` after it closes.                         |
| `page scroll?`        | Where the page is scrolled to: `top`, `middle`, or `bottom`.                  |
| `sideways?`           | Where the block holding the `look for` text is scrolled to: `start`, `middle`, or `end`. |
| `focused link?`       | The text of the focused link, or `none`.                                      |
| `opened url?`         | The address last opened in the web browser, or `none`.                        |
| `search open?`        | `yes` while the search box is showing.                                        |
| `results?`            | Search results in order, separated by commas, or `none`.                      |
| `tree open?`          | `yes` while the file tree is showing.                                         |
| `tree?`               | The rows of the file tree in order. A folder ends with `/`.                   |
| `tree selection?`     | The row of the file tree that is selected, or `none`.                         |
| `tree test files?`    | The rows of the file tree marked as test files, or `none`.                    |
| `tree marks?`         | Each row of the file tree with a mark, written `file: mark`, or `none`.       |
| `tests panel open?`   | `yes` while the tests panel is showing.                                       |
| `tests panel?`        | The tables listed in the tests panel, written `name: result`, or `none`.      |
| `not passed?`         | What the tests panel lists as not passed, separated by semicolons, or `none`. |
| `last command?`       | The command shown in the tests panel, or `none`.                              |
| `graph open?`         | `yes` while the graph shows.                                                  |
| `selected file?`      | The file selected in the graph, or `none`.                                    |
| `distance?`           | The distance shown for the selected file.                                     |
| `links out?`          | The links out of the selected file, or `none`.                                |
| `links in?`           | The links into the selected file, or `none`.                                  |
| `lit files?`          | The files the graph is drawing attention to, separated by commas.             |
| `arrowheads?`         | How many arrowheads the graph shows.                                          |
| `node moved?`         | `yes` if the file dragged is somewhere else in the graph, otherwise `no`.     |
| `others moved?`       | `yes` if any other file in the graph moved, once it settled.                  |
| `settled?`            | `yes` once the files of the graph have stopped moving.                        |
| `same layout?`        | `yes` if opening the graph again puts every file in the same place.           |
| `node result?`        | The result the graph shows for `file`, or `none`.                             |
| `focused table?`      | The name of the focused test table, or `none`.                                |
| `focused row?`        | The line of the focused row, or `none`.                                       |
| `run file?`           | The Run file button: `enabled`, `disabled`, or `none` when it is not there.   |
| `run row?`            | The Run row buttons showing, written `line: enabled` or `line: disabled`, or `none`. |
| `table results?`      | Each table's name and result, separated by commas.                            |
| `row results?`        | Each row that has a result, written `line: result`, or `none`.                |
| `marks?`              | Each row that has a mark, written `line: mark`, or `none`.                    |
| `mark shown?`         | `yes` if the mark of the row holding `look for` is on screen, otherwise `no`. |
| `note?`               | The note under the test row on line `row`, or `none`.                         |
| `table verdicts?`     | What each table says below itself, separated by commas, or `none`.            |
| `file result?`        | The result for the whole document, or `none`.                                 |
| `temoc arguments?`    | The arguments of the last temoc command, or `none`.                           |
| `image shown?`        | The pictures drawn on the page, or `none`.                                    |
| `checked?`            | Each task list item and whether it is ticked, or `none`.                      |
| `selection?`          | The selected text, or `none`.                                                 |
| `clipboard?`          | The text last copied to the clipboard, or `none`.                             |
| `mode?`               | `dark` or `light`.                                                            |
| `help open?`          | `yes` while help is showing.                                                  |
