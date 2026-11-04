# Running a Table

Back to the [specification index](../README.md).

## Choosing a table

The first table is focused when a test file opens. Move between tables with
[`]` and `[`](../viewer/navigation.md#moving-between-tables), or
[click one](../viewer/mouse.md#selecting-a-table). The header row of the focused
table is drawn reversed, so that you can see what `r` will run.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys | look for | highlight? |
| ------------------ | ---------- | ---------- | ---- | -------- | ---------- |
| The first table    | calculator | passing.md |      | sum?     | reversed   |
| Not the others     | calculator | passing.md |      | mul?     | none       |
| Follows the focus  | calculator | passing.md | `]`  | mul?     | reversed   |
| Leaves the old one | calculator | passing.md | `]`  | sum?     | none       |

## Running it

`r` runs the focused table. temoc selects a table by the line number of its
`decisionTable` directive, so that is the line temoc-ui passes.

Imports still apply to a table run alone, but symbols set by other tables do not.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation  | workspace  | arguments  | keys  | temoc arguments?                           |
| ------------ | ---------- | ---------- | ----- | ------------------------------------------ |
| First table  | calculator | passing.md | `r`   | -c temoc.toml --output jsonl passing.md:7  |
| Second table | calculator | passing.md | `] r` | -c temoc.toml --output jsonl passing.md:15 |

## Results

Only the table that ran changes. Until every table has a result, the file result is
`partial`, unless a table failed.

[//]: # "decisionTable Tui"

| # situation          | workspace  | arguments  | keys      | file result? | table results?                           |
| -------------------- | ---------- | ---------- | --------- | ------------ | ---------------------------------------- |
| Passing table        | calculator | failing.md | `r`       | partial      | Addition: passed, Multiplication: not run |
| Failing table        | calculator | failing.md | `] r`     | failed       | Addition: not run, Multiplication: failed |
| Both, one at a time  | calculator | passing.md | `r ] r`   | passed       | Addition: passed, Multiplication: passed |
| Snoozed table        | calculator | snoozed.md | `] r`     | partial      | Working: not run, Not Ready: snoozed     |
| Keeps other results  | calculator | failing.md | `R r`     | failed       | Addition: passed, Multiplication: failed |

## Nothing to run

`r` needs a focused table. A page may have none, either because it has no tests or
because its only directive has no table. temoc reads the whole file before choosing
a table, so one malformed table stops every table in that file from running. See
[when temoc cannot finish](running-a-file.md#when-temoc-cannot-finish).

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments | keys | temoc arguments? | status?              |
| ------------------ | ---------- | --------- | ---- | ---------------- | -------------------- |
| Page without tests | calculator | README.md | `r`  | none             | No test table to run |
| Directive only     | calculator | broken.md | `r`  | none             | No test table to run |
