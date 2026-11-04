# Reading temoc's Output

Back to the [specification index](../README.md).

temoc-ui needs temoc 0.10 or later. It runs temoc with `--output jsonl`, which prints
one JSON object per line while the tests run. temoc-ui reads each line as it arrives,
and that is how a row can show its [result](row-results.md) before the run is over.
temoc's [readme](https://github.com/killertux/temoc#machine-readable-output) describes
every event.

The `TemocOutput` fixture reads events by itself. In the `events` column, `\n`
separates the lines.

## The result of a run

The result is the `status` of `run_finished`, which temoc always prints last. The
`message` of a `run_error` is what temoc-ui shows for that run.

temoc-ui ignores event types and fields it does not know, as temoc asks of it.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable TemocOutput"

| # situation     | events                                                                                                                                                                   | stderr | exit code | result? | message?                                   |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------ | --------- | ------- | ------------------------------------------ |
| All passed      | `{"type":"run_started","schema":1}\n{"type":"run_finished","status":"passed","exit_code":0}`                                                                             |        | 0         | passed  | none                                       |
| A failure       | `{"type":"run_started","schema":1}\n{"type":"run_finished","status":"failed","exit_code":1}`                                                                             |        | 1         | failed  | none                                       |
| Only snoozed    | `{"type":"run_started","schema":1}\n{"type":"run_finished","status":"snoozed","exit_code":0}`                                                                            |        | 0         | snoozed | none                                       |
| An exception    | `{"type":"run_started","schema":1}\n{"type":"run_finished","status":"error","exit_code":1}`                                                                              |        | 1         | error   | none                                       |
| Nothing matched | `{"type":"run_started","schema":1}\n{"type":"run_error","kind":"no_match","message":"No decision table or row at a.md:9","file":"a.md"}\n{"type":"run_finished","status":"error","exit_code":1}` |  | 1 | error | No decision table or row at a.md:9 |
| Unknown event   | `{"type":"run_started","schema":1}\n{"type":"brand_new"}\n{"type":"run_finished","status":"passed","exit_code":0}`                                                       |        | 0         | passed  | none                                       |
| Unknown field   | `{"type":"run_started","schema":1,"extra":[1]}\n{"type":"run_finished","status":"passed","exit_code":0,"extra":{}}`                                                      |        | 0         | passed  | none                                       |

## When the output cannot be trusted

Anything short of a `run_finished` that temoc-ui can read is an error, and the message
says what happened. Only the first line of `stderr` is shown.

