# Reading temoc's Output

Back to the [specification index](../README.md).

temoc prints plain text. It lists failures but not passes, so anything in a table
that ran and is not listed has passed.

The result comes from the exit code together with the output. The word temoc prints
is not enough by itself: it prints `SNOOZED` if any failure is snoozed, even when
another failure is real and the exit code is 1.

The `TemocOutput` fixture reads output by itself. In the `stdout` column, `\n`
stands for a line break.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable TemocOutput"

| # situation            | stdout                                                                                                       | stderr                           | exit code | result? |
| ---------------------- | ------------------------------------------------------------------------------------------------------------ | -------------------------------- | --------- | ------- |
| All passed             | `Testing file a.md...OK`                                                                                     |                                  | 0         | passed  |
| A failure              | ``Testing file a.md...FAIL\nExpected `5`, got `4` in a.md:20:11 for method call mul``                        | `Error: Tests executed with error` | 1       | failed  |
| Only snoozed failures  | ``Testing file a.md...SNOOZED\nExpected `5`, got `4` in a.md:20:11 for method call mul -- snoozed until 2099-12-31`` |                         | 0         | snoozed |
| Snoozed and real       | ``Testing file a.md...SNOOZED\nExpected `9`, got `3` in a.md:11:11 for method call sum``                     | `Error: Tests executed with error` | 1       | failed  |
| Nothing selected       | `Testing file a.md...NONE`                                                                                   |                                  | 0         | no tests |
| temoc stopped          | `Testing file a.md...`                                                                                       | `Error: failed to fill whole buffer` | 1     | error   |

## Failure lines

Each failure line gives what was expected, what came back, and where. The position
places the failure in a table and a row. Fixture exceptions and setter failures
have the same shape.

[//]: # "decisionTable TemocOutput"

| # situation     | stdout                                                                                                | exit code | failures?                                                 |
| --------------- | ----------------------------------------------------------------------------------------------------- | --------- | --------------------------------------------------------- |
| Wrong value     | ``Testing file a.md...FAIL\nExpected `5`, got `4` in a.md:20:11 for method call mul``                 | 1         | line 20: mul expected 5, got 4                            |
| Exception       | ``Testing file a.md...FAIL\nExpected `5`, got Exception `boom` in a.md:20:11 for method call mul``    | 1         | line 20: mul expected 5, got exception boom               |
| Failing setter  | ``Testing file a.md...FAIL\nExpected NULL or VOID, got Exception `bad` in a.md:20:3 for method call setA`` | 1    | line 20: setA expected no value, got exception bad        |
| No method name  | ``Testing file a.md...FAIL\nExpected OK, got Exception `no class` in a.md:7:1``                       | 1         | line 7: expected OK, got exception no class               |
| Snoozed         | ``Testing file a.md...SNOOZED\nExpected `5`, got `4` in a.md:20:11 for method call mul -- snoozed until 2099-12-31`` | 0 | line 20: mul expected 5, got 4 (snoozed until 2099-12-31) |
