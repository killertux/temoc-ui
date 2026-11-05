# Running a File

Back to the [specification index](../README.md).

Running a file runs every table in it, using one temoc run. Tables in the same file
share a run, so symbols set by one table are available to later tables.

`--output jsonl` is always passed, because that is the [output](temoc-output.md)
temoc-ui reads.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation        | workspace  | path       | steps      | temoc arguments?                        |
| ------------------ | ---------- | ---------- | ---------- | --------------------------------------- |
| Before a run       | calculator | passing.md |            | none                                    |
| Runs the open file | calculator | passing.md | `run file` | -c temoc.toml --output jsonl passing.md |

## Results

The file gets one overall result, every table gets its own, and so does
[every row](row-results.md). A table takes the worst result of its rows, and a file
the worst result of its tables, in this order: `error`, `failed`, `partial`,
`snoozed`, `skipped`, `passed`.

[//]: # "decisionTable Core"

| # situation          | workspace  | path       | steps      | file result? | table results?                                                 |
| -------------------- | ---------- | ---------- | ---------- | ------------ | -------------------------------------------------------------- |
| Before a run         | calculator | passing.md |            | not run      | Addition: not run, Multiplication: not run                     |
| Everything passes    | calculator | passing.md | `run file` | passed       | Addition: passed, Multiplication: passed                       |
| One table fails      | calculator | failing.md | `run file` | failed       | Addition: passed, Multiplication: failed                       |
| One table is snoozed | calculator | snoozed.md | `run file` | snoozed      | Working: passed, Not Ready: snoozed                            |
| A real failure wins  | calculator | mixed.md   | `run file` | failed       | Broken Now: failed, Not Ready: snoozed                         |
| An exception         | outcomes   | errors.md  | `run file` | error        | Division: error, Unknown Symbol: error, No Such Fixture: error |

## Failures

Each failure names its table and line, and what was expected.

[//]: # "decisionTable Core"

| # situation   | workspace  | path       | steps      | failures?                                                                                                      |
| ------------- | ---------- | ---------- | ---------- | -------------------------------------------------------------------------------------------------------------- |
| No failures   | calculator | passing.md | `run file` | none                                                                                                           |
| Wrong value   | calculator | failing.md | `run file` | Multiplication line 20: mul expected 5, got 4                                                                  |
| Snoozed       | calculator | snoozed.md | `run file` | Not Ready line 20: mul expected 5, got 4 (snoozed until 2099-12-31)                                            |
| Both together | calculator | mixed.md   | `run file` | Broken Now line 11: sum expected 9, got 3; Not Ready line 20: mul expected 5, got 4 (snoozed until 2099-12-31) |

## Keeping results

Results stay while you navigate, until temoc-ui closes or the file runs again.

[//]: # "decisionTable Core"

| # situation           | workspace  | path       | steps                             | document?  | file result? |
| --------------------- | ---------- | ---------- | --------------------------------- | ---------- | ------------ |
| Kept after going away | calculator | passing.md | `run file, open failing.md, back` | passing.md | passed       |

## While temoc runs

A run happens in the background, so you can keep reading and moving around. Results
[arrive row by row](row-results.md#while-a-run-goes-on) and go to the file that ran,
even if you have left it. Only one run can go at a time: running again is refused with
`temoc is already running`. Closing temoc-ui stops temoc and the Slim server it started.

## Pages without tests

A page without tests has no result at all, not even `not run`.

[//]: # "decisionTable Core"

| # situation   | workspace  | path      | steps      | temoc arguments? | file result? | problem?                     |
| ------------- | ---------- | --------- | ---------- | ---------------- | ------------ | ---------------------------- |
| Before a run  | calculator | README.md |            | none             | none         | none                         |
| No tests here | calculator | README.md | `run file` | none             | none         | This file has no temoc tests |

## When temoc cannot finish

temoc stops early when a file is malformed or the Slim server does not respond.
That is an error, not a test failure, and temoc's message is shown.

[//]: # "decisionTable Core"

| # situation           | workspace      | path      | steps      | file result? | problem?                                                                                                 |
| --------------------- | -------------- | --------- | ---------- | ------------ | -------------------------------------------------------------------------------------------------------- |
| Malformed file        | calculator     | broken.md | `run file` | error        | ``temoc failed: Expected a test table. Got `This paragraph should have been a table.` at broken.md:9:1`` |
| Server will not start | server-failure | tests.md  | `run file` | error        | temoc failed: failed to fill whole buffer (server exit status: 1)                                        |
| Server stops midway   | outcomes       | dying.md  | `run file` | error        | temoc failed: Missing message length terminator (server exit status: 3)                                  |

What ran before that keeps its result, and temoc reports the rest as `skipped`. A
file that could not be read never started, so its tables have no result, whatever
they had before. The file result stays `error` until a run finishes.

[//]: # "decisionTable Core"

| # situation     | workspace      | path     | steps         | file result? | table results?                         |
| --------------- | -------------- | -------- | ------------- | ------------ | -------------------------------------- |
| Nothing ran     | server-failure | tests.md | `run file`    | error        | Unconfigured Tests: skipped            |
| Running a table | server-failure | tests.md | `run table 3` | error        | Unconfigured Tests: skipped            |
| Stopped midway  | outcomes       | dying.md | `run file`    | error        | Stopped: error, Never Reached: skipped |