[//]: # "decisionTable TemocOutput"

| # situation       | events                              | stderr                                        | exit code | result? | message?                                                                       |
| ----------------- | ----------------------------------- | --------------------------------------------- | --------- | ------- | ------------------------------------------------------------------------------ |
| An older temoc    |                                     | `error: unexpected argument '--output' found` | 2         | error   | temoc-ui needs temoc 0.10 or later: error: unexpected argument '--output' found |
| Silent and failed |                                     |                                               | 1         | error   | temoc-ui needs temoc 0.10 or later                                             |
| temoc crashed     | `{"type":"run_started","schema":1}` | `thread 'main' panicked`                      | 101       | error   | temoc stopped before finishing: thread 'main' panicked                         |
| Crash in silence  | `{"type":"run_started","schema":1}` |                                               | -1        | error   | temoc stopped before finishing                                                 |
| A newer format    | `{"type":"run_started","schema":2}` |                                               | 0         | error   | temoc-ui reads schema 1 of temoc's output, but this temoc uses schema 2        |
| Not an event      | `{"type":"run_started","schema":1}\nnot json` |                                     | 0         | error   | temoc printed something temoc-ui cannot read: not json                         |

## Messages

A row that did not pass gets a message for each check that did not pass. It says
which method it was, what was expected, if anything was, and what came back. The
same words are used in the [note under the row](row-results.md#notes). Elsewhere they
come after the line, as they do here.

A snooze only shows while it is active. temoc reports an expired snooze as a normal
failure.

[//]: # "decisionTable TemocOutput"

| # situation     | events                                                                                                                                                                                                  | failures?                                                 |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| A wrong value   | `{"type":"row_finished","table":15,"line":20,"status":"failed","checks":[{"role":"getter","method":"mul","status":"failed","expected":"5","actual":{"kind":"value","value":"4"}}]}`                     | line 20: mul expected 5, got 4                            |
| No value        | `{"type":"row_finished","table":15,"line":20,"status":"failed","checks":[{"role":"getter","method":"mul","status":"failed","expected":"5","actual":{"kind":"void"}}]}`                                  | line 20: mul expected 5, got no value                     |
| A list          | `{"type":"row_finished","table":15,"line":20,"status":"failed","checks":[{"role":"getter","method":"mul","status":"failed","expected":"5","actual":{"kind":"list","value":[{"kind":"value","value":"a"},{"kind":"void"}]}}]}` | line 20: mul expected 5, got [a, no value]    |
| An exception    | `{"type":"row_finished","table":15,"line":20,"status":"error","checks":[{"role":"getter","method":"mul","status":"error","expected":"5","actual":{"kind":"exception","message":"boom"}}]}`              | line 20: mul expected 5, got exception boom               |
| Several lines   | `{"type":"row_finished","table":15,"line":20,"status":"error","checks":[{"role":"getter","method":"mul","status":"error","expected":"5","actual":{"kind":"exception","message":"boom\u000aat line 1"}}]}` | line 20: mul expected 5, got exception boom              |
| A failing setter | `{"type":"row_finished","table":15,"line":20,"status":"error","checks":[{"role":"setter","method":"setA","status":"error","value":"x","actual":{"kind":"exception","message":"bad"}}]}`                | line 20: setA got exception bad                           |
| Unknown symbol  | `{"type":"row_finished","table":15,"line":20,"status":"error","checks":[{"role":"setter","method":"setA","status":"error","value":"$NOPE","actual":{"kind":"missing_symbol","symbol":"NOPE"}}]}`        | line 20: setA got unknown symbol $NOPE                    |
| Bad response    | `{"type":"row_finished","table":15,"line":20,"status":"error","checks":[{"role":"getter","method":"mul","status":"error","expected":"5","actual":{"kind":"protocol_error","message":"Different ID"}}]}`  | line 20: mul expected 5, got protocol error Different ID  |
| Snoozed         | `{"type":"row_finished","table":15,"line":20,"status":"snoozed","checks":[{"role":"getter","method":"mul","status":"snoozed","expected":"5","actual":{"kind":"value","value":"4"},"snoozed_until":"2099-12-31"}]}` | line 20: mul expected 5, got 4 (snoozed until 2099-12-31) |
| Expired snooze  | `{"type":"row_finished","table":15,"line":20,"status":"failed","checks":[{"role":"getter","method":"mul","status":"failed","expected":"5","actual":{"kind":"value","value":"4"},"snoozed_until":"2020-01-01"}]}` | line 20: mul expected 5, got 4                       |
| Two checks      | `{"type":"row_finished","table":15,"line":20,"status":"failed","checks":[{"role":"getter","method":"sum","status":"failed","expected":"1","actual":{"kind":"value","value":"2"}},{"role":"getter","method":"mul","status":"failed","expected":"3","actual":{"kind":"value","value":"4"}}]}` | line 20: sum expected 1, got 2; line 20: mul expected 3, got 4 |
| Passed checks   | `{"type":"row_finished","table":15,"line":20,"status":"passed","checks":[{"role":"getter","method":"mul","status":"passed","expected":"4","actual":{"kind":"value","value":"4"}}]}`                     | none                                                      |
| Skipped checks  | `{"type":"row_finished","table":15,"line":20,"status":"skipped","checks":[{"role":"getter","method":"mul","status":"skipped","expected":"4"}]}`                                                         | none                                                      |
| Lost connection | `{"type":"row_finished","table":15,"line":20,"status":"error","checks":[{"role":"getter","method":"mul","status":"skipped","expected":"4"}],"message":"Connection lost"}`                               | line 20: Connection lost                                  |
| A failed setup  | `{"type":"setup_failed","line":15,"phase":"make","status":"error","actual":{"kind":"exception","message":"NO_CLASS Nope"}}`                                                                             | line 15: make got exception NO_CLASS Nope                 |
