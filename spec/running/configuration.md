# Configuration

Back to the [specification index](../README.md).

temoc needs to know how to start your Slim server, and that lives in a temoc
configuration file. Viewing markdown needs no configuration, only running does.

temoc-ui uses the file it was given, or else `temoc.toml` at the root of the
[workspace](../viewer/opening.md#the-workspace). Its contents are for temoc, and
temoc-ui passes the file along without reading it.

temoc itself is found through your `PATH`, and runs with the workspace as its
current folder. It must be [version 0.10 or later](temoc-output.md). A relative
configuration path starts from the folder temoc-ui was started in, and temoc-ui passes
it along so that temoc finds the same file.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Core"

| # situation            | workspace     | path       | config        | steps      | temoc arguments?                         |
| ---------------------- | ------------- | ---------- | ------------- | ---------- | ---------------------------------------- |
| Found in the workspace | calculator    | passing.md |               | `run file` | -c temoc.toml --output jsonl passing.md  |
| Given to temoc-ui      | custom-config | tests.md   | ci/temoc.toml | `run file` | -c ci/temoc.toml --output jsonl tests.md |

## Without configuration

Pages still open, but nothing runs.

[//]: # "decisionTable Core"

| # situation             | workspace    | path     | config    | steps         | document? | temoc arguments? | problem?                                                      |
| ----------------------- | ------------ | -------- | --------- | ------------- | --------- | ---------------- | ------------------------------------------------------------- |
| No configuration        | unconfigured | tests.md |           | `run file`    | tests.md  | none             | No temoc configuration found. Add temoc.toml or use --config. |
| Running a single table  | unconfigured | tests.md |           | `run table 3` | tests.md  | none             | No temoc configuration found. Add temoc.toml or use --config. |
| Running a single row    | unconfigured | tests.md |           | `run row 7`   | tests.md  | none             | No temoc configuration found. Add temoc.toml or use --config. |
| The file is missing     | unconfigured | tests.md | nope.toml | `run file`    | tests.md  | none             | Configuration file not found: nope.toml                       |
