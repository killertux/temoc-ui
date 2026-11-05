# Row Results

Back to the [specification index](../README.md).

Every row that runs shows its [result](../running/row-results.md) as soon as temoc
finishes that row, while the rest of the file is still running.

## Marks and colours

A row with a result gets a mark and a colour. Colour alone would leave out anyone
who cannot tell red from green, and the mark alone is easy to miss in a long table.

| Result  | Mark | Colour |
| ------- | ---- | ------ |
| passed  | ✓    | green  |
| failed  | ✗    | red    |
| error   | !    | red    |
| snoozed | ~    | yellow |
| skipped | -    | none   |
| running | ⋯    | none   |

A row that has no result yet has no mark.

The mark sits two columns after the end of the row, so that the marks of a table
line up and the table itself does not move. The colour covers the whole row and its
mark, but not the header. In a table wider than the screen the mark can be out of
sight until you scroll sideways, and the colour still shows.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation  | workspace  | arguments  | keys  | row results?                       |
| ------------ | ---------- | ---------- | ----- | ---------------------------------- |
| Before a run | calculator | failing.md |       | none                               |
| A whole file | calculator | failing.md | `R`   | 11: passed, 19: passed, 20: failed |
| A table      | calculator | failing.md | `] r` | 19: passed, 20: failed             |

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments  | keys | look for                    | visible? |
| ------------------ | ---------- | ---------- | ---- | --------------------------- | -------- |
| No marks at first  | calculator | failing.md |      | ✓                           | no       |
| Passed             | calculator | failing.md | `R`  | 1 │ 2 │ 3     ✓             | yes      |
| Failed             | calculator | failing.md | `R`  | 2 │ 2 │ 5     ✗             | yes      |
| Snoozed            | calculator | snoozed.md | `R`  | 2 │ 2 │ 5     ~             | yes      |
| Error              | outcomes   | errors.md  | `R`  | 1 │ 0 │ 0     !             | yes      |
| Skipped            | outcomes   | dying.md   | `R`  | 3 │ 3 │ 6             -     | yes      |
| A table that ran   | calculator | failing.md | `r`  | 1 │ 2 │ 3     ✓             | yes      |
| A table that didn't | calculator | failing.md | `r` | ✗                           | no       |

