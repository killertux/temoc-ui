# Finding Tests

Back to the [specification index](../README.md).

A file is a test file when temoc would find at least one `decisionTable` directive
in it. temoc-ui uses the same rules as temoc, so the
[test file filter in search](../search.md#showing-only-test-files) and what temoc
actually runs always agree.

The `TemocDetection` fixture looks at one file in the
[`detection`](../../samples/detection) sample. `tables?` counts the runnable test
tables.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable TemocDetection"

| # situation                  | file              | test file? | tables? |
| ---------------------------- | ----------------- | ---------- | ------- |
| Directive in double quotes   | double-quoted.md  | yes        | 1       |
| Directive in parentheses     | parenthesized.md  | yes        | 1       |
| Several tables               | three-tables.md   | yes        | 3       |
| A normal table is not a test | plain.md          | no         | 0       |
| An import alone is not a test | import-only.md   | no         | 0       |
| Example inside a code block  | fenced-code.md    | no         | 0       |
| Directive inside a quote     | block-quote.md    | no         | 0       |
| Directive with no table      | missing-table.md  | yes        | 0       |

A directive with no table still counts as a test file so that you can find it and
see temoc's error when you
[run it](running-a-file.md#when-temoc-cannot-finish).

## Table names

A test table is named after the nearest heading above it. A table with no heading
above it uses its fixture class. Names appear in the results of a run.

[//]: # "decisionTable Tui"

| # situation        | workspace  | arguments        | focused table? |
| ------------------ | ---------- | ---------------- | -------------- |
| Named by a heading | calculator | passing.md       | Addition       |
| Uses the fixture   | detection  | double-quoted.md | Calculator     |
| Not a test file    | docs       | reference.md     | none           |
