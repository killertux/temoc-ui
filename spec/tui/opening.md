# Opening temoc-ui

Back to the [specification index](../README.md).

temoc-ui takes one optional path, plus the `--config` option described in
[Configuration](../running/configuration.md) and the `--theme` option described in
[Themes](themes.md):

```sh
temoc-ui [PATH] [--config FILE] [--theme NAME]
```

The path decides [the workspace and what opens first](../viewer/opening.md). If
nothing opens, search opens so that you can pick a file.

[//]: # "import Spec.Fixtures"
[//]: # "decisionTable Tui"

| # situation             | workspace | arguments    | document?    | search open? |
| ----------------------- | --------- | ------------ | ------------ | ------------ |
| No path                 | docs      |              | README.md    | no           |
| A path                  | docs      | reference.md | reference.md | no           |
| Folder without a README | docs      | guide        | none         | yes          |

The workspace follows from the path too.

[//]: # "decisionTable Tui"

| # situation        | workspace | arguments | results?                         |
| ------------------ | --------- | --------- | -------------------------------- |
| Folder replaces it | docs      | guide     | getting-started.md, long-page.md |

`--config` can come before or after the path, and reaches temoc.

[//]: # "decisionTable Tui"

| # situation        | workspace     | arguments                       | keys | temoc arguments?                         |
| ------------------ | ------------- | ------------------------------- | ---- | ---------------------------------------- |
| Given with --config | custom-config | tests.md --config ci/temoc.toml | `R`  | -c ci/temoc.toml --output jsonl tests.md |

## Paths that cannot be opened

temoc-ui prints a message and exits without opening.

[//]: # "decisionTable Tui"

| # situation  | workspace | arguments | still open? | status?                       |
| ------------ | --------- | --------- | ----------- | ----------------------------- |
| Missing path | docs      | nope.md   | no          | Path not found: nope.md       |
| Not markdown | docs      | data.txt  | no          | Not a markdown file: data.txt |

## Arguments that make no sense

[//]: # "decisionTable Tui"

| # situation           | workspace | arguments              | still open? | status?                     |
| --------------------- | --------- | ---------------------- | ----------- | --------------------------- |
| Unknown option        | docs      | --nope                 | no          | Unknown option: --nope      |
| Config without a file | docs      | --config               | no          | --config needs a file       |
| Two paths             | docs      | README.md reference.md | no          | Only one path can be opened |