[//]: # "decisionTable Tui"

| # situation    | workspace  | arguments  | keys | look for  | highlight? |
| -------------- | ---------- | ---------- | ---- | --------- | ---------- |
| Before a run   | calculator | failing.md |      | 1 │ 2 │ 3 | none       |
| A passing row  | calculator | failing.md | `R`  | 1 │ 2 │ 3 | green      |
| Its mark       | calculator | failing.md | `R`  | ✓         | green      |
| A failing row  | calculator | failing.md | `R`  | 2 │ 2 │ 5 | red        |
| Its mark       | calculator | failing.md | `R`  | ✗         | red        |
| An error       | outcomes   | errors.md  | `R`  | 1 │ 0 │ 0 | red        |
| A snoozed row  | calculator | snoozed.md | `R`  | 2 │ 2 │ 5 | yellow     |
| A skipped row  | outcomes   | dying.md   | `R`  | 3 │ 3 │ 6 | none       |
| The header     | calculator | failing.md | `] R` | sum?     | none       |

## Notes

A row that did not pass has a note on the line below it, indented by two columns,
with one line for each check that did not pass. A note is there to explain, not to
draw the eye, so it uses a quieter, greyer red than the row. The note of a snoozed
row is in the [quote colour](themes.md#colours), because nothing needs
doing about it yet.

The rest of the page moves down to make room. Only the first line of a message
shows. See [how messages are worded](../running/temoc-output.md#messages).

`next line?` reads the line below the `look for` text.

[//]: # "decisionTable Tui"

| # situation       | workspace  | arguments  | keys       | look for                        | next line?                                      |
| ----------------- | ---------- | ---------- | ---------- | ------------------------------- | ----------------------------------------------- |
| No note at first  | calculator | failing.md |            | 2 │ 3 │ 6                       | 2 │ 2 │ 5                                       |
| A passing row     | calculator | failing.md | `R`        | 2 │ 3 │ 6                       | 2 │ 2 │ 5     ✗                                 |
| A wrong value     | calculator | failing.md | `R`        | 2 │ 2 │ 5                       | mul expected 5, got 4                           |
| An exception      | outcomes   | errors.md  | `R`        | 1 │ 0 │ 0                       | div expected 0, got exception division by zero  |
| An unknown symbol | outcomes   | errors.md  | `R`        | $NOPE │ 2 │ 3                   | setA got unknown symbol $NOPE                   |
| A snoozed row     | calculator | snoozed.md | `R`        | 2 │ 2 │ 5                       | mul expected 5, got 4 (snoozed until 2099-12-31) |
| A lost connection | outcomes   | dying.md   | `R`        | 0 │ 9 │ 9                       | Missing message length terminator               |
| Beside a snooze   | calculator | mixed.md   | `R`        | 1 │ 2 │ 9                       | sum expected 9, got 3                           |
| The row after it  | outcomes   | hanging.md | `R Wait "0 │ 9 │ 9              ⋯"` | hangOnZero expected 5, got 4 | 0 │ 9 │ 9              ⋯ |

## Failures outside a row

[Failures that belong to a table](../running/row-results.md#failures-outside-a-row) and
not to a row are listed below the table's result, in the same quiet red.

[//]: # "decisionTable Tui"

| # situation     | workspace | arguments | keys  | look for                            | visible? |
| --------------- | --------- | --------- | ----- | ----------------------------------- | -------- |
| A missing class | outcomes  | errors.md | `R G` | make got exception NO_CLASS Missing | yes      |

## While a run goes on

The row temoc is working on shows `⋯`, and so does its table's result. The status bar
says what is running.

The `hanging.md` page has a row that never finishes, which holds the run at the
third row of its first table. `Wait` waits for that row to show its mark.

[//]: # "decisionTable Tui"

| # situation   | workspace | arguments  | keys                                | row results?                        | table results?                   | file result? | status?               |
| ------------- | --------- | ---------- | ----------------------------------- | ----------------------------------- | -------------------------------- | ------------ | --------------------- |
| In the middle | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯"` | 11: passed, 12: failed, 13: running | Stuck: running, Waiting: not run | running      | Running hanging.md... |

[//]: # "decisionTable Tui"

| # situation         | workspace | arguments  | keys                                | look for                       | visible? |
| ------------------- | --------- | ---------- | ----------------------------------- | ------------------------------ | -------- |
| The table's result  | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯"` | ⋯ running                      | yes      |
| A note arrives      | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯"` | hangOnZero expected 5, got 4   | yes      |
| A later row is bare | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯"` | 3 │ 3 │ 6              ⋯       | no       |
| The file result     | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯"` | file: running                  | yes      |

The page stays usable, and the keys that run something are refused.

[//]: # "decisionTable Tui"

| # situation      | workspace | arguments  | keys                                                  | document?  | row results?                        | status?                  | still open? |
| ---------------- | --------- | ---------- | ----------------------------------------------------- | ---------- | ----------------------------------- | ------------------------ | ----------- |
| Running again    | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯" r`                 | hanging.md | 11: passed, 12: failed, 13: running | temoc is already running | yes         |
| Leaving the page | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯" / "errors" Enter h` | hanging.md | 11: passed, 12: failed, 13: running | Running hanging.md...    | yes         |

Quitting [stops the run](../running/row-results.md#while-a-run-goes-on).

[//]: # "decisionTable Tui"

| # situation      | workspace | arguments  | keys                                  | still open? |
| ---------------- | --------- | ---------- | ------------------------------------- | ----------- |
| Quitting         | outcomes  | hanging.md | `R Wait "0 │ 9 │ 9              ⋯" q` | no          |

## Starting again

Starting a run [clears the results](../running/row-results.md#starting-again) of what
it is about to run, and the screen follows.

[//]: # "decisionTable Tui"

| # situation       | workspace | arguments  | keys                                      | table results?                   |
| ----------------- | --------- | ---------- | ----------------------------------------- | -------------------------------- |
| A file run clears | outcomes  | hanging.md | `] r [ R Wait "0 │ 9 │ 9              ⋯"` | Stuck: running, Waiting: not run |
