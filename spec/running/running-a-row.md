# Running a Row

Back to the [specification index](../README.md).

## Choosing a row

`}` focuses the next row of the [focused table](running-a-table.md#choosing-a-table)
and `{` the previous one, scrolling it into view. No row is focused at first, so `}`
starts at the first row and `{` at the last. Focus stops at the first and last rows
of the table and does not wrap around. Moving to another table clears it.

The focused row is drawn reversed, like the header of the focused table, so that you
can see what `Ctrl-r` will run. A row can also be
[clicked](../viewer/mouse.md#selecting-a-row).

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments    | keys        | focused row? | focused table? |
| ------------------ | ---------- | ------------ | ----------- | ------------ | -------------- |
| None at first      | calculator | passing.md   |             | none         | Addition       |
| The first row      | calculator | passing.md   | `}`         | 11           | Addition       |
| Stops at the last  | calculator | passing.md   | `} }`       | 11           | Addition       |
| In the second table | calculator | passing.md  | `] }`       | 19           | Multiplication |
| The next row       | calculator | passing.md   | `] } }`     | 20           | Multiplication |
| The previous row   | calculator | passing.md   | `] } } {`   | 19           | Multiplication |
| Stops at the first | calculator | passing.md   | `] } {`     | 19           | Multiplication |
| Backwards at first | calculator | passing.md   | `] {`       | 20           | Multiplication |
| Moving to a table  | calculator | passing.md   | `} ]`       | none         | Multiplication |
| Page without tests | docs       | reference.md | `}`         | none         | none           |

[//]: # "decisionTable Tui"

| # situation      | workspace  | arguments  | keys    | look for  | highlight? |
| ---------------- | ---------- | ---------- | ------- | --------- | ---------- |
| Not focused      | calculator | passing.md | `]`     | 2 │ 3 │ 6 | none       |
| Focused          | calculator | passing.md | `] }`   | 2 │ 3 │ 6 | reversed   |
| The next one     | calculator | passing.md | `] } }` | 2 │ 3 │ 6 | none       |
| Its table's header | calculator | passing.md | `] }` | mul?      | reversed   |
| Over a result    | calculator | failing.md | `R ] } }` | 2 │ 2 │ 5 | reversed |

## Running it

`Ctrl-r` runs the focused row. temoc selects a row by its line, so that is the line
temoc-ui passes. temoc still makes and sets up the row's table.

[//]: # "decisionTable Tui"

| # situation     | workspace  | arguments  | keys           | temoc arguments?                          |
| --------------- | ---------- | ---------- | -------------- | ----------------------------------------- |
| The first row   | calculator | passing.md | `} Ctrl-r`     | -c temoc.toml --output jsonl passing.md:11 |
| A later row     | calculator | passing.md | `] } } Ctrl-r` | -c temoc.toml --output jsonl passing.md:20 |

## Results

Only the row that ran changes. A table's result comes from all of its rows, so until
each row has a result the table is `partial`, unless a row did not pass. That is the
same rule as for a [file and its tables](running-a-table.md#results).

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys                  | row results?                       | table results?                            | file result? |
| ------------------ | ---------- | ---------- | --------------------- | ---------------------------------- | ----------------------------------------- | ------------ |
| A failing row      | calculator | failing.md | `] } } Ctrl-r`        | 20: failed                         | Addition: not run, Multiplication: failed | failed       |
| A passing row      | calculator | failing.md | `] } Ctrl-r`          | 19: passed                         | Addition: not run, Multiplication: partial | partial     |
| A snoozed row      | calculator | snoozed.md | `] } } Ctrl-r`        | 20: snoozed                        | Working: not run, Not Ready: partial      | partial      |
| Every row in turn  | calculator | passing.md | `] } Ctrl-r } Ctrl-r` | 19: passed, 20: passed             | Addition: not run, Multiplication: passed | partial      |
| The only row       | calculator | passing.md | `} Ctrl-r`            | 11: passed                         | Addition: passed, Multiplication: not run | partial      |
| Keeps other rows   | calculator | failing.md | `R ] } Ctrl-r`        | 11: passed, 19: passed, 20: failed | Addition: passed, Multiplication: failed  | failed       |

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys       | look for  | visible? |
| ------------------ | ---------- | ---------- | ---------- | --------- | -------- |
| The table's result | calculator | failing.md | `] } Ctrl-r` | · partial | yes    |

## Symbols

A row run alone cannot see symbols set by other rows or tables, because they never
ran. temoc reports that as an error in the row.

[//]: # "decisionTable Tui"

| # situation      | workspace | arguments  | keys         | row results?           | failures?                                        |
| ---------------- | --------- | ---------- | ------------ | ---------------------- | ------------------------------------------------ |
| The whole file   | outcomes  | symbols.md | `R`          | 11: passed, 19: passed | none                                             |
| A row by itself  | outcomes  | symbols.md | `] } Ctrl-r` | 19: error              | Recall line 19: setA got unknown symbol $TOTAL   |

## Nothing to run

`Ctrl-r` needs a focused row.

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys                                  | temoc arguments? | status?                  |
| ------------------ | ---------- | ---------- | ------------------------------------- | ---------------- | ------------------------ |
| No focused row     | calculator | passing.md | `Ctrl-r`                              | none             | No test row to run       |
| Page without tests | calculator | README.md  | `Ctrl-r`                              | none             | No test row to run       |

[//]: # "decisionTable Tui"

| # situation     | workspace | arguments  | keys                                     | status?                  |
| --------------- | --------- | ---------- | ---------------------------------------- | ------------------------ |
| Already running | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯" } Ctrl-r` | temoc is already running |
