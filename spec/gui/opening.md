# Opening the GUI

Back to the [specification index](../README.md).

`--gui` opens temoc-ui in a window. It takes the same path and `--config` as the
[terminal](../tui/opening.md):

```sh
temoc-ui --gui [PATH] [--config FILE]
```

The GUI is behind the `gui` feature of the crate, so that a build for the terminal does
not carry it. Without that feature, temoc-ui prints
`This build has no GUI. Rebuild it with the gui feature.` and exits.

The path decides [the workspace and what opens first](../viewer/opening.md). If nothing
opens, [search](search.md) opens so that you can pick a file.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation             | workspace | arguments    | document?    | search open? |
| ----------------------- | --------- | ------------ | ------------ | ------------ |
| No path                 | docs      |              | README.md    | no           |
| A path                  | docs      | reference.md | reference.md | no           |
| Folder without a README | docs      | guide        | none         | yes          |

`--config` reaches temoc as it does from the terminal.

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation    | workspace     | arguments                       | actions | temoc arguments?                         |
| -------------- | ------------- | ------------------------------- | ------- | ---------------------------------------- |
| With --config  | custom-config | tests.md --config ci/temoc.toml | `R`     | -c ci/temoc.toml --output jsonl tests.md |

## What cannot be opened

temoc-ui prints a message and exits before any window opens, as the terminal does.
Themes are for the terminal, so `--theme` is refused with `--gui`. The GUI has
[its own two modes](themes.md).

[//]: # "decisionTable Gui -- snooze until 2099-12-31"

| # situation  | workspace | arguments    | still open? | status?                       |
| ------------ | --------- | ------------ | ----------- | ----------------------------- |
| Missing path | docs      | nope.md      | no          | Path not found: nope.md       |
| Not markdown | docs      | data.txt     | no          | Not a markdown file: data.txt |
| A theme      | docs      | --theme dark | no          | --theme does not work with --gui |
