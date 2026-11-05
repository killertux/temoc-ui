# Row Results

Back to the [specification index](../README.md).

Every row that runs gets its own result, and it arrives as soon as temoc finishes
that row, while the rest of the file is still running.

`failed` is a wrong value. `error` is anything else that stopped a row from giving
an answer, such as an exception or an unknown symbol. `skipped` is a row that never
ran. A row that has no result yet has none in `row results?`.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation    | workspace  | path       | steps          | row results?                                    |
| -------------- | ---------- | ---------- | -------------- | ----------------------------------------------- |
| Before a run   | calculator | failing.md |                | none                                            |
| A whole file   | calculator | failing.md | `run file`     | 11: passed, 19: passed, 20: failed              |
| Snoozed        | calculator | snoozed.md | `run file`     | 11: passed, 19: passed, 20: snoozed             |
| Errors         | outcomes   | errors.md  | `run file`     | 11: passed, 12: error, 20: error, 28: skipped   |
| Server stopped | outcomes   | dying.md   | `run file`     | 11: passed, 12: error, 13: skipped, 21: skipped |
| A table        | calculator | failing.md | `run table 15` | 19: passed, 20: failed                          |

## Failures outside a row

Some failures belong to a table and not to a row, such as a fixture class that does
not exist. temoc says that a failed import is on the line of its directive, which is
above every table, so it belongs to the first table that ran.

[//]: # "decisionTable Core"

| # situation | workspace | path      | steps      | table results?                                                 | failures?                                                                                                                                                                         |
| ----------- | --------- | --------- | ---------- | -------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All of them | outcomes  | errors.md | `run file` | Division: error, Unknown Symbol: error, No Such Fixture: error | Division line 12: div expected 0, got exception division by zero; Unknown Symbol line 20: setA got unknown symbol $NOPE; No Such Fixture line 24: make got exception NO_CLASS Missing |

## While a run goes on

Results arrive one row at a time. The row temoc is working on is `running`, the
rows before it have their results, and the rows after it have nothing yet. A table
is `running` from its first row to its last, and the same goes for the file.

The `hanging.md` page has a row that never finishes, which holds the run at the
third row of its first table. `wait for row 13` waits for that row to be `running`.

[//]: # "decisionTable Core"

| # situation   | workspace | path       | steps                       | row results?                        | table results?                   | file result? | running?   |
| ------------- | --------- | ---------- | --------------------------- | ----------------------------------- | -------------------------------- | ------------ | ---------- |
| In the middle | outcomes  | hanging.md | `run file, wait for row 13` | 11: passed, 12: failed, 13: running | Stuck: running, Waiting: not run | running      | hanging.md |

You can keep going. Results keep going to the file that is running, and only one run
can go at a time.

[//]: # "decisionTable Core"

| # situation      | workspace | path       | steps                                         | document?  | row results?                        | problem?                 | running?   |
| ---------------- | --------- | ---------- | --------------------------------------------- | ---------- | ----------------------------------- | ------------------------ | ---------- |
| Running again    | outcomes  | hanging.md | `run file, wait for row 13, run table 7`      | hanging.md | 11: passed, 12: failed, 13: running | temoc is already running | hanging.md |
| Leaving the page | outcomes  | hanging.md | `run file, wait for row 13, open errors.md, back` | hanging.md | 11: passed, 12: failed, 13: running | none                     | hanging.md |

Closing temoc-ui stops the run. temoc-ui asks temoc to stop and gives it time to stop
the Slim server it started, so that the server is not left behind.

## Starting again

Starting a run clears the results of what it is about to run, right away, so that
an old result is never mistaken for a new one. Results of anything else stay.

[//]: # "decisionTable Core"

| # situation              | workspace | path       | steps                                     | table results?                   |
| ------------------------ | --------- | ---------- | ----------------------------------------- | -------------------------------- |
| A file run clears        | outcomes  | hanging.md | `run table 18, run file, wait for row 13` | Stuck: running, Waiting: not run |
| A table run keeps others | outcomes  | hanging.md | `run table 18, run table 7, wait for row 13` | Stuck: running, Waiting: passed  |
