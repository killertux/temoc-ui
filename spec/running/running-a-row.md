# Running a Row

Back to the [specification index](../README.md).

temoc selects a row by its line, so that is the line temoc-ui passes. temoc still
makes and sets up the row's table.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation   | workspace  | path       | steps        | temoc arguments?                           |
| ------------- | ---------- | ---------- | ------------ | ------------------------------------------ |
| The first row | calculator | passing.md | `run row 11` | -c temoc.toml --output jsonl passing.md:11 |
| A later row   | calculator | passing.md | `run row 20` | -c temoc.toml --output jsonl passing.md:20 |

## Results

Only the row that ran changes. A table's result comes from all of its rows, so until
each row has a result the table is `partial`, unless a row did not pass. That is the
same rule as for a [file and its tables](running-a-table.md#results).

[//]: # "decisionTable Core"

| # situation       | workspace  | path       | steps                    | row results?                       | table results?                             | file result? |
| ----------------- | ---------- | ---------- | ------------------------ | ---------------------------------- | ------------------------------------------ | ------------ |
| A failing row     | calculator | failing.md | `run row 20`             | 20: failed                         | Addition: not run, Multiplication: failed  | failed       |
| A passing row     | calculator | failing.md | `run row 19`             | 19: passed                         | Addition: not run, Multiplication: partial | partial      |
| A snoozed row     | calculator | snoozed.md | `run row 20`             | 20: snoozed                        | Working: not run, Not Ready: partial       | partial      |
| Every row in turn | calculator | passing.md | `run row 19, run row 20` | 19: passed, 20: passed             | Addition: not run, Multiplication: passed  | partial      |
| The only row      | calculator | passing.md | `run row 11`             | 11: passed                         | Addition: passed, Multiplication: not run  | partial      |
| Keeps other rows  | calculator | failing.md | `run file, run row 19`   | 11: passed, 19: passed, 20: failed | Addition: passed, Multiplication: failed   | failed       |

## Symbols

A row run alone cannot see symbols set by other rows or tables, because they never
ran. temoc reports that as an error in the row.

[//]: # "decisionTable Core"

| # situation     | workspace | path       | steps        | row results?           | failures?                                      |
| --------------- | --------- | ---------- | ------------ | ---------------------- | ---------------------------------------------- |
| The whole file  | outcomes  | symbols.md | `run file`   | 11: passed, 19: passed | none                                           |
| A row by itself | outcomes  | symbols.md | `run row 19` | 19: error              | Recall line 19: setA got unknown symbol $TOTAL |

## Nothing to run

A row needs to be a row of a test table.

[//]: # "decisionTable Core"

| # situation        | workspace  | path       | steps        | temoc arguments? | problem?           |
| ------------------ | ---------- | ---------- | ------------ | ---------------- | ------------------ |
| Not a row          | calculator | passing.md | `run row 9`  | none             | No test row to run |
| Page without tests | calculator | README.md  | `run row 11` | none             | No test row to run |

[//]: # "decisionTable Core"

| # situation     | workspace | path       | steps                                  | problem?                 |
| --------------- | --------- | ---------- | -------------------------------------- | ------------------------ |
| Already running | outcomes  | hanging.md | `run file, wait for row 13, run row 22` | temoc is already running |
