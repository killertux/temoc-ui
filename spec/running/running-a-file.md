# Running a File

Back to the [specification index](../README.md).

`R` runs every table in the open test file, using one temoc run. Tables in the same
file share a run, so symbols set by one table are available to later tables.

`--output jsonl` is always passed, because that is the [output](temoc-output.md)
temoc-ui reads.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation        | workspace  | arguments  | keys | temoc arguments?                        |
| ------------------ | ---------- | ---------- | ---- | --------------------------------------- |
| Before a run       | calculator | passing.md |      | none                                    |
| Runs the open file | calculator | passing.md | `R`  | -c temoc.toml --output jsonl passing.md |

## Results

The file gets one overall result, every table gets its own, and so does
[every row](row-results.md). A table takes the worst result of its rows, and a file
the worst result of its tables, in this order: `error`, `failed`, `partial`,
`snoozed`, `skipped`, `passed`.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation         | workspace  | arguments  | keys | file result? | table results?                         |
| ------------------- | ---------- | ---------- | ---- | ------------ | -------------------------------------- |
| Before a run        | calculator | passing.md |      | not run      | Addition: not run, Multiplication: not run |
| Everything passes   | calculator | passing.md | `R`  | passed       | Addition: passed, Multiplication: passed |
| One table fails     | calculator | failing.md | `R`  | failed       | Addition: passed, Multiplication: failed |
| One table is snoozed | calculator | snoozed.md | `R` | snoozed      | Working: passed, Not Ready: snoozed    |
| A real failure wins | calculator | mixed.md   | `R`  | failed       | Broken Now: failed, Not Ready: snoozed |
| An exception        | outcomes   | errors.md  | `R`  | error        | Division: error, Unknown Symbol: error, No Such Fixture: error |

### On screen

Results appear in place. Each table that has a result gets a line below it, and the
rest of the page moves down to make room. The file result sits at the right of the
status bar. A page that has not run looks the same as any other page.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation        | workspace  | arguments  | keys | look for      | visible? |
| ------------------ | ---------- | ---------- | ---- | ------------- | -------- |
| No table result yet | calculator | failing.md |     | failed        | no       |
| No file result yet | calculator | failing.md |      | file:         | no       |
| A passing table    | calculator | failing.md | `R`  | ✓ passed      | yes      |
| A failing table    | calculator | failing.md | `R`  | ✗ failed      | yes      |
| A snoozed table    | calculator | snoozed.md | `R`  | ~ snoozed     | yes      |
| A table with an error | outcomes | errors.md  | `R`  | ! error       | yes      |
| A skipped table    | outcomes   | dying.md   | `R`  | - skipped     | yes      |
| The file result    | calculator | failing.md | `R`  | file: failed  | yes      |
| A partial result   | calculator | failing.md | `r`  | file: partial | yes      |
| A failure is shown | calculator | failing.md | `R`  | mul expected 5, got 4 | yes |

## Failures

Each failure names its table and line, and what was expected. It shows in a
[note under its row](row-results.md#notes), and the row is highlighted: green when it
passed, red for a real failure, and yellow for a snoozed one.

[//]: # "decisionTable Tui"

| # situation    | workspace  | arguments  | keys | failures?                                                             |
| -------------- | ---------- | ---------- | ---- | --------------------------------------------------------------------- |
| No failures    | calculator | passing.md | `R`  | none                                                                  |
| Wrong value    | calculator | failing.md | `R`  | Multiplication line 20: mul expected 5, got 4                         |
| Snoozed        | calculator | snoozed.md | `R`  | Not Ready line 20: mul expected 5, got 4 (snoozed until 2099-12-31)   |
| Both together  | calculator | mixed.md   | `R`  | Broken Now line 11: sum expected 9, got 3; Not Ready line 20: mul expected 5, got 4 (snoozed until 2099-12-31) |

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation        | workspace  | arguments  | keys | look for      | highlight? |
| ------------------ | ---------- | ---------- | ---- | ------------- | ---------- |
| Before a run       | calculator | failing.md |      | 2 │ 2 │ 5     | none       |
| A failing row      | calculator | failing.md | `R`  | 2 │ 2 │ 5     | red        |
| A passing row      | calculator | failing.md | `R`  | 2 │ 3 │ 6     | green      |
| A snoozed row      | calculator | snoozed.md | `R`  | 2 │ 2 │ 5     | yellow     |
| Real beside snoozed | calculator | mixed.md  | `R`  | 1 │ 2 │ 9     | red        |
| Snoozed beside real | calculator | mixed.md  | `R`  | 2 │ 2 │ 5     | yellow     |

## Keeping results

Results stay while you navigate, until temoc-ui closes or the file runs again.

[//]: # "decisionTable Tui"

| # situation            | workspace  | arguments  | keys                      | document?  | file result? |
| ---------------------- | ---------- | ---------- | ------------------------- | ---------- | ------------ |
| Kept after going away  | calculator | passing.md | `R / "failing" Enter h`   | passing.md | passed       |

## While temoc runs

A run happens in the background, so you can keep reading and moving around, and the
status bar says what is running. Results [arrive row by row](row-results.md#while-a-run-goes-on)
and go to the file that ran, even if you have left it. Only one run can go at a time:
running again is refused with `temoc is already running`. Quitting stops temoc and the
Slim server it started.

## Pages without tests

[//]: # "decisionTable Tui"

| # situation     | workspace  | arguments | keys | temoc arguments? | status?                      |
| --------------- | ---------- | --------- | ---- | ---------------- | ---------------------------- |
| No tests here   | calculator | README.md | `R`  | none             | This file has no temoc tests |

## When temoc cannot finish

temoc stops early when a file is malformed or the Slim server does not respond.
That is an error, not a test failure, and temoc's message is shown.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation          | workspace      | arguments | keys | file result? | status?                                                                                          |
| -------------------- | -------------- | --------- | ---- | ------------ | ------------------------------------------------------------------------------------------------ |
| Malformed file       | calculator     | broken.md | `R`  | error        | ``temoc failed: Expected a test table. Got `This paragraph should have been a table.` at broken.md:9:1`` |
| Server will not start | server-failure | tests.md | `R`  | error        | temoc failed: failed to fill whole buffer (server exit status: 1)                                |
| Server stops midway   | outcomes       | dying.md | `R`  | error        | temoc failed: Missing message length terminator (server exit status: 3)                          |

What ran before that keeps its result, and temoc reports the rest as `skipped`. A
file that could not be read never started, so its tables have no result, whatever
they had before. The file result stays `error` until a run finishes.

[//]: # "decisionTable Tui -- snooze until 2099-12-31"

| # situation     | workspace      | arguments | keys | file result? | table results?                         |
| --------------- | -------------- | --------- | ---- | ------------ | -------------------------------------- |
| Nothing ran     | server-failure | tests.md  | `R`  | error        | Unconfigured Tests: skipped            |
| Running a table | server-failure | tests.md  | `r`  | error        | Unconfigured Tests: skipped            |
| Stopped midway  | outcomes       | dying.md  | `R`  | error        | Stopped: error, Never Reached: skipped |
