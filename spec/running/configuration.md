# Configuration

Back to the [specification index](../README.md).

temoc needs to know how to start your Slim server, and that lives in a temoc
configuration file. Viewing markdown needs no configuration, only running does.

temoc-ui uses the file given with `--config`, or else `temoc.toml` at the root of
the [workspace](../viewer/opening.md#the-workspace). Its contents are for temoc, and
temoc-ui passes the file along without reading it.

temoc itself is found through your `PATH`, and runs with the workspace as its
current folder.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation                | workspace     | arguments                        | keys | temoc arguments?                       |
| -------------------------- | ------------- | -------------------------------- | ---- | -------------------------------------- |
| Found in the workspace     | calculator    | passing.md                       | `R`  | -c temoc.toml -s passing.md            |
| Given with --config        | custom-config | tests.md --config ci/temoc.toml  | `R`  | -c ci/temoc.toml -s tests.md           |

## Without configuration

Pages still open, but nothing runs.

[//]: # "decisionTable Tui"

| # situation             | workspace     | arguments                  | keys | temoc arguments? | status?                                                  |
| ----------------------- | ------------- | -------------------------- | ---- | ---------------- | -------------------------------------------------------- |
| No configuration        | unconfigured  | tests.md                   | `R`  | none             | No temoc configuration found. Add temoc.toml or use --config. |
| Running a single table  | unconfigured  | tests.md                   | `r`  | none             | No temoc configuration found. Add temoc.toml or use --config. |
| --config file is missing | unconfigured | tests.md --config nope.toml | `R` | none             | Configuration file not found: nope.toml                  |
