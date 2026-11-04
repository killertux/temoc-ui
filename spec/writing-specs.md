# How These Pages Work

Back to the [specification index](README.md).

Every page in this folder is both documentation and a temoc test. We always write
the spec first. The new tests fail because the feature does not exist yet, and
then we write the code that makes them pass.

Run the whole specification from the repository root:

```sh
temoc -c temoc.toml
```

## Reading a test table

Each row is one independent scenario. Columns ending in `?` are what we expect to
see. The other columns are inputs. Columns starting with `#` are comments that
temoc ignores.

Most pages use the `Tui` fixture. For each row it starts a fresh temoc-ui in an
80 by 24 in-memory terminal, presses the keys, waits for any test run to finish,
and then reads the screen.

### Inputs

| Column      | Meaning                                                             |
| ----------- | ------------------------------------------------------------------- |
| `workspace` | The folder under [`samples/`](../samples) to start in.              |
| `arguments` | Command line arguments for temoc-ui. Empty means none.              |
| `keys`      | The keys to press, in order, separated by spaces.                   |
| `look for`  | Text to check for. Used with `visible?` or `in graph?`.             |

### Keys notation

| Written as                                 | Meaning                       |
| ------------------------------------------ | ----------------------------- |
| `j`, `/`, `G`                              | Press that key.               |
| `Enter`, `Esc`, `Tab`, `Backspace`         | Press that named key.         |
| `Shift-Tab`, `PageUp`, `PageDown`          | Press that named key.         |
| `Up`, `Down`, `Left`, `Right`, `Space`     | Press that named key.         |
| `Ctrl-t`                                   | Hold Ctrl and press the key.  |
| `"some text"`                              | Type each character in turn.  |

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
| `status?`         | The status bar message, or `none`.                            |
| `still open?`     | `yes` while temoc-ui is running, `no` after it exits.         |
| `focused link?`   | The text of the focused link, or `none`.                      |
| `search open?`    | `yes` while the search box is showing.                        |
| `results?`        | Search results in order, separated by commas, or `none`.      |
| `first result?`   | The top search result, or `none`.                             |
| `focused table?`  | The name of the focused test table, or `none`.                |
| `table results?`  | Each table's name and result, separated by commas.            |
| `file result?`    | The result for the whole document.                            |
| `failures?`       | Failures listed on screen, separated by semicolons, or `none`.|
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

## Sample workspaces

The workspaces live in [`samples/`](../samples), outside this folder so that
running the specification does not run them as tests too.

| Workspace                                     | Used for                          |
| --------------------------------------------- | --------------------------------- |
| [`docs`](../samples/docs/README.md)           | Viewing, navigating, and search.  |
| [`calculator`](../samples/calculator/README.md) | Running real tests.             |
| [`detection`](../samples/detection)           | Recognizing test files.           |
| [`unconfigured`](../samples/unconfigured)     | Running without configuration.    |
| [`server-failure`](../samples/server-failure) | A Slim server that will not start.|
| [`custom-config`](../samples/custom-config)   | Using the `--config` option.      |
| [`many-pages`](../samples/many-pages/README.md) | A graph taller than the screen. |
| [`link-chain`](../samples/link-chain/README.md) | A graph wider than the screen.  |
| [`wide-blocks`](../samples/wide-blocks/README.md) | Content wider than the screen.  |

The `calculator` workspace runs real temoc against a small Slim server built by
the spec harness.

## Writing a new page

- Describe the behavior in plain language first, then show it with a table.
- Keep each row independent, so that it can run alone.
- Add a link to the page from the [index](README.md).
- Link back to the index at the top of the page.
