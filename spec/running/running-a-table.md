# Running a Table

Back to the [specification index](../README.md).

temoc selects a table by the line number of its `decisionTable` directive, so that is
the line temoc-ui passes.

Imports still apply to a table run alone, but symbols set by other tables do not.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation  | workspace  | path       | steps          | temoc arguments?                           |
| ------------ | ---------- | ---------- | -------------- | ------------------------------------------ |
| First table  | calculator | passing.md | `run table 7`  | -c temoc.toml --output jsonl passing.md:7  |
| Second table | calculator | passing.md | `run table 15` | -c temoc.toml --output jsonl passing.md:15 |

## Results

Only the table that ran changes. Until every table has a result, the file result is
`partial`, unless a table failed.

[//]: # "decisionTable Core"

| # situation         | workspace  | path       | steps                        | file result? | table results?                            |
| ------------------- | ---------- | ---------- | ---------------------------- | ------------ | ----------------------------------------- |
| Passing table       | calculator | failing.md | `run table 7`                | partial      | Addition: passed, Multiplication: not run |
| Failing table       | calculator | failing.md | `run table 15`               | failed       | Addition: not run, Multiplication: failed |
| Both, one at a time | calculator | passing.md | `run table 7, run table 15`  | passed       | Addition: passed, Multiplication: passed  |
| Snoozed table       | calculator | snoozed.md | `run table 15`               | partial      | Working: not run, Not Ready: snoozed      |
| Keeps other results | calculator | failing.md | `run file, run table 7`      | failed       | Addition: passed, Multiplication: failed  |

## Nothing to run

A table needs a directive on that line, and a table below it. temoc reads the whole
file before choosing a table, so one malformed table stops every table in that file
from running. See [when temoc cannot finish](running-a-file.md#when-temoc-cannot-finish).

[//]: # "decisionTable Core"

| # situation        | workspace  | path       | steps         | temoc arguments? | problem?             |
| ------------------ | ---------- | ---------- | ------------- | ---------------- | -------------------- |
| Page without tests | calculator | README.md  | `run table 7` | none             | No test table to run |
| Not a directive    | calculator | passing.md | `run table 9` | none             | No test table to run |
| Directive only     | calculator | broken.md  | `run table 7` | none             | No test table to run |
