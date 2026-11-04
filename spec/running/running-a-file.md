# Running a File

Back to the [specification index](../README.md).

`R` runs every table in the open test file, using one temoc run. Tables in the same
file share a run, so symbols set by one table are available to later tables.

The `-s` option is always passed because temoc hides snoozed failures without it.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys | temoc arguments?            |
| ------------------ | ---------- | ---------- | ---- | --------------------------- |
| Before a run       | calculator | passing.md |      | none                        |
| Runs the open file | calculator | passing.md | `R`  | -c temoc.toml -s passing.md |

## Results

The file gets one overall result, and every table gets its own.

[//]: # "decisionTable Tui"

| # situation         | workspace  | arguments  | keys | file result? | table results?                         |
| ------------------- | ---------- | ---------- | ---- | ------------ | -------------------------------------- |
| Before a run        | calculator | passing.md |      | not run      | Addition: not run, Multiplication: not run |
| Everything passes   | calculator | passing.md | `R`  | passed       | Addition: passed, Multiplication: passed |
| One table fails     | calculator | failing.md | `R`  | failed       | Addition: passed, Multiplication: failed |
| One table is snoozed | calculator | snoozed.md | `R` | snoozed      | Working: passed, Not Ready: snoozed    |
| A real failure wins | calculator | mixed.md   | `R`  | failed       | Broken Now: failed, Not Ready: snoozed |

## Failures

Each failure names its table and line, and what was expected. The failing row is
highlighted in the table.

[//]: # "decisionTable Tui"

| # situation    | workspace  | arguments  | keys | failures?                                                             |
| -------------- | ---------- | ---------- | ---- | --------------------------------------------------------------------- |
| No failures    | calculator | passing.md | `R`  | none                                                                  |
| Wrong value    | calculator | failing.md | `R`  | Multiplication line 20: mul expected 5, got 4                         |
| Snoozed        | calculator | snoozed.md | `R`  | Not Ready line 20: mul expected 5, got 4 (snoozed until 2099-12-31)   |
| Both together  | calculator | mixed.md   | `R`  | Broken Now line 11: sum expected 9, got 3; Not Ready line 20: mul expected 5, got 4 (snoozed until 2099-12-31) |

## Keeping results

Results stay while you navigate, until temoc-ui closes or the file runs again.

[//]: # "decisionTable Tui"

| # situation            | workspace  | arguments  | keys                      | document?  | file result? |
| ---------------------- | ---------- | ---------- | ------------------------- | ---------- | ------------ |
| Kept after going away  | calculator | passing.md | `R / "failing" Enter h`   | passing.md | passed       |

## Pages without tests

[//]: # "decisionTable Tui"

| # situation     | workspace  | arguments | keys | temoc arguments? | status?                      |
| --------------- | ---------- | --------- | ---- | ---------------- | ---------------------------- |
| No tests here   | calculator | README.md | `R`  | none             | This file has no temoc tests |

## When temoc cannot finish

temoc stops without a result when a file is malformed or the Slim server does not
respond. That is an error, not a test failure, and temoc's message is shown.

[//]: # "decisionTable Tui"

| # situation          | workspace      | arguments | keys | file result? | status?                                                                                          |
| -------------------- | -------------- | --------- | ---- | ------------ | ------------------------------------------------------------------------------------------------ |
| Malformed file       | calculator     | broken.md | `R`  | error        | temoc failed: Expected a test table. Got `This paragraph should have been a table.` at broken.md:9:1 |
| Server will not start | server-failure | tests.md | `R`  | error        | temoc failed: failed to fill whole buffer                                                        |
