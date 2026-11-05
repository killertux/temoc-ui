# Running Tests

Back to the [specification index](../README.md).

`R` [runs the open file](../running/running-a-file.md), `r`
[the focused table](../running/running-a-table.md) and `Ctrl-r`
[the focused row](../running/running-a-row.md).

## Running a file

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys | temoc arguments?                        |
| ------------------ | ---------- | ---------- | ---- | --------------------------------------- |
| Before a run       | calculator | passing.md |      | none                                    |
| Runs the open file | calculator | passing.md | `R`  | -c temoc.toml --output jsonl passing.md |

[//]: # "decisionTable Tui"

| # situation     | workspace  | arguments  | keys | file result? | table results?                             | failures?                                     |
| --------------- | ---------- | ---------- | ---- | ------------ | ------------------------------------------ | --------------------------------------------- |
| Before a run    | calculator | passing.md |      | not run      | Addition: not run, Multiplication: not run | none                                          |
| One table fails | calculator | failing.md | `R`  | failed       | Addition: passed, Multiplication: failed   | Multiplication line 20: mul expected 5, got 4 |

### On screen

Results appear in place. Each table that has a result gets a line below it, and the
rest of the page moves down to make room. The file result sits at the right of the
status bar. A page that has not run looks the same as any other page.

[//]: # "decisionTable Tui"

| # situation           | workspace  | arguments  | keys | look for              | visible? |
| --------------------- | ---------- | ---------- | ---- | --------------------- | -------- |
| No table result yet   | calculator | failing.md |      | failed                | no       |
| No file result yet    | calculator | failing.md |      | file:                 | no       |
| A passing table       | calculator | failing.md | `R`  | ✓ passed              | yes      |
| A failing table       | calculator | failing.md | `R`  | ✗ failed              | yes      |
| A snoozed table       | calculator | snoozed.md | `R`  | ~ snoozed             | yes      |
| A table with an error | outcomes   | errors.md  | `R`  | ! error               | yes      |
| A skipped table       | outcomes   | dying.md   | `R`  | - skipped             | yes      |
| The file result       | calculator | failing.md | `R`  | file: failed          | yes      |
| A partial result      | calculator | failing.md | `r`  | file: partial         | yes      |
| A failure is shown    | calculator | failing.md | `R`  | mul expected 5, got 4 | yes      |

A failure shows in a [note under its row](row-results.md#notes), and the row is
highlighted: green when it passed, red for a real failure, and yellow for a snoozed
one.

[//]: # "decisionTable Tui"

| # situation         | workspace  | arguments  | keys | look for  | highlight? |
| ------------------- | ---------- | ---------- | ---- | --------- | ---------- |
| Before a run        | calculator | failing.md |      | 2 │ 2 │ 5 | none       |
| A failing row       | calculator | failing.md | `R`  | 2 │ 2 │ 5 | red        |
| A passing row       | calculator | failing.md | `R`  | 2 │ 3 │ 6 | green      |
| A snoozed row       | calculator | snoozed.md | `R`  | 2 │ 2 │ 5 | yellow     |
| Real beside snoozed | calculator | mixed.md   | `R`  | 1 │ 2 │ 9 | red        |
| Snoozed beside real | calculator | mixed.md   | `R`  | 2 │ 2 │ 5 | yellow     |

Results stay on the page while you navigate.

[//]: # "decisionTable Tui"

| # situation           | workspace  | arguments  | keys                    | document?  | file result? |
| --------------------- | ---------- | ---------- | ----------------------- | ---------- | ------------ |
| Kept after going away | calculator | passing.md | `R / "failing" Enter h` | passing.md | passed       |

## Choosing a table

The first table is focused when a test file opens. Move between tables with
[`]` and `[`](navigation.md#moving-between-tables), or
[click one](mouse.md#selecting-a-table). The header row of the focused table is drawn
reversed, so that you can see what `r` will run.

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys | look for | highlight? |
| ------------------ | ---------- | ---------- | ---- | -------- | ---------- |
| The first table    | calculator | passing.md |      | sum?     | reversed   |
| Not the others     | calculator | passing.md |      | mul?     | none       |
| Follows the focus  | calculator | passing.md | `]`  | mul?     | reversed   |
| Leaves the old one | calculator | passing.md | `]`  | sum?     | none       |

## Running a table

`r` runs the focused table.

[//]: # "decisionTable Tui"

| # situation  | workspace  | arguments  | keys  | temoc arguments?                           |
| ------------ | ---------- | ---------- | ----- | ------------------------------------------ |
| First table  | calculator | passing.md | `r`   | -c temoc.toml --output jsonl passing.md:7  |
| Second table | calculator | passing.md | `] r` | -c temoc.toml --output jsonl passing.md:15 |

[//]: # "decisionTable Tui"

| # situation   | workspace  | arguments  | keys  | file result? | table results?                            |
| ------------- | ---------- | ---------- | ----- | ------------ | ----------------------------------------- |
| Passing table | calculator | failing.md | `r`   | partial      | Addition: passed, Multiplication: not run |
| Failing table | calculator | failing.md | `] r` | failed       | Addition: not run, Multiplication: failed |

## Choosing a row

`}` focuses the next row of the [focused table](#choosing-a-table) and `{` the previous
one, scrolling it into view. No row is focused at first, so `}` starts at the first
row and `{` at the last. Focus stops at the first and last rows of the table and does
not wrap around. Moving to another table clears it.

The focused row is drawn reversed, like the header of the focused table, so that you
can see what `Ctrl-r` will run. A row can also be
[clicked](mouse.md#selecting-a-row).

[//]: # "decisionTable Tui"

| # situation         | workspace  | arguments    | keys      | focused row? | focused table? |
| ------------------- | ---------- | ------------ | --------- | ------------ | -------------- |
| None at first       | calculator | passing.md   |           | none         | Addition       |
| The first row       | calculator | passing.md   | `}`       | 11           | Addition       |
| Stops at the last   | calculator | passing.md   | `} }`     | 11           | Addition       |
| In the second table | calculator | passing.md   | `] }`     | 19           | Multiplication |
| The next row        | calculator | passing.md   | `] } }`   | 20           | Multiplication |
| The previous row    | calculator | passing.md   | `] } } {` | 19           | Multiplication |
| Stops at the first  | calculator | passing.md   | `] } {`   | 19           | Multiplication |
| Backwards at first  | calculator | passing.md   | `] {`     | 20           | Multiplication |
| Moving to a table   | calculator | passing.md   | `} ]`     | none         | Multiplication |
| Page without tests  | docs       | reference.md | `}`       | none         | none           |

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys      | look for  | highlight? |
| ------------------ | ---------- | ---------- | --------- | --------- | ---------- |
| Not focused        | calculator | passing.md | `]`       | 2 │ 3 │ 6 | none       |
| Focused            | calculator | passing.md | `] }`     | 2 │ 3 │ 6 | reversed   |
| The next one       | calculator | passing.md | `] } }`   | 2 │ 3 │ 6 | none       |
| Its table's header | calculator | passing.md | `] }`     | mul?      | reversed   |
| Over a result      | calculator | failing.md | `R ] } }` | 2 │ 2 │ 5 | reversed   |

## Running a row

`Ctrl-r` runs the focused row.

[//]: # "decisionTable Tui"

| # situation   | workspace  | arguments  | keys           | temoc arguments?                           |
| ------------- | ---------- | ---------- | -------------- | ------------------------------------------ |
| The first row | calculator | passing.md | `} Ctrl-r`     | -c temoc.toml --output jsonl passing.md:11 |
| A later row   | calculator | passing.md | `] } } Ctrl-r` | -c temoc.toml --output jsonl passing.md:20 |

[//]: # "decisionTable Tui"

| # situation   | workspace  | arguments  | keys           | row results? | table results?                            | file result? |
| ------------- | ---------- | ---------- | -------------- | ------------ | ----------------------------------------- | ------------ |
| A failing row | calculator | failing.md | `] } } Ctrl-r` | 20: failed   | Addition: not run, Multiplication: failed | failed       |

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys         | look for  | visible? |
| ------------------ | ---------- | ---------- | ------------ | --------- | -------- |
| The table's result | calculator | failing.md | `] } Ctrl-r` | · partial | yes      |

## Nothing to run

The status bar says when there is nothing to run, or why it cannot run.

[//]: # "decisionTable Tui"

| # situation        | workspace    | arguments  | keys       | temoc arguments? | status?                                                       |
| ------------------ | ------------ | ---------- | ---------- | ---------------- | ------------------------------------------------------------- |
| A file, no tests   | calculator   | README.md  | `R`        | none             | This file has no temoc tests                                  |
| No focused table   | calculator   | README.md  | `r`        | none             | No test table to run                                          |
| Directive only     | calculator   | broken.md  | `r`        | none             | No test table to run                                          |
| No focused row     | calculator   | passing.md | `Ctrl-r`   | none             | No test row to run                                            |
| Page without tests | calculator   | README.md  | `Ctrl-r`   | none             | No test row to run                                            |
| No configuration   | unconfigured | tests.md   | `R`        | none             | No temoc configuration found. Add temoc.toml or use --config. |
| A table            | unconfigured | tests.md   | `r`        | none             | No temoc configuration found. Add temoc.toml or use --config. |
| A row              | unconfigured | tests.md   | `} Ctrl-r` | none             | No temoc configuration found. Add temoc.toml or use --config. |

[//]: # "decisionTable Tui"

| # situation     | workspace | arguments  | keys                                         | status?                  |
| --------------- | --------- | ---------- | -------------------------------------------- | ------------------------ |
| Already running | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯" } Ctrl-r` | temoc is already running |

## When temoc cannot finish

[What temoc reports](../running/running-a-file.md#when-temoc-cannot-finish) goes in the
status bar.

[//]: # "decisionTable Tui"

| # situation         | workspace | arguments | keys | file result? | status?                                                                 |
| ------------------- | --------- | --------- | ---- | ------------ | ----------------------------------------------------------------------- |
| Server stops midway | outcomes  | dying.md  | `R`  | error        | temoc failed: Missing message length terminator (server exit status: 3) |
